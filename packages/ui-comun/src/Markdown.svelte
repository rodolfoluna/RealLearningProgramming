<script lang="ts">
  import { renderizarMarkdown } from "./markdown";

  interface Props {
    contenido: string;
    /** Si se da, los bloques de Python muestran un botón "Probar". */
    alProbar?: (codigo: string) => void;
    /** Se llama cuando el usuario copia texto de este contenido. */
    alCopiar?: (chars: number) => void;
  }
  let { contenido, alProbar, alCopiar }: Props = $props();

  const render = $derived(renderizarMarkdown(contenido, !!alProbar));

  function clic(e: MouseEvent) {
    const b = (e.target as HTMLElement).closest("button.probar") as HTMLButtonElement | null;
    if (b && alProbar) alProbar(render.bloques[Number(b.dataset.bloque)].codigo);
  }

  function copiar() {
    const texto = window.getSelection()?.toString() ?? "";
    if (texto) alCopiar?.(texto.length);
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="markdown" onclick={clic} oncopy={copiar}>
  {@html render.html}
</div>
