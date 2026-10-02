<script lang="ts">
  import { actividadesDe, type Actividad, type Leccion } from "@rlp/curso";
  import { curso, app, type Seleccion } from "../lib/app.svelte";
  import ActividadCodigo from "./ActividadCodigo.svelte";
  import ActividadPrediccion from "./ActividadPrediccion.svelte";
  import ActividadOpcion from "./ActividadOpcion.svelte";

  let { actividad, leccion, ir }: { actividad: Actividad; leccion: Leccion; ir: (s: Seleccion) => void } = $props();

  const orden = actividadesDe(curso);
  const i = $derived(orden.findIndex((a) => a.id === actividad.id));
  const anterior = $derived(i > 0 ? orden[i - 1] : null);
  const siguiente = $derived(i < orden.length - 1 ? orden[i + 1] : null);
  const estado = $derived(app.alumno?.actividades[actividad.id]);
  const nota = $derived(app.alumno?.retroalimentacion?.actividades[actividad.id]);
</script>

<div class="actividad">
  <header class="cabeza">
    <div class="titulo">
      <button class="fantasma chico miga" onclick={() => ir({ tipo: "leccion", id: leccion.id })}>📖 {leccion.titulo}</button>
      <h1>
        {actividad.titulo}
        {#if estado?.completada}<span class="insignia hecha">✓ Completada</span>{/if}
      </h1>
      <span class="suave meta">
        {"★".repeat(actividad.dificultad)}{"☆".repeat(3 - actividad.dificultad)} · {actividad.puntos} puntos
        {#if estado?.intentos}· {estado.intentos} intento(s){/if}
      </span>
    </div>
    <nav class="fila">
      <button class="chico" disabled={!anterior} onclick={() => anterior && ir({ tipo: "actividad", id: anterior.id })}>← Anterior</button>
      <button class="chico" disabled={!siguiente} onclick={() => siguiente && ir({ tipo: "actividad", id: siguiente.id })}>Siguiente →</button>
    </nav>
  </header>

  {#if nota}
    <aside class="retro" data-retroalimentacion>
      <span class="icono" aria-hidden="true">📬</span>
      <div>
        <strong>Tu profesor{app.alumno?.retroalimentacion?.profesor ? ` (${app.alumno.retroalimentacion.profesor})` : ""}:</strong>
        {#if nota.calificacion !== null}<span class="insignia calif">Calificación {nota.calificacion}</span>{/if}
        {#if nota.comentario}<p>{nota.comentario}</p>{/if}
      </div>
    </aside>
  {/if}

  {#if actividad.tipo === "codigo"}
    <ActividadCodigo {actividad} />
  {:else if actividad.tipo === "prediccion"}
    <ActividadPrediccion {actividad} />
  {:else}
    <ActividadOpcion {actividad} />
  {/if}
</div>

<style>
  .actividad {
    height: 100%;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .cabeza {
    display: flex;
    align-items: flex-end;
    gap: 1rem;
    padding: 0.6rem 1.2rem 0.6rem;
    border-bottom: 1px solid var(--borde);
    background: var(--superficie);
  }
  .titulo {
    flex: 1;
    min-width: 0;
  }
  .miga {
    padding-left: 0;
    color: var(--texto-suave);
  }
  h1 {
    font-size: 1.3rem;
    margin: 0.1em 0;
    display: flex;
    align-items: center;
    gap: 0.6rem;
    flex-wrap: wrap;
  }
  .meta {
    font-size: 0.88em;
  }
  .retro {
    display: flex;
    gap: 0.7rem;
    align-items: flex-start;
    padding: 0.55rem 1.2rem;
    background: var(--primario-suave);
    border-bottom: 1px solid var(--borde);
  }
  .retro p {
    margin: 0.2em 0 0;
    white-space: pre-wrap;
  }
  .retro .calif {
    margin-left: 0.4em;
  }
  .hecha {
    background: var(--exito-suave);
    color: var(--exito);
    border-color: transparent;
    font-size: 0.65em;
  }
</style>
