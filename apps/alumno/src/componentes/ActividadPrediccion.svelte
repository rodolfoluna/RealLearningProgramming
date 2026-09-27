<script lang="ts">
  import type { Actividad } from "@rlp/curso";
  import { Markdown, resaltarPython } from "@rlp/ui-comun";
  import { onMount } from "svelte";
  import { backend } from "../lib/backend";
  import { actualizarActividad, app, avisar, refrescarEstadisticas, registrarEvento } from "../lib/app.svelte";
  import Consola from "./Consola.svelte";
  import Pistas from "./Pistas.svelte";

  let { actividad }: { actividad: Actividad } = $props();
  // svelte-ignore state_referenced_locally (el componente se recrea al cambiar de actividad)
  const id = actividad.id;
  const estado = $derived(app.alumno?.actividades[id]);
  let respuesta = $state("");
  let resultado: "correcta" | "incorrecta" | null = $state(null);
  let consola: Consola | undefined = $state();
  let verReal = $state(false);

  onMount(() => {
    respuesta = estado?.respuesta ?? "";
    if (estado?.completada) resultado = "correcta";
    void registrarEvento("actividad_abierta", id, {});
  });

  const normalizar = (t: string) =>
    t
      .replace(/\r\n?/g, "\n")
      .split("\n")
      .map((l) => l.trimEnd())
      .join("\n")
      .replace(/^\n+|\n+$/g, "");

  async function verificar() {
    const correcta = normalizar(respuesta) === normalizar(actividad.salida_esperada ?? "");
    resultado = correcta ? "correcta" : "incorrecta";
    const e = await (await backend()).registrarRespuesta(id, respuesta, correcta, actividad.puntos);
    const antes = estado?.completada;
    actualizarActividad(id, e);
    refrescarEstadisticas();
    if (correcta && !antes) avisar(`🎉 ¡Correcto! +${actividad.puntos} puntos`);
  }

  async function ejecutarReal() {
    verReal = true;
    await Promise.resolve();
    void consola?.ejecutar(actividad.codigo ?? "");
  }

  function pegar(e: ClipboardEvent) {
    e.preventDefault();
    const chars = e.clipboardData?.getData("text/plain").length ?? 0;
    void registrarEvento("pegado", id, { chars, via: "teclado", interno: false, permitido: false, destino: "prediccion" });
    avisar("🚫 Pegar está deshabilitado: escribe tu respuesta.");
  }
</script>

<div class="prediccion">
  <Markdown contenido={actividad.enunciado} />
  <div class="bloque-codigo python"><pre><code>{@html resaltarPython((actividad.codigo ?? "").replace(/\n$/, ""))}</code></pre></div>

  <label for="resp">¿Qué mostrará en la consola?</label>
  <textarea id="resp" rows="6" bind:value={respuesta} onpaste={pegar} ondrop={(e) => e.preventDefault()} spellcheck="false"
    placeholder="Escribe aquí la salida, línea por línea"></textarea>

  <div class="fila acciones">
    <button class="primario" onclick={verificar} disabled={!respuesta.trim()}>Verificar</button>
    {#if (estado?.intentos ?? 0) >= 2 || resultado === "correcta"}
      <button onclick={ejecutarReal}>▶ Ejecutar el programa para comprobar</button>
    {/if}
  </div>

  {#if resultado === "correcta"}
    <div class="exito-msg">✔ ¡Correcto!</div>
    {#if actividad.explicacion}<div class="info"><Markdown contenido={actividad.explicacion} /></div>{/if}
  {:else if resultado === "incorrecta"}
    <div class="error">✖ No es la salida correcta. Revisa el programa línea por línea, como si fueras la computadora.</div>
  {/if}

  <Pistas {id} pistas={actividad.pistas} />

  {#if verReal}
    <div class="consola-real"><Consola bind:this={consola} compacta /></div>
  {/if}
</div>

<style>
  .prediccion {
    max-width: 820px;
    margin: 0 auto;
    padding: 1.2rem 1.5rem 3rem;
    overflow: auto;
    width: 100%;
  }
  textarea {
    font-size: 0.95em;
  }
  .acciones {
    margin: 0.8rem 0;
  }
  .consola-real {
    margin-top: 1rem;
  }
</style>
