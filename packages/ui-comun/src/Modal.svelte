<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    titulo: string;
    abierto: boolean;
    cerrar?: () => void;
    ancho?: string;
    children: Snippet;
    acciones?: Snippet;
  }
  let { titulo, abierto, cerrar, ancho = "520px", children, acciones }: Props = $props();
  let dialogo: HTMLDialogElement | undefined = $state();

  $effect(() => {
    if (!dialogo) return;
    if (abierto && !dialogo.open) dialogo.showModal();
    if (!abierto && dialogo.open) dialogo.close();
  });
</script>

<dialog
  bind:this={dialogo}
  class="modal"
  style:max-width={ancho}
  oncancel={(e) => {
    e.preventDefault();
    cerrar?.();
  }}
>
  <header>
    <h2>{titulo}</h2>
    {#if cerrar}
      <button type="button" class="cerrar" aria-label="Cerrar" onclick={cerrar}>✕</button>
    {/if}
  </header>
  <div class="cuerpo">{@render children()}</div>
  {#if acciones}
    <footer>{@render acciones()}</footer>
  {/if}
</dialog>
