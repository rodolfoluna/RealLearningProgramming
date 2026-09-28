<script lang="ts">
  import type { Actividad } from "@rlp/curso";
  import { crearVisor, type EditorCodigo } from "@rlp/editor";
  import type { ResultadoPruebas } from "@rlp/python-worker";
  import { fecha, Markdown, mensajeError, minutos, Modal, Semaforo } from "@rlp/ui-comun";
  import { onDestroy, onMount } from "svelte";
  import { backend } from "../lib/backend";
  import { actividades, app, avisar, curso, python } from "../lib/app.svelte";
  import { contadoresVacios, type ArchivoCreado, type DetalleEntrega, type LineaDeTiempo } from "../lib/tipos";
  import Reproductor from "./Reproductor.svelte";

  let { entregaId }: { entregaId: number } = $props();
  let d = $state<DetalleEntrega | null>(null);
  let error = $state("");
  let pestana: "integridad" | "actividades" | "estadisticas" = $state("integridad");
  let elegida = $state<Actividad | null>(null);
  let visorPadre: HTMLDivElement | undefined = $state();
  let visor: EditorCodigo | null = null;
  let pruebas = $state<ResultadoPruebas | null>(null);
  let probando = $state(false);
  let calificacion = $state("");
  let comentario = $state("");
  let reproduccion = $state<LineaDeTiempo | null>(null);
  let confirmarAcceso = $state(false);
  let acceso = $state<ArchivoCreado | null>(null);
  let cargandoReproduccion = $state(false);

  onMount(async () => {
    try {
      // svelte-ignore state_referenced_locally (el componente se recrea al cambiar de entrega)
      d = await (await backend()).detalle(entregaId);
      if (d.reporte.nivel === "verde") pestana = "actividades";
      elegida = actividades.find((a) => d!.actividades[a.id]) ?? null;
    } catch (e) {
      error = mensajeError(e);
    }
  });

  onDestroy(() => visor?.destruir());

  $effect(() => {
    if (!visorPadre || !elegida || !d || pestana !== "actividades") return;
    const codigo = d.actividades[elegida.id]?.codigo ?? "";
    visor?.destruir();
    visor = crearVisor(visorPadre, codigo);
    return () => {
      visor?.destruir();
      visor = null;
    };
  });

  $effect(() => {
    if (!elegida || !d) return;
    const c = d.calificaciones[elegida.id];
    calificacion = c?.calificacion != null ? String(c.calificacion) : "";
    comentario = c?.comentario ?? "";
    pruebas = null;
  });

  const iconos = { verde: "✔", amarillo: "⚠", rojo: "✖" };
  const est = $derived(elegida && d ? (d.estadisticas.por_actividad[elegida.id] ?? contadoresVacios()) : contadoresVacios());

  async function reprobar() {
    if (!elegida?.pruebas || !d) return;
    probando = true;
    try {
      pruebas = await python.probar(d.actividades[elegida.id]?.codigo ?? "", elegida.pruebas);
    } catch (e) {
      avisar(mensajeError(e));
    } finally {
      probando = false;
    }
  }

  async function verReproduccion() {
    if (!elegida || !d) return;
    cargandoReproduccion = true;
    try {
      reproduccion = await (await backend()).reproduccion(d.fila.entrega_id, elegida.id);
    } catch (e) {
      avisar(mensajeError(e));
    } finally {
      cargandoReproduccion = false;
    }
  }

  async function crearAcceso() {
    if (!d) return;
    try {
      const b = await backend();
      const carpeta = await b.elegirCarpeta("¿Dónde guardo el archivo de acceso?");
      if (!carpeta) return;
      acceso = await b.crearAcceso(d.fila.entrega_id, carpeta);
      confirmarAcceso = false;
    } catch (e) {
      avisar(mensajeError(e), 6000);
    }
  }

  async function guardarCalificacion() {
    if (!elegida || !d) return;
    const valor = calificacion.trim() === "" ? null : Number(calificacion);
    if (valor !== null && Number.isNaN(valor)) return avisar("La calificación debe ser un número.");
    await (await backend()).calificar(d.fila.perfil_id, elegida.id, valor, comentario);
    d.calificaciones[elegida.id] = { calificacion: valor, comentario, actualizado: Date.now() };
    avisar("Calificación guardada.");
  }

  function estadoAct(id: string) {
    const a = d?.actividades[id];
    if (!a) return "○";
    return a.completada ? "✓" : "●";
  }
</script>

<div class="detalle">
  <div class="fila cabeza">
    <button class="chico" onclick={() => (app.vista = { tipo: "tablero" })}>← Tablero</button>
    {#if d}
      <div class="quien">
        <h1>{d.fila.nombre}</h1>
        <span class="suave">{d.fila.numero_control} · {Object.values(d.actividades).filter((a) => a.completada).length}/{actividades.length} actividades · entrega del {fecha(d.fila.creado)}</span>
      </div>
      <span class="espaciador"></span>
      <Semaforo nivel={d.reporte.nivel} />
      <button class="chico" onclick={() => (confirmarAcceso = true)} title="Para un alumno que olvidó su contraseña y su código de recuperación">
        🔑 Archivo de acceso
      </button>
      {#if d.historial.length > 1}
        <select class="historial" onchange={(e) => (app.vista = { tipo: "detalle", entregaId: Number((e.target as HTMLSelectElement).value) })}>
          {#each d.historial as h (h.entrega_id)}
            <option value={h.entrega_id} selected={h.entrega_id === d.fila.entrega_id}>{fecha(h.creado)} · {h.nivel}</option>
          {/each}
        </select>
      {/if}
    {/if}
  </div>

  {#if error}
    <p class="error">{error}</p>
  {:else if !d}
    <p class="suave">Cargando…</p>
  {:else}
    <div class="pestanas" role="tablist">
      <button role="tab" class:activa={pestana === "integridad"} onclick={() => (pestana = "integridad")}>🛡 Integridad</button>
      <button role="tab" class:activa={pestana === "actividades"} onclick={() => (pestana = "actividades")}>⌨ Actividades</button>
      <button role="tab" class:activa={pestana === "estadisticas"} onclick={() => (pestana = "estadisticas")}>📊 Estadísticas</button>
    </div>

    {#if pestana === "integridad"}
      <section class="integridad">
        {#if d.fila.alerta_identidad}
          <p class="aviso">⚠ Este perfil aparece con otro número de control (o este número con otro perfil) en otras entregas.</p>
        {/if}
        <ul class="checks">
          {#each d.reporte.checks as c (c.id)}
            <li class={c.nivel}>
              <span class="icono">{iconos[c.nivel]}</span>
              <div><strong>{c.nombre}</strong><p>{c.detalle}</p></div>
            </li>
          {/each}
        </ul>
        <div class="tarjeta ritmo">
          <h3>Ritmo de escritura</h3>
          <p class="suave">
            Caracteres tecleados: <strong>{d.reporte.ritmo.tecleados}</strong> · automáticos (sangría, paréntesis):
            {d.reporte.ritmo.automaticos} · deshacer/rehacer: {d.reporte.ritmo.deshacer_rehacer} · pegados permitidos:
            {d.reporte.ritmo.pegados}
          </p>
          <p class="suave">
            Máximo sostenido: <strong>{d.reporte.ritmo.max_cps.toFixed(1)}</strong> caracteres/s (un principiante escribe 2–6).
            Ráfagas sospechosas: <strong class:alerta={d.reporte.ritmo.rafagas > 0}>{d.reporte.ritmo.rafagas}</strong>
          </p>
        </div>
        <p class="suave nota">
          La app no puede impedir al 100 % que alguien manipule un archivo sin conexión, pero cada entrega se reconstruye
          tecla a tecla desde su historial firmado: si el código no sale de lo escrito en la app, aquí aparece en rojo.
        </p>
      </section>
    {:else if pestana === "actividades"}
      <section class="actividades">
        <nav class="lista">
          {#each curso.unidades as u (u.id)}
            <h4>U{u.numero} · {u.titulo}</h4>
            {#each u.lecciones.flatMap((l) => l.actividades) as a (a.id)}
              {@const c = d.estadisticas.por_actividad[a.id]}
              <button class:activa={elegida?.id === a.id} onclick={() => (elegida = a)} disabled={!d.actividades[a.id]}>
                <span class="marca" class:hecha={d.actividades[a.id]?.completada}>{estadoAct(a.id)}</span>
                <span class="t">{a.titulo}</span>
                {#if c?.pegados_intentos}<span class="insignia alerta" title="Intentos de pegar">🚫{c.pegados_intentos}</span>{/if}
                {#if d.calificaciones[a.id]?.calificacion != null}<span class="insignia">{d.calificaciones[a.id].calificacion}</span>{/if}
              </button>
            {/each}
          {/each}
        </nav>
        {#if elegida}
          {@const e = d.actividades[elegida.id]}
          <div class="panel">
            <h2>{elegida.titulo}</h2>
            <div class="fila chips">
              <span class="insignia">{e?.completada ? "✓ Completada" : e?.total ? `Pruebas ${e.pasadas}/${e.total}` : "En progreso"}</span>
              <span class="insignia">⏱ {minutos(est.tiempo_ms)}</span>
              <span class="insignia">▶ {est.ejecuciones} ejecuciones ({est.errores} con error)</span>
              <span class="insignia">✔ {est.pruebas} pruebas</span>
              <span class="insignia">💡 {e?.pistas ?? 0} pistas</span>
              <span class="insignia">📋 {est.copias} copias</span>
              <span class="insignia" class:alerta={est.pegados_intentos > 0}>🚫 {est.pegados_intentos} intentos de pegar</span>
              <span class="insignia">↗ {est.salidas} salidas</span>
              {#if d.reporte.ritmo_por_actividad[elegida.id]?.rafagas}
                <span class="insignia alerta">⚡ {d.reporte.ritmo_por_actividad[elegida.id].rafagas} ráfagas</span>
              {/if}
            </div>
            {#if elegida.tipo === "codigo"}
              <div class="visor" bind:this={visorPadre}></div>
              <div class="fila">
                <button onclick={verReproduccion} disabled={cargandoReproduccion} data-ver-reproduccion>
                  {cargandoReproduccion ? "Cargando…" : "⏯ Ver cómo lo escribió"}
                </button>
                <button onclick={reprobar} disabled={probando}>{probando ? "Probando…" : "✔ Volver a correr las pruebas"}</button>
                {#if pruebas}
                  <span class:alerta={pruebas.pasadas !== pruebas.total}>Pasaron {pruebas.pasadas} de {pruebas.total}</span>
                {/if}
              </div>
              {#if pruebas}
                <ul class="pruebas">
                  {#each pruebas.resultados as r (r.indice)}
                    <li class:mal={!r.paso}>{r.paso ? "✔" : "✖"} {r.nombre}{r.paso ? "" : ` — ${r.mensaje}`}</li>
                  {/each}
                </ul>
              {/if}
            {:else}
              <div class="tarjeta">
                <p class="suave">Respuesta del alumno:</p>
                <pre class="respuesta">{e?.respuesta ?? "(sin responder)"}</pre>
                {#if elegida.tipo === "prediccion"}
                  <p class="suave">Salida correcta:</p>
                  <pre class="respuesta">{elegida.salida_esperada}</pre>
                {/if}
              </div>
            {/if}
            <details class="enunciado">
              <summary>Ver enunciado{elegida.solucion ? " y solución de referencia" : ""}</summary>
              <Markdown contenido={elegida.enunciado} />
              {#if elegida.solucion}<Markdown contenido={"```python\n" + elegida.solucion + "```"} />{/if}
            </details>
            <div class="tarjeta calificar">
              <div class="fila">
                <label for="cal">Calificación</label>
                <input id="cal" bind:value={calificacion} inputmode="decimal" placeholder="—" />
                <button class="primario" onclick={guardarCalificacion}>Guardar</button>
              </div>
              <textarea rows="2" bind:value={comentario} placeholder="Comentario para el alumno (opcional)"></textarea>
            </div>
          </div>
        {/if}
      </section>
    {:else}
      {@const g = d.estadisticas.global}
      <section class="tarjetas">
        {#each [["Tiempo de práctica", minutos(g.tiempo_ms)], ["Ejecuciones", g.ejecuciones], ["Con error", g.errores], ["Pruebas", g.pruebas], ["Pistas", g.pistas], ["Teclas", g.teclas], ["Copias", g.copias], ["Intentos de pegar", g.pegados_intentos], ["Inserciones sospechosas", g.inserciones_sospechosas], ["Salidas de ventana", g.salidas], ["Tiempo fuera", minutos(g.tiempo_fuera_ms)], ["Ejecutables creados", g.ejecutables]] as [t, v] (t)}
          <div class="tarjeta dato"><span class="valor">{v}</span><span class="suave">{t}</span></div>
        {/each}
      </section>
      <p class="suave">
        Versión de la App Alumno: {d.manifiesto.app.version}{d.manifiesto.app.dev ? " (compilación de desarrollo)" : ""} ·
        dispositivos: {Object.keys(d.manifiesto.cabezas).length}
      </p>
    {/if}
  {/if}
</div>

<Modal titulo="Archivo de acceso" abierto={confirmarAcceso} cerrar={() => (confirmarAcceso = false)} ancho="560px">
  <p>
    Si {d?.fila.nombre ?? "el alumno"} olvidó su contraseña <strong>y</strong> su código de recuperación, este archivo le permite
    entrar y elegir una contraseña nueva (en la App Alumno: "Tengo un archivo de acceso de mi profesor").
  </p>
  <p class="suave">
    Se crea con su entrega más reciente y solo sirve con la contraseña temporal que verás a continuación. Dáselos por
    separado y en persona.
  </p>
  {#snippet acciones()}
    <button onclick={() => (confirmarAcceso = false)}>Cancelar</button>
    <button class="primario" onclick={crearAcceso}>Crear archivo</button>
  {/snippet}
</Modal>

<Modal titulo="Archivo de acceso creado" abierto={acceso !== null} cerrar={() => (acceso = null)} ancho="560px">
  {#if acceso}
    <p>Guardado en:</p>
    <p class="ruta">{acceso.ruta}</p>
    <p>Contraseña temporal para {d?.fila.nombre}:</p>
    <p class="temporal" data-temporal>{acceso.temporal}</p>
    <p class="suave">Anótala ahora: no se vuelve a mostrar. Al entrar, el alumno elegirá una contraseña nueva y recibirá un código de recuperación nuevo.</p>
  {/if}
  {#snippet acciones()}
    <button class="primario" onclick={() => (acceso = null)}>Listo</button>
  {/snippet}
</Modal>

<Modal titulo={elegida ? `Cómo escribió: ${elegida.titulo}` : "Reproducción"} abierto={reproduccion !== null} cerrar={() => (reproduccion = null)} ancho="1000px">
  {#if reproduccion}
    {#key reproduccion}
      <Reproductor linea={reproduccion} />
    {/key}
  {/if}
</Modal>

<style>
  .ruta {
    font-family: var(--fuente-codigo);
    word-break: break-all;
    background: var(--superficie-2);
    padding: 0.5em;
    border-radius: var(--radio-chico);
  }
  .temporal {
    font-family: var(--fuente-codigo);
    font-size: 1.6rem;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-align: center;
    padding: 0.4em;
    border: 2px dashed var(--primario);
    border-radius: var(--radio);
  }
  .detalle {
    padding: 1rem 1.2rem 2rem;
  }
  .cabeza {
    gap: 1rem;
    margin-bottom: 1rem;
  }
  .quien h1 {
    font-size: 1.35rem;
    margin: 0;
  }
  .historial {
    width: auto;
  }
  .pestanas {
    display: flex;
    gap: 0.3rem;
    border-bottom: 1px solid var(--borde);
    margin-bottom: 1rem;
  }
  .pestanas button {
    border: none;
    border-radius: 0;
    border-bottom: 3px solid transparent;
    background: transparent;
    color: var(--texto-suave);
  }
  .pestanas button.activa {
    border-bottom-color: var(--primario);
    color: var(--primario);
    font-weight: 600;
  }
  .checks {
    list-style: none;
    padding: 0;
    margin: 0 0 1rem;
    display: grid;
    gap: 0.5rem;
    max-width: 900px;
  }
  .checks li {
    display: flex;
    gap: 0.8rem;
    align-items: flex-start;
    background: var(--superficie);
    border: 1px solid var(--borde);
    border-left-width: 4px;
    border-radius: var(--radio-chico);
    padding: 0.6rem 0.9rem;
  }
  .checks li p {
    margin: 0.15em 0 0;
    color: var(--texto-suave);
  }
  .checks li.verde {
    border-left-color: var(--exito);
  }
  .checks li.amarillo {
    border-left-color: var(--aviso);
  }
  .checks li.rojo {
    border-left-color: var(--peligro);
    background: var(--peligro-suave);
  }
  .icono {
    font-weight: 700;
  }
  .verde .icono {
    color: var(--exito);
  }
  .amarillo .icono {
    color: var(--aviso);
  }
  .rojo .icono {
    color: var(--peligro);
  }
  .ritmo {
    max-width: 900px;
  }
  .ritmo h3 {
    margin: 0 0 0.3rem;
    font-size: 1rem;
  }
  .nota {
    max-width: 900px;
    font-size: 0.9em;
  }
  .actividades {
    display: grid;
    grid-template-columns: 300px 1fr;
    gap: 1rem;
    align-items: start;
  }
  .lista {
    display: flex;
    flex-direction: column;
    background: var(--superficie);
    border: 1px solid var(--borde);
    border-radius: var(--radio);
    padding: 0.5rem;
    max-height: calc(100vh - 230px);
    overflow: auto;
    position: sticky;
    top: 0;
  }
  .lista h4 {
    margin: 0.6rem 0.4rem 0.2rem;
    font-size: 0.85em;
    color: var(--texto-suave);
  }
  .lista button {
    border: none;
    background: transparent;
    justify-content: flex-start;
    text-align: left;
    white-space: normal;
    font-size: 0.9em;
    padding: 0.3em 0.5em;
  }
  .lista button.activa {
    background: var(--primario-suave);
  }
  .t {
    flex: 1;
  }
  .marca {
    width: 1em;
    color: var(--aviso);
  }
  .marca.hecha {
    color: var(--exito);
  }
  .panel h2 {
    margin: 0 0 0.5rem;
  }
  .chips {
    margin-bottom: 0.8rem;
  }
  .insignia.alerta {
    background: var(--aviso-suave);
    color: var(--aviso);
  }
  .visor {
    height: 340px;
    border: 1px solid var(--borde);
    border-radius: var(--radio-chico);
    overflow: hidden;
    margin-bottom: 0.6rem;
  }
  .visor :global(.cm-editor) {
    height: 100%;
  }
  .pruebas {
    padding-left: 1.2em;
  }
  .pruebas .mal {
    color: var(--peligro);
  }
  .respuesta {
    background: var(--superficie-2);
    padding: 0.5em 0.8em;
    border-radius: var(--radio-chico);
    font-family: var(--fuente-codigo);
    white-space: pre-wrap;
  }
  .enunciado {
    margin: 0.8rem 0;
  }
  .enunciado summary {
    cursor: pointer;
    color: var(--primario);
  }
  .calificar {
    display: grid;
    gap: 0.5rem;
    max-width: 640px;
  }
  .calificar label {
    margin: 0;
  }
  .calificar input {
    width: 100px;
  }
  .tarjetas {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(170px, 1fr));
    gap: 0.8rem;
    margin-bottom: 1rem;
  }
  .dato {
    display: flex;
    flex-direction: column;
  }
  .valor {
    font-size: 1.4rem;
    font-weight: 700;
  }
</style>
