<script lang="ts">
  import { onMount } from "svelte";
  import Inicio from "./componentes/Inicio.svelte";
  import Principal from "./componentes/Principal.svelte";
  import { app, aplicarTema, cargarEstadoApp } from "./lib/app.svelte";
  import { opciones } from "./lib/opciones";
  import { mensajeError } from "@rlp/ui-comun";

  let errorFatal = $state("");

  onMount(async () => {
    aplicarTema(app.tema);
    try {
      await opciones.alIniciar?.();
      await cargarEstadoApp();
      app.vista = "inicio";
      const fase = app.estadoApp?.autoprueba;
      if (fase && opciones.autoprueba) void opciones.autoprueba(fase);
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
