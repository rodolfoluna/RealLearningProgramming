<script lang="ts">
  import { actividadesDe } from "@rlp/curso";
  import { minutos, porcentaje } from "@rlp/ui-comun";
  import { app, curso, type Seleccion } from "../lib/app.svelte";

  let { ir }: { ir: (s: Seleccion) => void } = $props();

  const alumno = $derived(app.alumno!);
  const todas = actividadesDe(curso);
  const hechas = $derived(todas.filter((a) => alumno.actividades[a.id]?.completada).length);
  const siguiente = $derived(todas.find((a) => !alumno.actividades[a.id]?.completada));
  const puntos = $derived(Object.values(alumno.actividades).reduce((s, a) => s + (a.puntos ?? 0), 0));
  const puntosTotales = todas.reduce((s, a) => s + a.puntos, 0);
  const primerNombre = $derived(alumno.perfil.nombre.split(" ")[0]);
</script>

<div class="bienvenida">
  <h1>¡Hola, {primerNombre}! 👋</h1>
  <p class="suave">{curso.descripcion}</p>

  <div class="resumen">
    <div class="tarjeta dato">
      <span class="valor">{porcentaje(hechas, todas.length)}%</span>
      <span class="suave">del curso ({hechas} de {todas.length} actividades)</span>
    </div>
    <div class="tarjeta dato">
      <span class="valor">{puntos}</span>
      <span class="suave">puntos de {puntosTotales}</span>
    </div>
    <div class="tarjeta dato">
      <span class="valor">{minutos(alumno.estadisticas.global.tiempo_ms)}</span>
      <span class="suave">de práctica</span>
    </div>
  </div>

  {#if siguiente}
    <div class="tarjeta continuar">
      <div>
        <span class="suave">Continúa con</span>
        <h2>{siguiente.titulo}</h2>
      </div>
      <button class="primario" onclick={() => ir({ tipo: "actividad", id: siguiente.id })}>Continuar →</button>
    </div>
  {:else}
    <div class="exito-msg">🎉 ¡Completaste todas las actividades! Exporta tu entrega para tu profesor.</div>
  {/if}

  <h2 class="titulo-unidades">Unidades</h2>
  <div class="unidades">
    {#each curso.unidades as u (u.id)}
      {@const acts = u.lecciones.flatMap((l) => l.actividades)}
      {@const listas = acts.filter((a) => alumno.actividades[a.id]?.completada).length}
      <button class="tarjeta unidad" onclick={() => ir({ tipo: "leccion", id: u.lecciones[0].id })}>
        <span class="num">Unidad {u.numero}</span>
        <strong>{u.titulo}</strong>
        <span class="suave">{u.descripcion}</span>
        <span class="barra-avance"><span style:width="{porcentaje(listas, acts.length)}%"></span></span>
        <span class="suave chico">{listas}/{acts.length} actividades</span>
      </button>
    {/each}
  </div>
</div>

<style>
  .bienvenida {
    max-width: 980px;
    margin: 0 auto;
    padding: 2rem 1.5rem;
  }
  .resumen {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 1rem;
    margin: 1.5rem 0;
  }
  .dato {
    display: flex;
    flex-direction: column;
  }
  .valor {
    font-size: 1.8rem;
    font-weight: 700;
  }
  .continuar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    border-left: 4px solid var(--primario);
  }
  .continuar h2 {
    margin: 0.1em 0 0;
  }
  .titulo-unidades {
    margin-top: 2rem;
  }
  .unidades {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(230px, 1fr));
    gap: 1rem;
  }
  .unidad {
    flex-direction: column;
    align-items: flex-start;
    text-align: left;
    white-space: normal;
    gap: 0.3em;
    padding: 1rem;
  }
  .num {
    font-size: 0.8em;
    font-weight: 700;
    color: var(--primario);
    text-transform: uppercase;
  }
  .barra-avance {
    width: 100%;
    height: 5px;
    border-radius: 3px;
    background: var(--superficie-2);
    overflow: hidden;
    margin-top: 0.3em;
  }
  .barra-avance span {
    display: block;
    height: 100%;
    background: var(--exito);
  }
  .chico {
    font-size: 0.82em;
  }
</style>
