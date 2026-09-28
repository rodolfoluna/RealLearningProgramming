<script lang="ts">
  import { fecha, mensajeError, minutos, porcentaje, Semaforo } from "@rlp/ui-comun";
  import { backend } from "../lib/backend";
  import { actividades, app, avisar, curso } from "../lib/app.svelte";
  import type { FilaTablero } from "../lib/tipos";

  let filas: FilaTablero[] = $state([]);
  let cargando = $state(true);
  let error = $state("");
  let buscar = $state("");
  let mapa = $state(false);

  $effect(() => {
    const grupo = app.grupoId;
    cargando = true;
    backend()
      .then((b) => b.tablero(grupo))
      .then((f) => (filas = f))
      .catch((e) => (error = mensajeError(e)))
      .finally(() => (cargando = false));
  });

  const visibles = $derived(
    filas.filter((f) => `${f.nombre} ${f.numero_control}`.toLowerCase().includes(buscar.trim().toLowerCase())),
  );
  const completadas = (f: FilaTablero) => Object.values(f.actividades).filter((a) => a.completada).length;
  const puntos = (f: FilaTablero) => Object.values(f.actividades).reduce((s, a) => s + a.puntos, 0);
  const resumen = $derived({
    alumnos: filas.length,
    rojos: filas.filter((f) => f.nivel === "rojo").length,
    amarillos: filas.filter((f) => f.nivel === "amarillo").length,
    avance: filas.length ? Math.round(filas.reduce((s, f) => s + porcentaje(completadas(f), actividades.length), 0) / filas.length) : 0,
  });

  /** Primera actividad de cada unidad: ahí el mapa marca la separación. */
  const iniciosDeUnidad = new Set(curso.unidades.map((u) => u.lecciones.flatMap((l) => l.actividades)[0]?.id).filter(Boolean));

  function estadoCelda(f: FilaTablero, id: string): string {
    const a = f.actividades[id];
    if (!a) return "vacia";
    if (a.completada) return "hecha";
    return "progreso";
  }

  async function retroalimentacion() {
    try {
      const b = await backend();
      const carpeta = await b.elegirCarpeta("¿Dónde guardo el archivo de retroalimentación?");
      if (!carpeta) return;
      const r = await b.exportarRetroalimentacion(app.grupoId, carpeta);
      avisar(`Retroalimentación para ${r.alumnos} alumno(s) guardada en ${r.ruta}. Compártela con el grupo: cada alumno solo puede leer la suya.`, 8000);
    } catch (e) {
      avisar(mensajeError(e), 6000);
    }
  }

  async function exportar(formato: "xlsx" | "csv") {
    try {
      const b = await backend();
      const carpeta = await b.elegirCarpeta(formato === "xlsx" ? "¿Dónde guardo el archivo de Excel?" : "¿Dónde guardo el archivo CSV?");
      if (!carpeta) return;
      const pares = actividades.map((a) => [a.id, a.titulo] as [string, string]);
      const ruta = formato === "xlsx" ? await b.exportarXlsx(app.grupoId, pares, carpeta) : await b.exportarCsv(app.grupoId, pares, carpeta);
      avisar(`${formato === "xlsx" ? "Excel" : "CSV"} guardado en ${ruta}`, 6000);
    } catch (e) {
      avisar(mensajeError(e), 6000);
    }
  }
</script>

<div class="tablero">
  <div class="resumen">
    <div class="tarjeta dato"><span class="valor">{resumen.alumnos}</span><span class="suave">alumnos con entregas</span></div>
    <div class="tarjeta dato"><span class="valor">{resumen.avance}%</span><span class="suave">avance promedio</span></div>
    <div class="tarjeta dato"><span class="valor alerta-amarilla">{resumen.amarillos}</span><span class="suave">entregas por revisar</span></div>
    <div class="tarjeta dato"><span class="valor alerta-roja">{resumen.rojos}</span><span class="suave">entregas alteradas</span></div>
  </div>

  <div class="fila herramientas">
    <input class="buscar" placeholder="Buscar por nombre o número de control" bind:value={buscar} />
    <label class="fila interruptor"><input type="checkbox" bind:checked={mapa} /> Ver mapa de actividades</label>
    <span class="espaciador"></span>
    <button onclick={() => exportar("xlsx")} disabled={!filas.length} title="Hojas Resumen, Actividades y Calificaciones">⬇ Exportar a Excel</button>
    <button onclick={() => exportar("csv")} disabled={!filas.length}>CSV</button>
    <button onclick={retroalimentacion} disabled={!filas.length} title="Calificaciones y comentarios para los alumnos (.rlpr)">📬 Retroalimentación</button>
  </div>

  {#if error}
    <p class="error">{error}</p>
  {:else if cargando}
    <p class="suave">Cargando…</p>
  {:else if !filas.length}
    <div class="tarjeta vacio">
      <h2>Aún no hay entregas</h2>
      <p class="suave">Importa los archivos <code>.rlp</code> que te entreguen tus alumnos.</p>
      <button class="primario" onclick={() => (app.vista = { tipo: "importar" })}>📥 Importar entregas</button>
    </div>
  {:else}
    <div class="tabla">
      <table class="datos">
        <thead>
          <tr>
            <th class="fijo">Alumno</th>
            <th>Integridad</th>
            <th>Avance</th>
            <th class="num">Puntos</th>
            {#if mapa}
              {#each curso.unidades as u (u.id)}
                <th class="unidad-cab" colspan={u.lecciones.reduce((s, l) => s + l.actividades.length, 0)}>U{u.numero}</th>
              {/each}
            {:else}
              <th class="num">Tiempo</th>
              <th class="num">Ejecuciones</th>
              <th class="num">Copias</th>
              <th class="num">Intentos de pegar</th>
              <th class="num">Salidas</th>
              <th class="num">Pistas</th>
              <th>Última entrega</th>
            {/if}
          </tr>
        </thead>
        <tbody>
          {#each visibles as f (f.perfil_id)}
            <tr onclick={() => (app.vista = { tipo: "detalle", entregaId: f.entrega_id })} class="clic" data-alumno={f.numero_control}>
              <td class="fijo">
                <strong>{f.nombre}</strong>
                <div class="suave chico">{f.numero_control}{f.alerta_identidad ? " · ⚠ identidad duplicada" : ""}</div>
              </td>
              <td><Semaforo nivel={f.nivel} /></td>
              <td>
                <div class="avance">
                  <span class="barra"><span style:width="{porcentaje(completadas(f), actividades.length)}%"></span></span>
                  <span class="chico">{completadas(f)}/{actividades.length}</span>
                </div>
              </td>
              <td class="num">{puntos(f)}</td>
              {#if mapa}
                {#each actividades as a (a.id)}
                  <td class="celda {estadoCelda(f, a.id)}" class:inicio-unidad={iniciosDeUnidad.has(a.id)} title={a.titulo}></td>
                {/each}
              {:else}
                <td class="num">{minutos(f.global.tiempo_ms)}</td>
                <td class="num">{f.global.ejecuciones}</td>
                <td class="num">{f.global.copias}</td>
                <td class="num" class:alerta={f.global.pegados_intentos > 0}>{f.global.pegados_intentos}</td>
                <td class="num">{f.global.salidas}</td>
                <td class="num">{f.global.pistas}</td>
                <td>{fecha(f.creado)} <span class="suave chico">({f.entregas})</span></td>
              {/if}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<style>
  .tablero {
    padding: 1.2rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    min-height: 100%;
  }
  .resumen {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(170px, 1fr));
    gap: 0.8rem;
  }
  .dato {
    display: flex;
    flex-direction: column;
    padding: 0.8rem 1rem;
  }
  .valor {
    font-size: 1.6rem;
    font-weight: 700;
  }
  .alerta-amarilla {
    color: var(--aviso);
  }
  .alerta-roja {
    color: var(--peligro);
  }
  .buscar {
    max-width: 360px;
  }
  .interruptor {
    font-weight: normal;
    margin: 0;
  }
  .interruptor input {
    width: auto;
  }
  .tabla {
    overflow: auto;
    border: 1px solid var(--borde);
    border-radius: var(--radio);
    max-height: calc(100vh - 290px);
  }
  .clic {
    cursor: pointer;
  }
  .chico {
    font-size: 0.82em;
  }
  .avance {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .barra {
    width: 90px;
    height: 6px;
    background: var(--superficie-2);
    border-radius: 3px;
    overflow: hidden;
  }
  .barra span {
    display: block;
    height: 100%;
    background: var(--exito);
  }
  .unidad-cab {
    text-align: center !important;
    border-left: 2px solid var(--borde);
  }
  .celda {
    padding: 0 !important;
    width: 11px;
    min-width: 11px;
    border-left: 1px solid var(--superficie);
  }
  .celda.inicio-unidad {
    border-left: 2px solid var(--borde);
  }
  /* El nombre queda visible al desplazar el mapa de actividades a la derecha. */
  .fijo {
    position: sticky;
    left: 0;
    background: var(--superficie);
    z-index: 1;
    box-shadow: 1px 0 0 var(--borde);
  }
  th.fijo {
    background: var(--superficie-2);
    z-index: 2;
  }
  tr:hover .fijo {
    background: var(--primario-suave);
  }
  .celda.hecha {
    background: var(--exito);
  }
  .celda.progreso {
    background: var(--acento);
  }
  .celda.vacia {
    background: var(--superficie-2);
  }
  .vacio {
    text-align: center;
    padding: 3rem;
  }
</style>
