<script lang="ts">
  import type { ErrorPython, ResultadoEjecucion } from "@rlp/python-worker";
  import { tick } from "svelte";
  import { python } from "../lib/app.svelte";
  import ErrorPy from "./ErrorPython.svelte";

  interface Props {
    /** Clic en "ir a la línea" de un error. */
    alIrALinea?: (linea: number) => void;
    compacta?: boolean;
  }
  let { alIrALinea, compacta = false }: Props = $props();

  type Tipo = "out" | "err" | "entrada" | "sistema";
  let segmentos: { texto: string; tipo: Tipo }[] = $state([]);
  let estado: "inactiva" | "preparando" | "corriendo" | "ok" | "error" | "detenido" = $state("inactiva");
  let esperando = $state(false);
  let entrada = $state("");
  let error: ErrorPython | null = $state(null);
  let duracion = $state(0);
  let caja: HTMLDivElement | undefined = $state();
  let campo: HTMLInputElement | undefined = $state();
  let total = 0;
  const LIMITE = 200_000;

  const ANSI = /\x1b\[[0-9;]*[A-Za-z]/g;

  async function bajar() {
    await tick();
    if (caja) caja.scrollTop = caja.scrollHeight;
  }

  function agregar(texto: string, tipo: Tipo) {
    if (!texto) return;
    texto = texto.replace(ANSI, "");
    if (total > LIMITE) return;
    total += texto.length;
    const ultimo = segmentos[segmentos.length - 1];
    if (ultimo && ultimo.tipo === tipo) ultimo.texto += texto;
    else segmentos.push({ texto, tipo });
    void bajar();
  }

  export function limpiar() {
    segmentos = [];
    total = 0;
    error = null;
    estado = "inactiva";
  }

  export function corriendo(): boolean {
    return estado === "corriendo" || estado === "preparando";
  }

  /** Ejecuta un programa en la consola (input() pide datos aquí). */
  export async function ejecutar(codigo: string): Promise<ResultadoEjecucion> {
    limpiar();
    estado = "preparando";
    await python.iniciar();
    estado = "corriendo";
    const t0 = performance.now();
    const r = await python.ejecutar(codigo, {
      salida: (t, flujo) => agregar(t, flujo),
      entradaSolicitada: async () => {
        esperando = true;
        await tick();
        campo?.focus();
      },
      limpiar: () => {
        segmentos = [];
        total = 0;
      },
    });
    esperando = false;
    duracion = (performance.now() - t0) / 1000;
    if (r.salida_excedida) agregar("\n[El programa imprimió demasiado texto y se detuvo. ¿Hay un ciclo infinito?]\n", "sistema");
    error = r.error;
    estado = r.estado === "ok" || r.estado === "salida" ? "ok" : r.estado === "detenido" ? "detenido" : "error";
    void bajar();
    return r;
  }

  export function detener() {
    python.detener();
  }

  function enviar(e: SubmitEvent) {
    e.preventDefault();
    if (!esperando) return;
    agregar(entrada + "\n", "entrada");
    python.enviarEntrada(entrada);
    entrada = "";
    esperando = false;
  }
</script>

<div class="consola" class:compacta>
  <div class="estado-barra">
    {#if estado === "preparando"}
      <span>⏳ Preparando Python…</span>
    {:else if estado === "corriendo"}
      <span>▶ Ejecutando… {#if esperando}<strong>esperando tu dato</strong>{/if}</span>
    {:else if estado === "ok"}
      <span class="ok">✔ Programa terminado ({duracion.toFixed(1)} s)</span>
    {:else if estado === "error"}
      <span class="mal">✖ El programa terminó con un error</span>
    {:else if estado === "detenido"}
      <span>■ Programa detenido</span>
    {:else}
      <span class="suave">Consola — aquí verás lo que muestra tu programa</span>
    {/if}
  </div>
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div class="salida" bind:this={caja} onclick={() => esperando && campo?.focus()} data-consola>
    <pre>{#each segmentos as s, i (i)}<span class={s.tipo}>{s.texto}</span>{/each}</pre>
    {#if esperando}
      <form onsubmit={enviar} class="entrada">
        <span aria-hidden="true">›</span>
        <input bind:this={campo} bind:value={entrada} aria-label="Dato para el programa" autocomplete="off" spellcheck="false" />
        <button type="submit" class="chico">Enviar ⏎</button>
      </form>
    {/if}
  </div>
  {#if python.modo === "limitado"}
    <p class="limitado" role="note">
      Aquí tus programas no pueden pedir datos con <code>input()</code>: este navegador no aisló la página.
      Recarga la página; si sigue igual, instala la app o usa otro navegador.
    </p>
  {/if}
  {#if error}
    <ErrorPy {error} {alIrALinea} />
  {/if}
</div>

<style>
  .limitado {
    margin: 0;
    padding: 0.4em 0.75em;
    font-size: 0.85em;
    background: var(--aviso-suave);
    color: var(--aviso);
  }
  .consola {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    background: var(--consola-fondo);
    color: var(--consola-texto);
    border-radius: var(--radio-chico);
    overflow: hidden;
  }
  .estado-barra {
    padding: 0.3em 0.8em;
    font-size: 0.85em;
    background: rgb(255 255 255 / 0.05);
    border-bottom: 1px solid rgb(255 255 255 / 0.08);
  }
  .estado-barra .suave {
    color: #93a0b3;
  }
  .ok {
    color: #7fe0a0;
  }
  .mal {
    color: var(--consola-error);
  }
  .salida {
    flex: 1;
    overflow: auto;
    padding: 0.6em 0.8em;
    font-family: var(--fuente-codigo);
    font-size: 0.92em;
    min-height: 60px;
    cursor: text;
  }
  pre {
    margin: 0;
    white-space: pre-wrap;
    word-break: break-word;
    font-family: inherit;
    display: inline;
  }
  .err {
    color: var(--consola-error);
  }
  .entrada {
    color: var(--consola-entrada);
  }
  .sistema {
    color: #f0c674;
  }
  form.entrada {
    display: flex;
    gap: 0.4em;
    align-items: center;
    margin-top: 0.2em;
  }
  form.entrada input {
    flex: 1;
    background: rgb(255 255 255 / 0.07);
    color: var(--consola-entrada);
    border-color: rgb(255 255 255 / 0.15);
    font-family: var(--fuente-codigo);
    padding: 0.25em 0.5em;
  }
  form.entrada button {
    background: transparent;
    color: var(--consola-texto);
    border-color: rgb(255 255 255 / 0.2);
  }
  .compacta .salida {
    max-height: 260px;
  }
</style>
