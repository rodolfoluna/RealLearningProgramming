<script lang="ts">
  import type { ErrorPython } from "@rlp/python-worker";

  let { error, alIrALinea }: { error: ErrorPython; alIrALinea?: (linea: number) => void } = $props();
  let detalles = $state(false);
</script>

<div class="error-py" role="alert">
  <div class="titulo">
    <strong>{error.linea ? `Error en la línea ${error.linea}` : "Error"}</strong>
    <span class="tipo">{error.tipo}</span>
    {#if error.linea && alIrALinea}
      <button class="chico" onclick={() => alIrALinea(error.linea!)}>Ir a la línea</button>
    {/if}
  </div>
  <p class="explicacion">💡 {error.explicacion}</p>
  {#if error.texto_linea}
    <pre class="linea">{error.texto_linea}</pre>
  {/if}
  <button class="fantasma chico" onclick={() => (detalles = !detalles)}>{detalles ? "Ocultar" : "Ver"} mensaje original de Python</button>
  {#if detalles}
    <pre class="original">{#each error.traza as t}{t}
{/each}{error.tipo}: {error.mensaje}</pre>
  {/if}
</div>

<style>
  .error-py {
    background: #2a1416;
    color: #ffd9d9;
    border-top: 2px solid var(--consola-error);
    padding: 0.6em 0.9em;
    font-size: 0.92em;
    max-height: 65%;
    overflow: auto;
  }
  .titulo {
    display: flex;
    gap: 0.6em;
    align-items: center;
  }
  .tipo {
    font-family: var(--fuente-codigo);
    font-size: 0.85em;
    opacity: 0.8;
  }
  .titulo button,
  .error-py .fantasma {
    background: transparent;
    color: inherit;
    border-color: rgb(255 255 255 / 0.25);
  }
  .error-py .fantasma {
    border-color: transparent;
    padding-left: 0;
    opacity: 0.8;
  }
  .linea,
  .original {
    font-family: var(--fuente-codigo);
    background: rgb(0 0 0 / 0.25);
    padding: 0.3em 0.6em;
    border-radius: 4px;
    margin: 0.4em 0;
    white-space: pre-wrap;
  }
  .explicacion {
    margin: 0.4em 0;
    line-height: 1.45;
  }
</style>
