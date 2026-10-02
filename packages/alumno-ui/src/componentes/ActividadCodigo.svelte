<script lang="ts">
  import type { Actividad } from "@rlp/curso";
  import { EditorCodigo, type AccionTecla, type LoteOperaciones } from "@rlp/editor";
  import type { ResultadoPrueba, ResultadoPruebas } from "@rlp/python-worker";
  import { Markdown, Modal, mensajeError } from "@rlp/ui-comun";
  import { onDestroy, onMount } from "svelte";
  import { backend } from "../lib/backend";
  import { enDialogoNativo } from "../lib/dialogos";
  import {
    actualizarActividad,
    alCerrar,
    app,
    avisar,
    esTactil,
    politicaPegado,
    python,
    refrescarEstadisticas,
    registrarEvento,
    registrarSalidas,
  } from "../lib/app.svelte";
  import Consola from "./Consola.svelte";
  import Pistas from "./Pistas.svelte";
  import Resultados from "./Resultados.svelte";

  let { actividad }: { actividad: Actividad } = $props();

  let contenedor: HTMLDivElement | undefined = $state();
  let consola: Consola | undefined = $state();
  let editor: EditorCodigo | null = null;
  let cargando = $state(true);
  let errorCarga = $state("");
  let corriendo = $state(false);
  let probando = $state(false);
  let pestana: "consola" | "pruebas" = $state("consola");
  let panelMovil: "enunciado" | "codigo" | "salida" = $state("codigo");
  let resultado: ResultadoPruebas | null = $state(null);
  let enCurso: ResultadoPrueba[] = $state([]);
  let fuente = $state(Number(leer("rlp-fuente") ?? 15));
  let confirmarReinicio = $state(false);

  // Generar ejecutable
  let modalExe = $state(false);
  // svelte-ignore state_referenced_locally (el componente se recrea al cambiar de actividad)
  let nombreExe = $state(actividad.id.replace(/^u\d+-/, "").replace(/-/g, "_"));
  let progresoExe: string[] = $state([]);
  let rutaExe = $state("");
  let errorExe = $state("");
  let generando = $state(false);

  // svelte-ignore state_referenced_locally (el componente se recrea al cambiar de actividad)
  const id = actividad.id;
  let colaGuardado: Promise<unknown> = Promise.resolve();
  let temporizadorSintaxis: ReturnType<typeof setTimeout> | null = null;
  let salidaDesde = 0;

  // Barra de teclas para pantallas táctiles: símbolos difíciles de alcanzar en el teclado del
  // celular. Se insertan como tecleo normal (cuentan en el historial).
  const tactil = esTactil();
  const TECLAS: { etiqueta: string; texto?: string; accion?: AccionTecla; titulo?: string }[] = [
    { etiqueta: "⇥", accion: "tab", titulo: "Sangría" },
    ...[":", "(", ")", "[", "]", "{", "}", '"', "'", "=", "<", ">", "+", "-", "*", "/", "%", "#", "_", ",", "."].map((t) => ({ etiqueta: t, texto: t })),
    { etiqueta: "←", accion: "izquierda", titulo: "Izquierda" },
    { etiqueta: "→", accion: "derecha", titulo: "Derecha" },
    { etiqueta: "↑", accion: "arriba", titulo: "Arriba" },
    { etiqueta: "↓", accion: "abajo", titulo: "Abajo" },
    { etiqueta: "↶", accion: "deshacer", titulo: "Deshacer" },
  ];

  function tecla(t: (typeof TECLAS)[number]) {
    if (!editor) return;
    if (t.texto) editor.teclear(t.texto);
    else if (t.accion) editor.accion(t.accion);
  }

  /** En el celular, cambiar de app no siempre quita el foco a la ventana: cuenta como salida. */
  function cambioDeVisibilidad() {
    guardarPendiente();
    if (document.hidden) alSalir();
    else alVolver();
  }

  function leer(clave: string) {
    try {
      return localStorage.getItem(clave);
    } catch {
      return null;
    }
  }

  function guardar(lote: LoteOperaciones) {
    const texto = editor?.texto ?? "";
    colaGuardado = colaGuardado.then(async () => {
      try {
        const r = await (await backend()).guardarEdicion(id, lote, texto);
        if (!r.ok && editor) {
          editor.establecerTexto(r.codigo);
          avisar("Se restauró tu código desde el historial guardado.");
        }
        const e = app.alumno?.actividades[id];
        if (e) e.codigo = r.codigo;
      } catch (err) {
        avisar(`No se pudo guardar: ${mensajeError(err)}`, 6000);
      }
    });
  }

  function revisarSintaxis(texto: string) {
    if (temporizadorSintaxis) clearTimeout(temporizadorSintaxis);
    temporizadorSintaxis = setTimeout(async () => {
      if (!editor || python.ocupado) return;
      const err = await python.sintaxis(texto).catch(() => null);
      if (!editor || editor.texto !== texto) return;
      if (err) editor.marcarError(err.linea, err.columna, `${err.explicacion} (${err.tipo}: ${err.mensaje})`);
      else editor.limpiarErrores();
    }, 700);
  }

  onMount(async () => {
    try {
      const estado = await (await backend()).abrirActividad(id, actividad.codigo_inicial ?? "");
      actualizarActividad(id, estado);
      editor = new EditorCodigo({
        padre: contenedor!,
        texto: estado.codigo,
        politica: politicaPegado,
        alOperaciones: guardar,
        alCambiar: revisarSintaxis,
        alPegar: (e) => {
          void registrarEvento("pegado", id, { chars: e.chars, via: e.via, interno: e.interno, permitido: e.permitido });
          if (!e.permitido) avisar("🚫 Pegar está deshabilitado: escribe el código tú mismo. Tu profesor verá este intento.");
        },
        alCopiar: (e) => void registrarEvento("copia", id, { chars: e.chars, origen: "editor", cortar: e.cortar }),
        alSospechar: (e) => {
          void registrarEvento("insercion_sospechosa", id, { chars: e.chars, userEvent: e.userEvent });
        },
        atajos: [
          { tecla: "F5", accion: ejecutar },
          { tecla: "F6", accion: probar },
          { tecla: "Mod-s", accion: () => editor?.vaciarOperaciones() },
        ],
      });
      editor.enfocar();
      revisarSintaxis(estado.codigo);
    } catch (e) {
      errorCarga = mensajeError(e);
    } finally {
      cargando = false;
    }
    window.addEventListener("blur", alSalir);
    window.addEventListener("focus", alVolver);
    window.addEventListener("keydown", teclas);
    window.addEventListener("pagehide", guardarPendiente);
    document.addEventListener("visibilitychange", cambioDeVisibilidad);
  });

  function guardarPendiente() {
    editor?.vaciarOperaciones();
  }

  async function antesDeCerrar() {
    editor?.vaciarOperaciones();
    await colaGuardado;
  }
  alCerrar.add(antesDeCerrar);

  onDestroy(() => {
    window.removeEventListener("blur", alSalir);
    window.removeEventListener("focus", alVolver);
    window.removeEventListener("keydown", teclas);
    window.removeEventListener("pagehide", guardarPendiente);
    alCerrar.delete(antesDeCerrar);
    document.removeEventListener("visibilitychange", cambioDeVisibilidad);
    if (temporizadorSintaxis) clearTimeout(temporizadorSintaxis);
    if (python.ocupado) python.detener();
    editor?.destruir();
    editor = null;
  });

  function alSalir() {
    editor?.vaciarOperaciones(); // guarda lo tecleado antes de perder el foco (o cerrar la ventana)
    if (!registrarSalidas() || enDialogoNativo() || salidaDesde) return;
    salidaDesde = Date.now();
    void registrarEvento("foco", id, { estado: "perdido" });
  }

  function alVolver() {
    if (!salidaDesde) return;
    const fuera_ms = Date.now() - salidaDesde;
    salidaDesde = 0;
    void registrarEvento("foco", id, { estado: "recuperado", fuera_ms });
  }

  function teclas(e: KeyboardEvent) {
    if (e.key === "F5") {
      e.preventDefault();
      ejecutar();
    } else if (e.key === "F6") {
      e.preventDefault();
      probar();
    } else if (e.key === "Escape" && (corriendo || probando)) {
      detener();
    }
  }

  async function ejecutar() {
    if (!editor || corriendo || probando) return;
    editor.vaciarOperaciones();
    pestana = "consola";
    panelMovil = "salida";
    corriendo = true;
    editor.limpiarErrores();
    try {
      const r = await consola!.ejecutar(editor.texto);
      if (r.error) editor?.marcarError(r.error.linea, r.error.columna, r.error.explicacion);
      void registrarEvento("ejecucion", id, { estado: r.estado, error: r.error?.tipo ?? null, ms: r.duracion_ms });
    } catch (e) {
      avisar(mensajeError(e), 6000);
    } finally {
      corriendo = false;
    }
  }

  async function probar() {
    if (!editor || corriendo || probando || !actividad.pruebas) return;
    editor.vaciarOperaciones();
    pestana = "pruebas";
    panelMovil = "salida";
    probando = true;
    resultado = null;
    enCurso = [];
    try {
      await python.iniciar();
      const r = await python.probar(editor.texto, actividad.pruebas, (p) => (enCurso = [...enCurso, p]));
      resultado = r;
      await colaGuardado;
      const antes = app.alumno?.actividades[id]?.completada;
      const estado = await (await backend()).registrarPruebas(id, r.pasadas, r.total, actividad.puntos);
      actualizarActividad(id, estado);
      refrescarEstadisticas();
      if (estado.completada && !antes) avisar(`🎉 ¡Actividad completada! +${actividad.puntos} puntos`, 5000);
      const primerError = r.resultados.find((x) => x.error?.linea)?.error;
      if (primerError) editor?.marcarError(primerError.linea, primerError.columna, primerError.explicacion);
    } catch (e) {
      avisar(mensajeError(e), 6000);
    } finally {
      probando = false;
    }
  }

  function detener() {
    python.detener();
  }

  async function reiniciar() {
    confirmarReinicio = false;
    await colaGuardado;
    editor?.vaciarOperaciones();
    await colaGuardado;
    const estado = await (await backend()).reiniciarActividad(id, actividad.codigo_inicial ?? "");
    actualizarActividad(id, estado);
    editor?.establecerTexto(estado.codigo);
    resultado = null;
    consola?.limpiar();
  }

  function cambiarFuente(delta: number) {
    fuente = Math.min(26, Math.max(11, fuente + delta));
    try {
      localStorage.setItem("rlp-fuente", String(fuente));
    } catch {
      /* sin almacenamiento */
    }
  }

  async function generarExe() {
    if (!editor) return;
    editor.vaciarOperaciones();
    errorExe = "";
    rutaExe = "";
    progresoExe = [];
    generando = true;
    try {
      const err = await python.sintaxis(editor.texto);
      if (err) throw new Error(`Corrige primero el error de la línea ${err.linea}: ${err.explicacion}`);
      rutaExe = await (await backend()).generarEjecutable(nombreExe, editor.texto, (l) => (progresoExe = [...progresoExe.slice(-200), l]));
      refrescarEstadisticas();
    } catch (e) {
      errorExe = mensajeError(e);
    } finally {
      generando = false;
    }
  }

  async function abrirCarpetaExe() {
    const carpeta = rutaExe.replace(/[\\/][^\\/]+$/, "");
    await (await backend()).abrirCarpeta(carpeta);
  }
</script>

<div class="espacio" data-panel={panelMovil}>
  <nav class="pestanas-movil" aria-label="Paneles">
    <button class:activa={panelMovil === "enunciado"} onclick={() => (panelMovil = "enunciado")}>Enunciado</button>
    <button class:activa={panelMovil === "codigo"} onclick={() => (panelMovil = "codigo")}>Código</button>
    <button class:activa={panelMovil === "salida"} onclick={() => (panelMovil = "salida")}>Consola y pruebas</button>
  </nav>

  <section class="enunciado">
    <Markdown
      contenido={actividad.enunciado}
      alCopiar={(chars) => registrarEvento("copia", id, { chars, origen: "enunciado" })}
    />
    <Pistas {id} pistas={actividad.pistas} />
    {#if app.alumno?.actividades[id]?.completada && actividad.explicacion}
      <div class="info"><Markdown contenido={actividad.explicacion} /></div>
    {/if}
  </section>

  <section class="trabajo">
    <div class="herramientas">
      <button class="primario" onclick={ejecutar} disabled={cargando || corriendo || probando} title="Ejecutar (F5)">▶ Ejecutar</button>
      <button class="exito" onclick={probar} disabled={cargando || corriendo || probando} title="Probar (F6)">✔ Probar</button>
      <button onclick={detener} disabled={!corriendo && !probando} title="Detener (Esc)">■ Detener</button>
      <span class="espaciador"></span>
      <button class="fantasma chico" onclick={() => cambiarFuente(-1)} title="Letra más chica">A−</button>
      <button class="fantasma chico" onclick={() => cambiarFuente(1)} title="Letra más grande">A+</button>
      <button class="chico" onclick={() => (confirmarReinicio = true)} disabled={cargando} title="Volver al código inicial">↺</button>
      {#if app.estadoApp?.puede_generar_exe}
        <button class="chico" onclick={() => ((modalExe = true), (rutaExe = ""), (errorExe = ""))} disabled={cargando} title="Crear un programa .exe">⚙ Crear .exe</button>
      {/if}
    </div>
    <div class="editor" bind:this={contenedor} style:--editor-fuente="{fuente}px" data-editor>
      {#if errorCarga}<p class="error">{errorCarga}</p>{/if}
    </div>
    {#if tactil}
      <div class="barra-teclas" role="toolbar" aria-label="Teclas de código" data-barra-teclas>
        {#each TECLAS as t (t.etiqueta)}
          <!-- pointerdown sin foco: el teclado del celular sigue abierto -->
          <button type="button" title={t.titulo ?? t.etiqueta} aria-label={t.titulo ?? t.etiqueta}
            onpointerdown={(e) => e.preventDefault()} onmousedown={(e) => e.preventDefault()} onclick={() => tecla(t)}>{t.etiqueta}</button>
        {/each}
      </div>
    {/if}
    <div class="inferior">
      <div class="tabs" role="tablist">
        <button role="tab" class:activa={pestana === "consola"} onclick={() => (pestana = "consola")}>Consola</button>
        <button role="tab" class:activa={pestana === "pruebas"} onclick={() => (pestana = "pruebas")}>
          Pruebas {#if resultado}<span class="insignia" class:ok={resultado.pasadas === resultado.total}>{resultado.pasadas}/{resultado.total}</span>{/if}
        </button>
      </div>
      <div class="panel" hidden={pestana !== "consola"}>
        <Consola bind:this={consola} alIrALinea={(l) => editor?.irALinea(l)} />
      </div>
      <div class="panel" hidden={pestana !== "pruebas"}>
        <Resultados {resultado} {enCurso} alIrALinea={(l) => editor?.irALinea(l)} />
      </div>
    </div>
  </section>
</div>

<Modal titulo="¿Reiniciar el código?" abierto={confirmarReinicio} cerrar={() => (confirmarReinicio = false)}>
  <p>Tu código volverá a como estaba al principio de la actividad. Tu historial se conserva.</p>
  {#snippet acciones()}
    <button onclick={() => (confirmarReinicio = false)}>Cancelar</button>
    <button class="primario" onclick={reiniciar}>Reiniciar</button>
  {/snippet}
</Modal>

<Modal titulo="Generar ejecutable" abierto={modalExe} cerrar={generando ? undefined : () => (modalExe = false)} ancho="620px">
  <p class="suave">
    Crea un programa <strong>.exe</strong> que puedes abrir con doble clic en cualquier computadora con Windows, sin
    instalar Python. Se guarda en la carpeta <code>mis_ejecutables</code> de la app.
  </p>
  <div class="campo">
    <label for="nexe">Nombre del programa</label>
    <input id="nexe" bind:value={nombreExe} disabled={generando} />
  </div>
  {#if progresoExe.length}
    <pre class="progreso">{progresoExe.join("\n")}</pre>
  {/if}
  {#if errorExe}<p class="error">{errorExe}</p>{/if}
  {#if rutaExe}
    <p class="exito-msg">¡Listo! Tu programa está en: <code>{rutaExe}</code></p>
    <p class="suave">Si el antivirus lo bloquea, pide a tu profesor que agregue la carpeta de la app como excepción.</p>
  {/if}
  {#snippet acciones()}
    {#if rutaExe}<button onclick={abrirCarpetaExe}>📂 Abrir carpeta</button>{/if}
    <button onclick={() => (modalExe = false)} disabled={generando}>Cerrar</button>
    <button class="primario" onclick={generarExe} disabled={generando || !nombreExe.trim()}>
      {generando ? "Generando… (puede tardar un minuto)" : "Generar"}
    </button>
  {/snippet}
</Modal>

<style>
  .espacio {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: minmax(280px, 38%) 1fr;
  }
  .pestanas-movil {
    display: none;
  }
  .enunciado {
    overflow: auto;
    padding: 1rem 1.2rem 2rem;
    border-right: 1px solid var(--borde);
    background: var(--superficie);
  }
  .trabajo {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    padding: 0.5rem;
    gap: 0.5rem;
  }
  .herramientas {
    display: flex;
    gap: 0.4rem;
    align-items: center;
    flex-wrap: wrap;
  }
  .editor {
    flex: 3;
    min-height: 140px;
    border: 1px solid var(--borde);
    border-radius: var(--radio-chico);
    overflow: hidden;
    background: var(--editor-fondo);
  }
  .editor :global(.cm-editor) {
    height: 100%;
  }
  .barra-teclas {
    display: flex;
    gap: 0.3rem;
    overflow-x: auto;
    padding: 0.1rem 0;
    scrollbar-width: none;
    flex: none;
  }
  .barra-teclas button {
    flex: none;
    min-width: 2.6rem;
    min-height: 2.4rem;
    padding: 0 0.5rem;
    font-family: var(--fuente-codigo);
    font-size: 1.05rem;
    justify-content: center;
    touch-action: manipulation;
  }
  .inferior {
    flex: 2;
    min-height: 150px;
    display: flex;
    flex-direction: column;
    border: 1px solid var(--borde);
    border-radius: var(--radio-chico);
    overflow: hidden;
    background: var(--superficie);
  }
  .tabs {
    display: flex;
    border-bottom: 1px solid var(--borde);
    background: var(--superficie-2);
  }
  .tabs button {
    border: none;
    border-radius: 0;
    background: transparent;
    border-bottom: 2px solid transparent;
    color: var(--texto-suave);
  }
  .tabs button.activa {
    color: var(--texto);
    border-bottom-color: var(--primario);
    background: var(--superficie);
    font-weight: 600;
  }
  .insignia.ok {
    background: var(--exito-suave);
    color: var(--exito);
  }
  .panel {
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }
  .panel[hidden] {
    display: none;
  }
  .progreso {
    max-height: 220px;
    overflow: auto;
    background: var(--consola-fondo);
    color: var(--consola-texto);
    padding: 0.5em 0.8em;
    border-radius: var(--radio-chico);
    font-size: 0.82em;
    white-space: pre-wrap;
  }
  @media (max-width: 900px) {
    .espacio {
      grid-template-columns: 1fr;
      grid-template-rows: auto 1fr;
    }
    .pestanas-movil {
      display: flex;
      border-bottom: 1px solid var(--borde);
      background: var(--superficie);
    }
    .pestanas-movil button {
      flex: 1;
      border: none;
      border-radius: 0;
      border-bottom: 3px solid transparent;
      background: transparent;
    }
    .pestanas-movil button.activa {
      border-bottom-color: var(--primario);
      font-weight: 600;
    }
    .enunciado {
      border-right: none;
    }
    [data-panel="enunciado"] .trabajo,
    [data-panel="codigo"] .enunciado,
    [data-panel="salida"] .enunciado {
      display: none;
    }
    [data-panel="codigo"] .inferior {
      display: none;
    }
    [data-panel="salida"] .editor,
    [data-panel="salida"] .barra-teclas {
      display: none;
    }
  }
</style>
