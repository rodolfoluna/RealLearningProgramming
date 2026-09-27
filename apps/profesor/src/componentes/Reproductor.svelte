<script lang="ts">
  // Vuelve a escribir, tecla a tecla, el código de una actividad a partir del historial firmado.
  import { crearVisor, type EditorCodigo } from "@rlp/editor";
  import { onDestroy, onMount } from "svelte";
  import type { LineaDeTiempo, Marca } from "../lib/tipos";

  let { linea }: { linea: LineaDeTiempo } = $props();

  interface Paso {
    t: number;
    tramo: number;
    /** Paso "base": el tramo empieza desde este texto. */
    texto?: string;
    op?: { desde: number; hasta: number; insertado: string; origen: string };
  }

  /** Pausas más largas que esto se acortan con "saltar pausas". */
  const PAUSA_MAX_MS = 1200;
  const VELOCIDADES = [1, 5, 20, 100];

  const ETIQUETAS: Record<string, string> = {
    pegado: "Intento de pegar",
    insercion_sospechosa: "Inserción sospechosa",
    copia: "Copió texto",
    foco: "Salió de la ventana",
    ejecucion: "Ejecutó el programa",
    prueba: "Corrió las pruebas",
    pista: "Vio una pista",
    reinicio: "Reinició el código",
    ejecutable: "Creó un .exe",
  };

  // svelte-ignore state_referenced_locally (el componente se crea de nuevo para cada línea de tiempo)
  const pasos: Paso[] = construirPasos(linea);
  const virtual = tiemposVirtuales(true);
  let saltarPausas = $state(true);
  const tiempos = $derived(saltarPausas ? virtual : tiemposVirtuales(false));
  const duracion = $derived(tiempos.length ? tiempos[tiempos.length - 1] : 0);

  let padre: HTMLDivElement | undefined = $state();
  let visor: EditorCodigo | null = null;
  /** Cuántos pasos se han aplicado. */
  let aplicados = $state(0);
  /** Posición en la línea de tiempo (ms virtuales). */
  let posicion = $state(0);
  let velocidad = $state(5);
  let reproduciendo = $state(false);
  let error = $state("");
  let cuadro = 0;
  let ultimoCuadro = 0;

  const actual = $derived(aplicados > 0 ? pasos[aplicados - 1] : null);
  const tramoActual = $derived(actual ? linea.tramos[actual.tramo] : null);
  const dispositivos = $derived([...new Set(linea.tramos.map((t) => t.dispositivo))]);
  const terminado = $derived(aplicados === pasos.length && pasos.length > 0);
  const coincide = $derived(terminado && visor !== null && textoVisor() === linea.codigo_final);
  const marcasVisibles = $derived(
    linea.marcas.map((m) => ({ m, v: tiempoDeMarca(m.t) })).filter((x) => !(x.m.tipo === "foco" && x.m.datos?.estado !== "perdido")),
  );

  function construirPasos(l: LineaDeTiempo): Paso[] {
    const lista: Paso[] = [];
    l.tramos.forEach((tr, i) => {
      if (tr.texto_inicial === null) return;
      lista.push({ t: tr.t_inicio, tramo: i, texto: tr.texto_inicial });
      for (const [t, desde, hasta, insertado, origen] of tr.ops) lista.push({ t, tramo: i, op: { desde, hasta, insertado, origen } });
    });
    return lista;
  }

  function tiemposVirtuales(saltar: boolean): number[] {
    const v: number[] = [];
    pasos.forEach((p, i) => {
      if (i === 0) return v.push(0);
      const dt = Math.max(0, p.t - pasos[i - 1].t);
      v.push(v[i - 1] + (saltar || p.texto !== undefined ? Math.min(dt, PAUSA_MAX_MS) : dt));
    });
    return v;
  }

  /** Posición virtual de algo que pasó en el momento real `t`. */
  function tiempoDeMarca(t: number): number {
    let i = pasos.findIndex((p) => p.t > t);
    if (i === -1) i = pasos.length;
    if (i === 0) return 0;
    return Math.min(tiempos[i - 1] + Math.min(t - pasos[i - 1].t, PAUSA_MAX_MS), i < pasos.length ? tiempos[i] : duracion);
  }

  function textoVisor(): string {
    return visor?.texto ?? "";
  }

  function aplicarPaso(p: Paso) {
    if (!visor) return;
    if (p.texto !== undefined) {
      visor.establecerTexto(p.texto);
      return;
    }
    const { desde, hasta, insertado } = p.op!;
    const largo = visor.view.state.doc.length;
    if (hasta > largo || desde > hasta) throw new Error("El historial no coincide con el texto (operación fuera de rango).");
    visor.view.dispatch({
      changes: { from: desde, to: hasta, insert: insertado },
      selection: { anchor: desde + insertado.length },
      scrollIntoView: true,
    });
  }

  /** Reconstruye el texto hasta `n` pasos aplicando las operaciones como cadenas (para saltar). */
  function irAPaso(n: number) {
    if (!visor) return;
    let inicio = n - 1;
    while (inicio >= 0 && pasos[inicio].texto === undefined) inicio--;
    let texto = inicio >= 0 ? pasos[inicio].texto! : "";
    let cursor = 0;
    try {
      for (let i = inicio + 1; i < n; i++) {
        const { desde, hasta, insertado } = pasos[i].op!;
        if (hasta > texto.length || desde > hasta) throw new Error("El historial no coincide con el texto (operación fuera de rango).");
        texto = texto.slice(0, desde) + insertado + texto.slice(hasta);
        cursor = desde + insertado.length;
      }
      error = "";
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
    visor.establecerTexto(texto);
    if (cursor) visor.view.dispatch({ selection: { anchor: Math.min(cursor, texto.length) }, scrollIntoView: true });
    aplicados = n;
  }

  function buscarPaso(v: number): number {
    // Cantidad de pasos cuyo tiempo virtual es <= v.
    let bajo = 0;
    let alto = tiempos.length;
    while (bajo < alto) {
      const medio = (bajo + alto) >> 1;
      if (tiempos[medio] <= v) bajo = medio + 1;
      else alto = medio;
    }
    return bajo;
  }

  function saltarA(v: number) {
    posicion = Math.max(0, Math.min(v, duracion));
    irAPaso(buscarPaso(posicion));
  }

  function avanzar(ahora: number) {
    if (!reproduciendo) return;
    const dt = ultimoCuadro ? ahora - ultimoCuadro : 0;
    ultimoCuadro = ahora;
    posicion = Math.min(duracion, posicion + dt * velocidad);
    try {
      while (aplicados < pasos.length && tiempos[aplicados] <= posicion) {
        aplicarPaso(pasos[aplicados]);
        aplicados++;
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      reproduciendo = false;
      return;
    }
    if (aplicados >= pasos.length) {
      reproduciendo = false;
      return;
    }
    cuadro = requestAnimationFrame(avanzar);
  }

  function alternar() {
    if (reproduciendo) {
      reproduciendo = false;
      cancelAnimationFrame(cuadro);
      return;
    }
    if (terminado) saltarA(0);
    reproduciendo = true;
    ultimoCuadro = 0;
    cuadro = requestAnimationFrame(avanzar);
  }

  function cambiarPausas() {
    // Mantiene el punto actual al cambiar la escala de tiempo.
    saltarPausas = !saltarPausas;
    posicion = aplicados > 0 ? tiempos[aplicados - 1] : 0;
  }

  function hora(t: number): string {
    return new Date(t).toLocaleString("es-MX", { dateStyle: "short", timeStyle: "medium" });
  }

  function duracionTexto(ms: number): string {
    const s = Math.round(ms / 1000);
    const h = Math.floor(s / 3600);
    const m = Math.floor((s % 3600) / 60);
    const seg = s % 60;
    return h ? `${h} h ${m} min` : m ? `${m} min ${seg} s` : `${seg} s`;
  }

  function detalleMarca(m: Marca): string {
    const d = m.datos ?? {};
    if (m.tipo === "prueba") return ` (${d.pasadas ?? "?"}/${d.total ?? "?"})`;
    if (m.tipo === "ejecucion" && d.estado === "error") return " (con error)";
    if (m.tipo === "pegado" && d.permitido) return " (permitido)";
    return "";
  }

  onMount(() => {
    if (padre) visor = crearVisor(padre, "");
    if (pasos.length) irAPaso(1);
  });

  onDestroy(() => {
    cancelAnimationFrame(cuadro);
    visor?.destruir();
  });
</script>

<div class="reproductor">
  {#if !pasos.length}
    <p class="suave">No hay historial de escritura para esta actividad.</p>
  {/if}
  {#if linea.avisos.length}
    <p class="aviso">⚠ Parte del historial no se pudo reconstruir: {linea.avisos.join("; ")}</p>
  {/if}

  <div class="visor" bind:this={padre} data-reproductor-visor></div>

  <div class="controles fila">
    <button class="primario" onclick={alternar} disabled={!pasos.length} data-reproducir>
      {reproduciendo ? "⏸ Pausa" : terminado ? "↺ Repetir" : "▶ Reproducir"}
    </button>
    <label class="fila velocidad">
      Velocidad
      <select bind:value={velocidad}>
        {#each VELOCIDADES as v (v)}<option value={v}>{v}×</option>{/each}
      </select>
    </label>
    <label class="fila interruptor"><input type="checkbox" checked={saltarPausas} onchange={cambiarPausas} /> Saltar pausas</label>
    <span class="espaciador"></span>
    <span class="suave chico">{duracionTexto(posicion)} / {duracionTexto(duracion)}</span>
  </div>

  <div class="linea">
    <input
      type="range"
      min="0"
      max={Math.max(duracion, 1)}
      step="1"
      value={posicion}
      oninput={(e) => saltarA(Number((e.target as HTMLInputElement).value))}
      aria-label="Posición en el historial"
    />
    <div class="marcas" aria-hidden="true">
      {#each marcasVisibles as { m, v }, i (i)}
        <button
          class="marca {m.tipo}"
          class:mal={m.tipo === "prueba" && m.datos?.pasadas !== m.datos?.total}
          style:left="{duracion ? (v / duracion) * 100 : 0}%"
          title="{ETIQUETAS[m.tipo] ?? m.tipo}{detalleMarca(m)} · {hora(m.t)}"
          tabindex="-1"
          onclick={() => saltarA(v)}
        ></button>
      {/each}
    </div>
  </div>

  <div class="estado fila">
    {#if actual}
      <span class="suave chico">{hora(actual.t)}</span>
      {#if tramoActual}
        <span class="insignia">
          {dispositivos.length > 1 ? `Equipo ${dispositivos.indexOf(tramoActual.dispositivo) + 1} de ${dispositivos.length}` : "Un equipo"}
          {tramoActual.motivo === "reinicio" ? " · reinició el código" : tramoActual.motivo === "continuacion" ? " · continuó" : ""}
        </span>
      {/if}
      {#if actual.op?.origen === "p"}<span class="insignia alerta">Pegado permitido</span>{/if}
      {#if actual.op?.origen === "u" || actual.op?.origen === "r"}<span class="insignia">Deshacer/rehacer</span>{/if}
    {/if}
    <span class="espaciador"></span>
    {#if terminado}
      {#if coincide}
        <span class="exito-msg" data-coincide>✔ El resultado coincide con el código entregado</span>
      {:else}
        <span class="alerta">⚠ El resultado no coincide con el código entregado</span>
      {/if}
    {/if}
  </div>
  {#if error}<p class="error">{error}</p>{/if}

  <div class="leyenda suave chico">
    {#each Object.entries(ETIQUETAS) as [tipo, texto] (tipo)}
      {#if linea.marcas.some((m) => m.tipo === tipo)}<span><i class="punto {tipo}"></i>{texto}</span>{/if}
    {/each}
  </div>
</div>

<style>
  .reproductor {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }
  .visor {
    height: 360px;
    border: 1px solid var(--borde);
    border-radius: var(--radio-chico);
    overflow: hidden;
  }
  .visor :global(.cm-editor) {
    height: 100%;
  }
  .controles {
    gap: 0.8rem;
    align-items: center;
  }
  .velocidad,
  .interruptor {
    gap: 0.4rem;
    margin: 0;
    font-weight: normal;
    align-items: center;
  }
  .velocidad select {
    width: auto;
  }
  .interruptor input {
    width: auto;
  }
  .linea {
    position: relative;
    padding-top: 12px;
  }
  .linea input {
    width: 100%;
    margin: 0;
  }
  .marcas {
    position: absolute;
    top: 0;
    left: 8px;
    right: 8px;
    height: 12px;
  }
  .marca {
    position: absolute;
    top: 0;
    width: 4px;
    height: 12px;
    padding: 0;
    min-height: 0;
    border: none;
    border-radius: 2px;
    transform: translateX(-2px);
    background: var(--texto-suave);
    cursor: pointer;
  }
  .marca.pegado,
  .marca.insercion_sospechosa,
  .punto.pegado,
  .punto.insercion_sospechosa {
    background: var(--peligro);
  }
  .marca.copia,
  .punto.copia {
    background: var(--primario);
  }
  .marca.foco,
  .punto.foco,
  .marca.reinicio,
  .punto.reinicio {
    background: var(--aviso);
  }
  .marca.prueba,
  .punto.prueba {
    background: var(--exito);
  }
  .marca.prueba.mal {
    background: var(--aviso);
  }
  .marca.pista,
  .punto.pista {
    background: var(--acento);
  }
  .estado {
    gap: 0.6rem;
    align-items: center;
    min-height: 1.6em;
  }
  .leyenda {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem 1rem;
  }
  .punto {
    display: inline-block;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    margin-right: 0.35em;
    background: var(--texto-suave);
  }
  .chico {
    font-size: 0.85em;
  }
  .alerta {
    color: var(--aviso);
  }
  .exito-msg {
    color: var(--exito);
  }
</style>
