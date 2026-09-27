<script lang="ts">
  interface Props {
    nivel: "verde" | "amarillo" | "rojo";
    texto?: boolean;
  }
  let { nivel, texto = true }: Props = $props();
  const etiquetas = { verde: "Íntegro", amarillo: "Revisar", rojo: "Alterado" };
</script>

<span class="semaforo {nivel}" title={etiquetas[nivel]}>
  <span class="punto" aria-hidden="true"></span>{#if texto}{etiquetas[nivel]}{/if}
</span>
