<script lang="ts">
  import { onMount } from "svelte";
  import Inicio from "./componentes/Inicio.svelte";
  import Principal from "./componentes/Principal.svelte";
  import { app, aplicarTema, cargarEstadoApp, prepararCierre } from "./lib/app.svelte";
  import { enTauri } from "./lib/backend";
  import { mensajeError } from "@rlp/ui-comun";

  let errorFatal = $state("");

  onMount(async () => {
    aplicarTema(app.tema);
    if (enTauri()) {
      const [{ listen }, { invoke }] = await Promise.all([import("@tauri-apps/api/event"), import("@tauri-apps/api/core")]);
      void listen("cerrando", async () => {
        await prepararCierre();
        await invoke("cerrar_app");
      });
    }
    try {
      await cargarEstadoApp();
      app.vista = "inicio";
      if (app.estadoApp?.autoprueba) void import("./lib/autoprueba").then((m) => m.autoprueba(app.estadoApp!.autoprueba!));
    } catch (e) {
      errorFatal = mensajeError(e);
    }
  });
</script>

{#if errorFatal}
  <main class="centro">
    <div class="tarjeta" style="max-width: 560px">
      <h2>No se pudo iniciar la app</h2>
      <p class="error">{errorFatal}</p>
    </div>
  </main>
{:else if app.vista === "cargando"}
  <main class="centro"><p class="suave">Cargando…</p></main>
{:else if app.vista === "inicio"}
  <Inicio />
{:else}
  <Principal />
{/if}

{#if app.aviso}
  <div class="aviso-flotante" role="status">{app.aviso}</div>
{/if}
