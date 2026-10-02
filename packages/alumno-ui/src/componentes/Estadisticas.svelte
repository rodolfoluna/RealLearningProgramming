<script lang="ts">
  import { actividadesDe } from "@rlp/curso";
  import { minutos } from "@rlp/ui-comun";
  import { onMount } from "svelte";
  import { app, curso, refrescarEstadisticas } from "../lib/app.svelte";
  import { contadoresVacios } from "../lib/tipos";

  const est = $derived(app.alumno!.estadisticas);
  const g = $derived(est.global);
  const todas = actividadesDe(curso);
  const completadas = $derived(todas.filter((a) => app.alumno?.actividades[a.id]?.completada).length);

  onMount(refrescarEstadisticas);

  const tarjetas = $derived([
    { etiqueta: "Tiempo de práctica", valor: minutos(g.tiempo_ms) },
    { etiqueta: "Actividades completadas", valor: `${completadas} / ${todas.length}` },
    { etiqueta: "Ejecuciones", valor: g.ejecuciones },
    { etiqueta: "Ejecuciones con error", valor: g.errores },
    { etiqueta: "Veces que probaste", valor: g.pruebas },
    { etiqueta: "Pistas usadas", valor: g.pistas },
    { etiqueta: "Teclas escritas", valor: g.teclas },
    { etiqueta: "Copias", valor: g.copias, nota: "Copiar sí se permite; se cuenta." },
    { etiqueta: "Intentos de pegar", valor: g.pegados_intentos, alerta: g.pegados_intentos > 0 },
    { etiqueta: "Salidas de la ventana", valor: g.salidas, nota: `Tiempo fuera: ${minutos(g.tiempo_fuera_ms)}` },
  ]);
</script>

<div class="estadisticas">
  <h1>Mis estadísticas</h1>
  <p class="info">
    Estas son las mismas estadísticas que verá tu profesor cuando revise tu entrega. Se calculan a partir del historial
    que guarda la app mientras trabajas.
  </p>
  <div class="tarjetas">
    {#each tarjetas as t (t.etiqueta)}
      <div class="tarjeta dato" class:alerta={t.alerta}>
        <span class="valor">{t.valor}</span>
        <span class="suave">{t.etiqueta}</span>
        {#if t.nota}<span class="nota suave">{t.nota}</span>{/if}
      </div>
    {/each}
  </div>

  <h2>Por actividad</h2>
  <div class="tabla-envoltura">
    <table>
      <thead>
        <tr>
          <th>Actividad</th><th>Estado</th><th>Tiempo</th><th>Ejecuciones</th><th>Pruebas</th><th>Pistas</th><th>Copias</th><th>Pegar</th><th>Salidas</th>
        </tr>
      </thead>
      <tbody>
        {#each todas as a (a.id)}
          {@const c = est.por_actividad[a.id] ?? contadoresVacios()}
          {@const e = app.alumno?.actividades[a.id]}
          {#if e || c.tiempo_ms}
            <tr>
              <td>{a.titulo}</td>
              <td>{e?.completada ? "✓ Completada" : e?.total ? `${e.pasadas}/${e.total}` : "En progreso"}</td>
              <td>{minutos(c.tiempo_ms)}</td>
              <td>{c.ejecuciones}{c.errores ? ` (${c.errores} con error)` : ""}</td>
              <td>{c.pruebas}</td>
              <td>{c.pistas}</td>
              <td>{c.copias}</td>
              <td class:alerta={c.pegados_intentos > 0}>{c.pegados_intentos}</td>
              <td>{c.salidas}</td>
            </tr>
          {/if}
        {/each}
      </tbody>
    </table>
  </div>
</div>

<style>
  .estadisticas {
    max-width: 1080px;
    margin: 0 auto;
    padding: 1.5rem;
  }
  .tarjetas {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(170px, 1fr));
    gap: 0.8rem;
    margin: 1rem 0 2rem;
  }
  .dato {
    display: flex;
    flex-direction: column;
    padding: 0.9rem 1rem;
  }
  .valor {
    font-size: 1.5rem;
    font-weight: 700;
  }
  .nota {
    font-size: 0.8em;
  }
  .alerta {
    color: var(--aviso);
  }
  .tarjeta.alerta {
    border-color: color-mix(in srgb, var(--aviso) 40%, var(--borde));
  }
  .tabla-envoltura {
    overflow-x: auto;
  }
  table {
    border-collapse: collapse;
    width: 100%;
    background: var(--superficie);
    font-size: 0.92em;
  }
  th,
  td {
    text-align: left;
    padding: 0.45em 0.7em;
    border-bottom: 1px solid var(--borde);
    white-space: nowrap;
  }
  th {
    background: var(--superficie-2);
    font-weight: 600;
  }
</style>
