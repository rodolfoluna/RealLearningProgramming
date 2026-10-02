<script lang="ts">
  import { Modal, mensajeError } from "@rlp/ui-comun";
  import { backend } from "../lib/backend";
  import { app, cargarEstado } from "../lib/app.svelte";

  const estado = $derived(app.estado!);
  let modo: "desbloquear" | "crear" | "restaurar" = $state(app.estado?.tiene_identidad ? "desbloquear" : "crear");
  let nombre = $state("");
  let clave = $state("");
  let clave2 = $state("");
  let archivo = $state("");
  let error = $state("");
  let ocupado = $state(false);
  let pedirRespaldo = $state(false);
  let respaldo = $state("");

  async function accion(fn: () => Promise<void>) {
    error = "";
    ocupado = true;
    try {
      await fn();
    } catch (e) {
      error = mensajeError(e);
    } finally {
      ocupado = false;
    }
  }

  const desbloquear = () =>
    accion(async () => {
      await (await backend()).desbloquear(clave);
      await cargarEstado();
    });

  const crear = () =>
    accion(async () => {
      if (clave !== clave2) throw new Error("Las contraseñas no coinciden.");
      await (await backend()).crearIdentidad(nombre, clave);
      pedirRespaldo = true;
    });

  const elegir = () =>
    accion(async () => {
      const r = await (await backend()).elegirArchivos("Elige tu respaldo de llaves", "rlpk", "Respaldo de llaves", false);
      if (r.length) archivo = r[0];
    });

  const restaurar = () =>
    accion(async () => {
      if (!archivo) throw new Error("Elige primero tu archivo de respaldo.");
      await (await backend()).restaurarIdentidad(archivo, clave);
      await cargarEstado();
    });

  const guardarRespaldo = () =>
    accion(async () => {
      const b = await backend();
      const carpeta = await b.elegirCarpeta("¿Dónde guardo el respaldo? (usa una memoria USB o tu nube)");
      if (carpeta) respaldo = await b.respaldarIdentidad(carpeta);
    });
</script>

<main class="inicio">
  <section>
    <div class="logo" aria-hidden="true">🧑‍🏫</div>
    <h1>LP Profesor</h1>
    <p class="suave">Revisa las entregas de tus alumnos, verifica que no se hayan modificado fuera de la app y sigue su avance.</p>
    <ul class="suave puntos">
      <li>Tus llaves se guardan cifradas con tu contraseña en <code>{estado.carpeta}</code>.</li>
      <li>Solo tú (y los coprofesores que agregues) pueden abrir las entregas de tus grupos.</li>
      <li>Todo funciona sin internet.</li>
    </ul>
  </section>

  <section class="tarjeta">
    {#if modo === "desbloquear"}
      <h2>Hola{estado.nombre ? `, ${estado.nombre}` : ""}</h2>
      <form onsubmit={(e) => { e.preventDefault(); desbloquear(); }}>
        <div class="campo">
          <label for="c">Contraseña de tus llaves</label>
          <!-- svelte-ignore a11y_autofocus -->
          <input id="c" type="password" bind:value={clave} autofocus />
        </div>
        {#if error}<p class="error">{error}</p>{/if}
        <div class="fila">
          <button type="button" class="fantasma chico" onclick={() => (modo = "restaurar")}>Restaurar desde un respaldo</button>
          <span class="espaciador"></span>
          <button class="primario" disabled={ocupado}>Desbloquear</button>
        </div>
      </form>
    {:else if modo === "crear"}
      <h2>Crear mis llaves de profesor</h2>
      <form onsubmit={(e) => { e.preventDefault(); crear(); }}>
        <div class="campo"><label for="n">Tu nombre (lo verán tus alumnos)</label><input id="n" bind:value={nombre} /></div>
        <div class="campo">
          <label for="c1">Contraseña</label><input id="c1" type="password" bind:value={clave} />
          <p class="ayuda">Al menos 10 caracteres. Protege las llaves con las que abres las entregas.</p>
        </div>
        <div class="campo"><label for="c2">Repite la contraseña</label><input id="c2" type="password" bind:value={clave2} /></div>
        {#if error}<p class="error">{error}</p>{/if}
        <div class="fila">
          <button type="button" class="fantasma chico" onclick={() => (modo = "restaurar")}>Ya tengo un respaldo</button>
          <span class="espaciador"></span>
          <button class="primario" disabled={ocupado || !estado.escribible}>Crear llaves</button>
        </div>
      </form>
    {:else}
      <h2>Restaurar desde un respaldo</h2>
      <p class="suave">Usa el archivo <code>.rlpk</code> que guardaste al crear tus llaves (por ejemplo, para usar otra computadora).</p>
      <form onsubmit={(e) => { e.preventDefault(); restaurar(); }}>
        <div class="campo fila">
          <button type="button" onclick={elegir}>Elegir respaldo…</button>
          <span class="suave">{archivo || "Ningún archivo elegido"}</span>
        </div>
        <div class="campo"><label for="rc">Contraseña</label><input id="rc" type="password" bind:value={clave} /></div>
        {#if error}<p class="error">{error}</p>{/if}
        <div class="fila">
          <button type="button" class="fantasma chico" onclick={() => (modo = estado.tiene_identidad ? "desbloquear" : "crear")}>Volver</button>
          <span class="espaciador"></span>
          <button class="primario" disabled={ocupado}>Restaurar</button>
        </div>
      </form>
    {/if}
    {#if !estado.escribible}
      <p class="error">Esta carpeta no permite guardar datos. Copia la app a Documentos o a una memoria USB.</p>
    {/if}
  </section>
</main>

<Modal titulo="Guarda un respaldo de tus llaves" abierto={pedirRespaldo}>
  <p>
    <strong>Muy importante:</strong> si pierdes tus llaves (se daña la computadora, formateas o borras la carpeta),
    <strong>no podrás abrir las entregas</strong> de tus alumnos. Guarda un respaldo en una memoria USB o en tu nube;
    está protegido con tu contraseña.
  </p>
  {#if respaldo}<p class="exito-msg">Respaldo guardado en <code>{respaldo}</code></p>{/if}
  {#if error}<p class="error">{error}</p>{/if}
  {#snippet acciones()}
    <button onclick={guardarRespaldo} class:primario={!respaldo} disabled={ocupado}>💾 Guardar respaldo…</button>
    <button class:primario={!!respaldo} disabled={!respaldo} onclick={() => ((pedirRespaldo = false), cargarEstado())}>Continuar</button>
  {/snippet}
</Modal>

<style>
  .inicio {
    min-height: 100%;
    display: grid;
    grid-template-columns: 1fr minmax(0, 460px);
    gap: 3rem;
    align-items: center;
    max-width: 1000px;
    margin: 0 auto;
    padding: 2rem;
  }
  .logo {
    font-size: 3rem;
  }
  .puntos {
    padding-left: 1.2em;
  }
  @media (max-width: 800px) {
    .inicio {
      grid-template-columns: 1fr;
    }
  }
</style>
