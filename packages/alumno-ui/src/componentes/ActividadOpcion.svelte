<script lang="ts">
  import type { Actividad } from "@rlp/curso";
  import { Markdown } from "@rlp/ui-comun";
  import { onMount } from "svelte";
  import { backend } from "../lib/backend";
  import { actualizarActividad, app, avisar, refrescarEstadisticas, registrarEvento } from "../lib/app.svelte";

  let { actividad }: { actividad: Actividad } = $props();
  // svelte-ignore state_referenced_locally (el componente se recrea al cambiar de actividad)
  const id = actividad.id;
  // svelte-ignore state_referenced_locally (el componente se recrea al cambiar de actividad)
  const opciones = actividad.opciones ?? [];
  const estado = $derived(app.alumno?.actividades[id]);
  let elegida: number | null = $state(null);
  let revisada: number | null = $state(null);

  onMount(() => {
    const previa = opciones.findIndex((o) => o.texto === estado?.respuesta);
    if (previa >= 0) {
      elegida = previa;
      revisada = previa;
    }
    void registrarEvento("actividad_abierta", id, {});
  });

  async function verificar() {
    if (elegida === null) return;
    revisada = elegida;
    const o = opciones[elegida];
    const antes = estado?.completada;
    const e = await (await backend()).registrarRespuesta(id, o.texto, o.correcta, actividad.puntos);
    actualizarActividad(id, e);
    refrescarEstadisticas();
    if (o.correcta && !antes) avisar(`🎉 ¡Correcto! +${actividad.puntos} puntos`);
  }
</script>

<div class="opcion">
  <Markdown contenido={actividad.enunciado} />
  <fieldset>
    <legend class="suave">Elige una respuesta</legend>
    {#each opciones as o, i (i)}
      <label class="op" class:elegida={elegida === i} class:bien={revisada === i && o.correcta} class:mal={revisada === i && !o.correcta}>
        <input type="radio" name="op-{id}" value={i} bind:group={elegida} />
        <span><Markdown contenido={o.texto} /></span>
      </label>
    {/each}
  </fieldset>
  <button class="primario" onclick={verificar} disabled={elegida === null}>Verificar</button>
  {#if revisada !== null}
    {@const o = opciones[revisada]}
    <div class={o.correcta ? "exito-msg" : "error"}>
      {o.correcta ? "✔ ¡Correcto!" : "✖ No es correcta."}
      {#if o.explicacion}<Markdown contenido={o.explicacion} />{/if}
    </div>
  {/if}
</div>

<style>
  .opcion {
    max-width: 820px;
    margin: 0 auto;
    padding: 1.2rem 1.5rem 3rem;
    width: 100%;
    overflow: auto;
  }
  fieldset {
    border: none;
    padding: 0;
    margin: 1rem 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .op {
    display: flex;
    gap: 0.7rem;
    align-items: center;
    font-weight: normal;
    border: 1px solid var(--borde);
    border-radius: var(--radio-chico);
    padding: 0.4em 0.9em;
    background: var(--superficie);
    cursor: pointer;
    margin: 0;
  }
  .op input {
    width: auto;
  }
  .op :global(.markdown p) {
    margin: 0.2em 0;
  }
  .elegida {
    border-color: var(--primario);
  }
  .bien {
    border-color: var(--exito);
    background: var(--exito-suave);
  }
  .mal {
    border-color: var(--peligro);
    background: var(--peligro-suave);
  }
</style>
