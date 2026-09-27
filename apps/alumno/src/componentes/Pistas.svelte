<script lang="ts">
  import { backend } from "../lib/backend";
  import { actualizarActividad, app, refrescarEstadisticas } from "../lib/app.svelte";

  let { id, pistas }: { id: string; pistas: string[] } = $props();
  const vistas = $derived(app.alumno?.actividades[id]?.pistas ?? 0);

  async function otra() {
    const estado = await (await backend()).registrarPista(id, vistas + 1);
    actualizarActividad(id, estado);
    refrescarEstadisticas();
  }
</script>

{#if pistas.length}
  <section class="pistas">
    {#each pistas.slice(0, vistas) as p, i (i)}
      <p class="pista"><strong>Pista {i + 1}:</strong> {p}</p>
    {/each}
    {#if vistas < pistas.length}
      <button class="chico" onclick={otra}>💡 {vistas === 0 ? "Ver una pista" : "Ver otra pista"} ({vistas}/{pistas.length})</button>
    {/if}
  </section>
{/if}

<style>
  .pistas {
    margin-top: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    align-items: flex-start;
  }
  .pista {
    margin: 0;
    padding: 0.5em 0.8em;
    background: color-mix(in srgb, var(--acento) 14%, var(--superficie));
    border-radius: var(--radio-chico);
    border-left: 3px solid var(--acento);
  }
</style>
