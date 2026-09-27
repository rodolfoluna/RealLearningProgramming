<script lang="ts">
  import { mensajeError } from "@rlp/ui-comun";
  import { onMount } from "svelte";
  import Inicio from "./componentes/Inicio.svelte";
  import Principal from "./componentes/Principal.svelte";
  import { app, cargarEstado } from "./lib/app.svelte";

  let error = $state("");

  onMount(async () => {
    try {
      await cargarEstado();
      if (app.estado?.autoprueba) void import("./lib/autoprueba").then((m) => m.autoprueba(app.estado!.autoprueba!));
    } catch (e) {
      error = mensajeError(e);
    }
  });
</script>

{#if error}
  <main class="centro"><p class="error">{error}</p></main>
{:else if !app.estado}
  <main class="centro"><p class="suave">Cargando…</p></main>
{:else if !app.estado.desbloqueado}
  <Inicio />
{:else}
  <Principal />
{/if}

{#if app.aviso}
  <div class="aviso-flotante" role="status">{app.aviso}</div>
{/if}
