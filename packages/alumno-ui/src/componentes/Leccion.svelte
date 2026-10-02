<script lang="ts">
  import type { Leccion, Unidad } from "@rlp/curso";
  import { Markdown } from "@rlp/ui-comun";
  import { onMount } from "svelte";
  import { app, registrarEvento, type Seleccion } from "../lib/app.svelte";
  import Consola from "./Consola.svelte";

  let { leccion, unidad, ir }: { leccion: Leccion; unidad: Unidad; ir: (s: Seleccion) => void } = $props();
  let consola: Consola | undefined = $state();
  let consolaVisible = $state(false);

  onMount(() => {
    void registrarEvento("leccion_abierta", null, { leccion: leccion.id });
  });

  async function probar(codigo: string) {
    consolaVisible = true;
    await Promise.resolve();
    void consola?.ejecutar(codigo);
  }

  const estado = (id: string) => app.alumno?.actividades[id];
</script>

<div class="pagina">
  <article class="leccion">
    <p class="miga suave">Unidad {unidad.numero} · {unidad.titulo}</p>
    <Markdown
      contenido={leccion.contenido}
      alProbar={probar}
      alCopiar={(chars) => registrarEvento("copia", null, { chars, origen: "leccion", leccion: leccion.id })}
    />

    {#if leccion.actividades.length}
      <section class="actividades">
        <h2>Actividades de esta lección</h2>
        {#each leccion.actividades as a (a.id)}
          {@const e = estado(a.id)}
          <button class="tarjeta act" onclick={() => ir({ tipo: "actividad", id: a.id })}>
            <span class="marca" class:hecha={e?.completada}>{e?.completada ? "✓" : "○"}</span>
            <span class="t">
              <strong>{a.titulo}</strong>
              <span class="suave">{a.tipo === "codigo" ? "Programa" : a.tipo === "prediccion" ? "Predice la salida" : "Pregunta"} · {"★".repeat(a.dificultad)} · {a.puntos} pts</span>
            </span>
            <span aria-hidden="true">→</span>
          </button>
        {/each}
      </section>
    {/if}
  </article>
  {#if consolaVisible}
    <aside class="panel-consola">
      <div class="fila cabeza">
        <strong>Ejemplo</strong>
        <span class="espaciador"></span>
        <button class="chico" onclick={() => consola?.detener()}>■ Detener</button>
        <button class="chico" onclick={() => (consolaVisible = false)}>Cerrar</button>
      </div>
      <Consola bind:this={consola} compacta />
    </aside>
  {/if}
</div>

<style>
  .pagina {
    display: flex;
    flex-direction: column;
    min-height: 100%;
  }
  .leccion {
    max-width: 820px;
    width: 100%;
    margin: 0 auto;
    padding: 1.5rem 1.5rem 3rem;
    flex: 1;
  }
  .miga {
    margin: 0 0 0.5rem;
    font-size: 0.88em;
  }
  .actividades {
    margin-top: 2.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }
  .act {
    text-align: left;
    justify-content: flex-start;
    padding: 0.8rem 1rem;
    white-space: normal;
  }
  .t {
    flex: 1;
    display: flex;
    flex-direction: column;
  }
  .marca {
    font-size: 1.1em;
    width: 1.4em;
    color: var(--texto-suave);
  }
  .marca.hecha {
    color: var(--exito);
    font-weight: 700;
  }
  .panel-consola {
    position: sticky;
    bottom: 0;
    background: var(--superficie);
    border-top: 1px solid var(--borde);
    padding: 0.5rem;
    box-shadow: 0 -6px 20px rgb(0 0 0 / 0.08);
  }
  .cabeza {
    margin-bottom: 0.4rem;
  }
</style>
