<script lang="ts">
  import type { Actividad } from "@rlp/curso";
  import { app, curso, type Seleccion } from "../lib/app.svelte";

  let { ir }: { ir: (s: Seleccion) => void } = $props();

  const actividades = $derived(app.alumno?.actividades ?? {});
  let abiertas: Record<string, boolean> = $state({});

  const iconoTipo = { codigo: "⌨", prediccion: "🔮", opcion_multiple: "☑" };

  function estadoDe(a: Actividad): "completada" | "progreso" | "pendiente" {
    const e = actividades[a.id];
    if (e?.completada) return "completada";
    if (e && (e.intentos > 0 || (e.dispositivo && e.codigo !== (a.codigo_inicial ?? "")))) return "progreso";
    return "pendiente";
  }

  function avance(u: (typeof curso.unidades)[number]) {
    const todas = u.lecciones.flatMap((l) => l.actividades);
    const hechas = todas.filter((a) => actividades[a.id]?.completada).length;
    return { hechas, total: todas.length, pct: todas.length ? Math.round((hechas / todas.length) * 100) : 0 };
  }

  function activa(tipo: "leccion" | "actividad", id: string) {
    return app.seleccion.tipo === tipo && app.seleccion.id === id;
  }

  function abierta(uid: string, i: number) {
    if (uid in abiertas) return abiertas[uid];
    // Por defecto se abre la unidad actual (la primera con actividades pendientes).
    const primeraPendiente = curso.unidades.findIndex((u) => avance(u).pct < 100);
    return i === (primeraPendiente === -1 ? 0 : primeraPendiente);
  }
</script>

<nav class="temario" aria-label="Temario">
  {#each curso.unidades as u, i (u.id)}
    {@const av = avance(u)}
    <section class="unidad">
      <button class="cabeza" onclick={() => (abiertas[u.id] = !abierta(u.id, i))} aria-expanded={abierta(u.id, i)}>
        <span class="num">{u.numero}</span>
        <span class="texto">
          <strong>{u.titulo}</strong>
          <span class="barra-avance" aria-label="{av.pct}% completado"><span style:width="{av.pct}%"></span></span>
        </span>
        <span class="suave pct">{av.hechas}/{av.total}</span>
      </button>
      {#if abierta(u.id, i)}
        <ul>
          {#each u.lecciones as l (l.id)}
            <li>
              <button class="leccion" class:activa={activa("leccion", l.id)} onclick={() => ir({ tipo: "leccion", id: l.id })}>
                📖 {l.titulo}
              </button>
              <ul>
                {#each l.actividades as a (a.id)}
                  {@const est = estadoDe(a)}
                  <li>
                    <button class="actividad {est}" class:activa={activa("actividad", a.id)} onclick={() => ir({ tipo: "actividad", id: a.id })}
                      data-actividad={a.id}>
                      <span class="marca" aria-label={est}>{est === "completada" ? "✓" : est === "progreso" ? "●" : "○"}</span>
                      <span class="tipo" aria-hidden="true">{iconoTipo[a.tipo]}</span>
                      <span class="t">{a.titulo}</span>
                    </button>
                  </li>
                {/each}
              </ul>
            </li>
          {/each}
        </ul>
      {/if}
    </section>
  {/each}
</nav>

<style>
  .temario {
    padding: 0.5rem 0.4rem 2rem;
  }
  .unidad {
    margin-bottom: 0.2rem;
  }
  button {
    border: none;
    background: transparent;
    width: 100%;
    justify-content: flex-start;
    text-align: left;
    white-space: normal;
  }
  .cabeza {
    padding: 0.55em 0.5em;
  }
  .num {
    width: 1.8em;
    height: 1.8em;
    flex: none;
    border-radius: 6px;
    background: var(--primario-suave);
    color: var(--primario);
    display: grid;
    place-items: center;
    font-weight: 700;
  }
  .texto {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.25em;
  }
  .barra-avance {
    height: 4px;
    border-radius: 2px;
    background: var(--superficie-2);
    overflow: hidden;
  }
  .barra-avance span {
    display: block;
    height: 100%;
    background: var(--exito);
  }
  .pct {
    font-size: 0.8em;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0 0 0 0.6rem;
  }
  .leccion {
    font-weight: 600;
    font-size: 0.92em;
    padding: 0.35em 0.5em;
  }
  .actividad {
    font-size: 0.9em;
    padding: 0.25em 0.5em;
    gap: 0.45em;
    color: var(--texto);
  }
  .activa {
    background: var(--primario-suave);
    color: var(--primario);
  }
  .marca {
    width: 1em;
    text-align: center;
    color: var(--texto-suave);
  }
  .completada .marca {
    color: var(--exito);
    font-weight: 700;
  }
  .progreso .marca {
    color: var(--aviso);
  }
  .tipo {
    opacity: 0.7;
    font-size: 0.9em;
  }
  .t {
    flex: 1;
  }
</style>
