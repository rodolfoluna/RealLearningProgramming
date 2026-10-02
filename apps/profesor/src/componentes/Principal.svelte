<script lang="ts">
  import { backend } from "../lib/backend";
  import { app, cargarEstado } from "../lib/app.svelte";
  import Tablero from "./Tablero.svelte";
  import Detalle from "./Detalle.svelte";
  import Grupos from "./Grupos.svelte";
  import Importar from "./Importar.svelte";

  async function bloquear() {
    await (await backend()).bloquear();
    await cargarEstado();
  }
</script>

<div class="principal">
  <header class="barra">
    <strong class="marca">🧑‍🏫 LP Profesor</strong>
    <nav class="fila">
      <button class:activa={app.vista.tipo === "tablero" || app.vista.tipo === "detalle"} onclick={() => (app.vista = { tipo: "tablero" })}>📊 Tablero</button>
      <button class:activa={app.vista.tipo === "importar"} onclick={() => (app.vista = { tipo: "importar" })}>📥 Importar entregas</button>
      <button class:activa={app.vista.tipo === "grupos"} onclick={() => (app.vista = { tipo: "grupos" })}>👥 Grupos</button>
    </nav>
    <span class="espaciador"></span>
    <label class="selector">
      <span class="suave">Grupo</span>
      <select bind:value={app.grupoId}>
        <option value={null}>Todos</option>
        {#each app.grupos as g (g.grupo_id)}
          <option value={g.grupo_id}>{g.nombre}</option>
        {/each}
      </select>
    </label>
    <span class="suave">{app.estado?.nombre}</span>
    <button class="chico" onclick={bloquear} title="Cerrar y proteger tus llaves">🔒 Bloquear</button>
  </header>
  <main class="contenido">
    {#if app.vista.tipo === "tablero"}
      <Tablero />
    {:else if app.vista.tipo === "detalle"}
      {#key app.vista.entregaId}
        <Detalle entregaId={app.vista.entregaId} />
      {/key}
    {:else if app.vista.tipo === "grupos"}
      <Grupos />
    {:else}
      <Importar />
    {/if}
  </main>
</div>

<style>
  .principal {
    height: 100%;
    display: flex;
    flex-direction: column;
  }
  .barra {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.5rem 1rem;
    background: var(--superficie);
    border-bottom: 1px solid var(--borde);
  }
  nav button {
    border: none;
    background: transparent;
    color: var(--texto-suave);
  }
  nav button.activa {
    background: var(--primario-suave);
    color: var(--primario);
    font-weight: 600;
  }
  .selector {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin: 0;
    font-weight: normal;
  }
  .selector select {
    width: auto;
    min-width: 180px;
  }
  .contenido {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }
</style>
