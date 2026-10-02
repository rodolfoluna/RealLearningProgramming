<script lang="ts">
  import { Modal, mensajeError } from "@rlp/ui-comun";
  import { backend } from "../lib/backend";
  import { app, cargarEstadoApp, curso, entrar, esWeb, lugarDeDatos, puedeEscanearQr } from "../lib/app.svelte";
  import type { EstadoAlumno, InfoAcceso, PerfilLocal } from "../lib/tipos";

  type Modo = "entrar" | "registro" | "restaurar" | "acceso";
  let modo: Modo = $state(app.estadoApp?.perfiles.length ? "entrar" : "registro");
  let error = $state("");
  let ocupado = $state(false);

  // Entrar
  let perfil: PerfilLocal | null = $state(app.estadoApp?.perfiles.length === 1 ? app.estadoApp.perfiles[0] : null);
  let contrasena = $state("");
  let olvide = $state(false);
  let codigo = $state("");
  let nueva = $state("");
  let nueva2 = $state("");

  // Registro
  let numeroControl = $state("");
  let nombre = $state("");
  let clave1 = $state("");
  let clave2 = $state("");

  // Restaurar
  let archivo = $state("");
  let usarCodigo = $state(false);

  // Archivo de acceso del profesor
  let rutaAcceso = $state("");
  let infoAcceso = $state<InfoAcceso | null>(null);
  let temporal = $state("");
  let rutaEntrega = $state("");

  // Código de recuperación recién creado
  let codigoNuevo = $state("");
  let anotado = $state(false);
  let estadoPendiente: EstadoAlumno | null = null;

  const estadoApp = $derived(app.estadoApp!);
  const grupo = $derived(estadoApp.grupo);

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

  function seleccionar(p: PerfilLocal) {
    perfil = p;
    contrasena = "";
    olvide = false;
    error = "";
  }

  const iniciar = () =>
    accion(async () => {
      if (!perfil) return;
      const b = await backend();
      if (olvide) {
        if (nueva !== nueva2) throw new Error("Las contraseñas nuevas no coinciden.");
        entrar(await b.iniciarSesion(perfil.carpeta, { tipo: "codigo", codigo, nueva_contrasena: nueva }));
      } else {
        entrar(await b.iniciarSesion(perfil.carpeta, { tipo: "contrasena", contrasena }));
      }
    });

  const registrarse = () =>
    accion(async () => {
      if (clave1 !== clave2) throw new Error("Las contraseñas no coinciden.");
      const r = await (await backend()).registrar(numeroControl, nombre, clave1);
      estadoPendiente = r.estado;
      codigoNuevo = r.codigo;
    });

  const elegirRespaldo = () =>
    accion(async () => {
      const r = await (await backend()).elegirArchivo("Elige tu archivo de avances", "rlp", "Avances de LP");
      if (r) archivo = r;
    });

  const restaurar = () =>
    accion(async () => {
      if (!archivo) throw new Error("Elige primero tu archivo .rlp.");
      const b = await backend();
      if (usarCodigo) {
        if (nueva !== nueva2) throw new Error("Las contraseñas nuevas no coinciden.");
        entrar(await b.restaurar(archivo, { tipo: "codigo", codigo, nueva_contrasena: nueva }));
      } else {
        entrar(await b.restaurar(archivo, { tipo: "contrasena", contrasena }));
      }
    });

  const elegirAcceso = () =>
    accion(async () => {
      const b = await backend();
      const r = await b.elegirArchivo("Elige el archivo de acceso que te dio tu profesor", "rlpa", "Acceso de LP");
      if (!r) return;
      infoAcceso = await b.leerAcceso(r);
      rutaAcceso = r;
    });

  const elegirEntrega = () =>
    accion(async () => {
      const r = await (await backend()).elegirArchivo("Elige tu último archivo de avances", "rlp", "Avances de LP");
      if (r) rutaEntrega = r;
    });

  const entrarConAcceso = () =>
    accion(async () => {
      if (!infoAcceso) throw new Error("Elige primero el archivo de acceso.");
      if (!infoAcceso.perfil_local && !rutaEntrega) throw new Error("Elige también tu último archivo .rlp.");
      if (nueva !== nueva2) throw new Error("Las contraseñas nuevas no coinciden.");
      const est = await (await backend()).entrarConAcceso(rutaAcceso, temporal, nueva, infoAcceso.perfil_local ? null : rutaEntrega);
      estadoPendiente = est;
      if (est.codigo_nuevo) codigoNuevo = est.codigo_nuevo;
      else entrar(est);
    });

  const importarGrupo = () =>
    accion(async () => {
      const b = await backend();
      const r = await b.elegirArchivo("Elige el archivo de grupo que te dio tu profesor", "rlpg", "Grupo de LP");
      if (!r) return;
      await b.importarGrupo(r);
      await cargarEstadoApp();
    });

  const escanearGrupo = () =>
    accion(async () => {
      const b = await backend();
      const texto = await b.escanearQr();
      if (!texto) return;
      await b.importarGrupoQr(texto);
      await cargarEstadoApp();
    });

  function continuar() {
    if (estadoPendiente) entrar(estadoPendiente);
  }
</script>

<main class="inicio">
  <section class="portada">
    <div class="logo" aria-hidden="true">🐍</div>
    <h1>{curso.titulo}</h1>
    <p class="suave">{curso.descripcion}</p>
    {#if grupo}
      <div class="grupo">
        <strong>{grupo.nombre}</strong>
        <span class="suave">{grupo.materia} · {grupo.periodo}</span>
        <span class="suave">Profesor(a): {grupo.profesor}</span>
      </div>
    {:else}
      <div class="aviso">
        <strong>Sin grupo.</strong> Para entregar tus avances, importa el archivo de grupo (.rlpg) que te dio tu profesor.
        Mientras tanto puedes practicar.
        <div class="fila" style="margin-top: 0.5em">
          {#if puedeEscanearQr()}
            <button class="primario" onclick={escanearGrupo} disabled={ocupado}>📷 Escanear QR del grupo</button>
          {/if}
          <button onclick={importarGrupo} disabled={ocupado}>Importar archivo de grupo</button>
        </div>
      </div>
    {/if}
    {#if !estadoApp.escribible && esWeb()}
      <p class="error">
        Este navegador no permite guardar datos (¿es una ventana privada?). Abre la app en una ventana normal.
      </p>
    {:else if !estadoApp.escribible}
      <p class="error">
        Esta carpeta no permite guardar datos ({estadoApp.carpeta_datos}). Copia la carpeta de la app a Documentos
        o a tu memoria USB y ábrela desde ahí.
      </p>
    {/if}
    <p class="suave pie">
      Tus avances se guardan cifrados en {lugarDeDatos()}: nadie más puede abrirlos sin tu contraseña.
      {#if esWeb()}
        Si borras los datos de este sitio se borran también: exporta tu entrega seguido, te sirve de respaldo.
      {/if}
    </p>
  </section>

  <section class="tarjeta formulario">
    <div class="pestanas" role="tablist">
      <button role="tab" class:activa={modo === "entrar"} aria-selected={modo === "entrar"} onclick={() => (modo = "entrar")}
        disabled={!estadoApp.perfiles.length}>Entrar</button>
      <button role="tab" class:activa={modo === "registro"} aria-selected={modo === "registro"} onclick={() => (modo = "registro")}>
        Soy nuevo
      </button>
      <button role="tab" class:activa={modo === "restaurar"} aria-selected={modo === "restaurar"} onclick={() => (modo = "restaurar")}>
        Tengo mis avances en un archivo
      </button>
    </div>

    {#if modo === "entrar"}
      <div class="perfiles">
        {#each estadoApp.perfiles as p (p.carpeta)}
          <button class="perfil" class:elegido={perfil?.carpeta === p.carpeta} onclick={() => seleccionar(p)}>
            <span class="avatar">{p.perfil.nombre.slice(0, 1).toUpperCase()}</span>
            <span class="datos">
              <strong>{p.perfil.nombre}</strong>
              <span class="suave">{p.perfil.numero_control}{p.grupo ? ` · ${p.grupo}` : ""}</span>
            </span>
          </button>
        {/each}
      </div>
      {#if perfil}
        <form onsubmit={(e) => { e.preventDefault(); iniciar(); }}>
          {#if !olvide}
            <div class="campo">
              <label for="clave">Contraseña</label>
              <!-- svelte-ignore a11y_autofocus -->
              <input id="clave" type="password" bind:value={contrasena} autocomplete="current-password" autofocus />
            </div>
          {:else}
            <p class="info">Escribe el código de recuperación que anotaste al registrarte y elige una contraseña nueva.</p>
            <div class="campo">
              <label for="codigo">Código de recuperación</label>
              <input id="codigo" bind:value={codigo} placeholder="XXXX-XXXX-XXXX-XXXX-XXXX" autocomplete="off" />
            </div>
            <div class="campo">
              <label for="nueva">Contraseña nueva</label>
              <input id="nueva" type="password" bind:value={nueva} autocomplete="new-password" />
            </div>
            <div class="campo">
              <label for="nueva2">Repite la contraseña nueva</label>
              <input id="nueva2" type="password" bind:value={nueva2} autocomplete="new-password" />
            </div>
          {/if}
          {#if error}<p class="error">{error}</p>{/if}
          <div class="fila">
            <button type="button" class="fantasma chico" onclick={() => (olvide = !olvide)}>
              {olvide ? "Usar mi contraseña" : "Olvidé mi contraseña"}
            </button>
            <span class="espaciador"></span>
            <button type="submit" class="primario" disabled={ocupado}>Entrar</button>
          </div>
        </form>
      {:else}
        <p class="suave">Elige tu perfil.</p>
      {/if}
    {:else if modo === "registro"}
      <form onsubmit={(e) => { e.preventDefault(); registrarse(); }}>
        <p class="suave">Esto se hace solo la primera vez en {esWeb() ? "este navegador" : "esta carpeta"}.</p>
        <div class="campo">
          <label for="nc">Número de control</label>
          <input id="nc" bind:value={numeroControl} autocomplete="off" inputmode="numeric" />
        </div>
        <div class="campo">
          <label for="nombre">Nombre completo</label>
          <input id="nombre" bind:value={nombre} autocomplete="name" />
        </div>
        <div class="campo">
          <label for="c1">Contraseña</label>
          <input id="c1" type="password" bind:value={clave1} autocomplete="new-password" />
          <p class="ayuda">Al menos 8 caracteres. La necesitarás para abrir tus trabajos aquí o en otro equipo.</p>
        </div>
        <div class="campo">
          <label for="c2">Repite la contraseña</label>
          <input id="c2" type="password" bind:value={clave2} autocomplete="new-password" />
        </div>
        {#if error}<p class="error">{error}</p>{/if}
        <div class="fila">
          <span class="espaciador"></span>
          <button type="submit" class="primario" disabled={ocupado || !estadoApp.escribible}>Crear mi perfil</button>
        </div>
      </form>
    {:else if modo === "restaurar"}
      <form onsubmit={(e) => { e.preventDefault(); restaurar(); }}>
        <p class="suave">
          ¿Trabajaste en otra computadora o en tu celular? Elige el archivo <strong>.rlp</strong> que exportaste allá
          para continuar aquí.
        </p>
        <div class="campo fila">
          <button type="button" onclick={elegirRespaldo} disabled={ocupado}>Elegir archivo…</button>
          <span class="suave archivo">{archivo || "Ningún archivo elegido"}</span>
        </div>
        {#if !usarCodigo}
          <div class="campo">
            <label for="rclave">Tu contraseña</label>
            <input id="rclave" type="password" bind:value={contrasena} autocomplete="current-password" />
          </div>
        {:else}
          <div class="campo">
            <label for="rcodigo">Código de recuperación</label>
            <input id="rcodigo" bind:value={codigo} autocomplete="off" />
          </div>
          <div class="campo">
            <label for="rnueva">Contraseña nueva</label>
            <input id="rnueva" type="password" bind:value={nueva} autocomplete="new-password" />
          </div>
          <div class="campo">
            <label for="rnueva2">Repite la contraseña nueva</label>
            <input id="rnueva2" type="password" bind:value={nueva2} autocomplete="new-password" />
          </div>
        {/if}
        {#if error}<p class="error">{error}</p>{/if}
        <div class="fila">
          <button type="button" class="fantasma chico" onclick={() => (usarCodigo = !usarCodigo)}>
            {usarCodigo ? "Usar mi contraseña" : "Olvidé mi contraseña"}
          </button>
          <span class="espaciador"></span>
          <button type="submit" class="primario" disabled={ocupado || !estadoApp.escribible}>Continuar aquí</button>
        </div>
      </form>
    {:else}
      <form onsubmit={(e) => { e.preventDefault(); entrarConAcceso(); }} data-acceso>
        <p class="suave">
          Si olvidaste tu contraseña y tu código de recuperación, tu profesor puede darte un <strong>archivo de acceso</strong>
          (.rlpa) y una contraseña temporal.
        </p>
        <div class="campo fila">
          <button type="button" onclick={elegirAcceso} disabled={ocupado}>Elegir archivo de acceso…</button>
          <span class="suave archivo">{infoAcceso ? `${infoAcceso.nombre} · ${infoAcceso.numero_control}` : "Ningún archivo elegido"}</span>
        </div>
        {#if infoAcceso}
          <p class="info">Archivo de {infoAcceso.profesor} para <strong>{infoAcceso.nombre}</strong>.</p>
          {#if !infoAcceso.perfil_local}
            <div class="campo fila">
              <button type="button" onclick={elegirEntrega} disabled={ocupado}>Elegir mi último .rlp…</button>
              <span class="suave archivo">{rutaEntrega || "Tu perfil no está aquí: elige tu último archivo de avances"}</span>
            </div>
          {/if}
          <div class="campo">
            <label for="temporal">Contraseña temporal (te la dio tu profesor)</label>
            <input id="temporal" bind:value={temporal} autocomplete="off" placeholder="XXXX-XXXX-XXXX" />
          </div>
          <div class="campo">
            <label for="anueva">Contraseña nueva</label>
            <input id="anueva" type="password" bind:value={nueva} autocomplete="new-password" />
          </div>
          <div class="campo">
            <label for="anueva2">Repite la contraseña nueva</label>
            <input id="anueva2" type="password" bind:value={nueva2} autocomplete="new-password" />
          </div>
        {/if}
        {#if error}<p class="error">{error}</p>{/if}
        <div class="fila">
          <button type="button" class="fantasma chico" onclick={() => (modo = estadoApp.perfiles.length ? "entrar" : "registro")}>← Volver</button>
          <span class="espaciador"></span>
          <button type="submit" class="primario" disabled={ocupado || !infoAcceso || !estadoApp.escribible}>Entrar</button>
        </div>
      </form>
    {/if}
    {#if modo !== "acceso"}
      <button type="button" class="fantasma chico acceso-profesor" onclick={() => { modo = "acceso"; error = ""; }}>
        ¿Olvidaste tu contraseña y tu código? Tengo un archivo de acceso de mi profesor
      </button>
    {/if}
  </section>
  <p class="version suave">v{estadoApp.version} · {estadoApp.plataforma}{estadoApp.dev ? " · compilación de desarrollo" : ""}</p>
</main>

<Modal titulo="Tu código de recuperación" abierto={!!codigoNuevo} ancho="480px">
  <p>Si olvidas tu contraseña, este código es la única forma de recuperar tus trabajos. <strong>Anótalo en tu cuaderno</strong> o en un lugar seguro. Solo se muestra esta vez.</p>
  <p class="codigo-recuperacion">{codigoNuevo}</p>
  <label class="fila" style="font-weight: normal">
    <input type="checkbox" bind:checked={anotado} style="width: auto" /> Ya lo anoté en un lugar seguro
  </label>
  {#snippet acciones()}
    <button class="primario" disabled={!anotado} onclick={continuar}>{modo === "acceso" ? "Continuar" : "Empezar el curso"}</button>
  {/snippet}
</Modal>

<style>
  .inicio {
    min-height: 100%;
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 480px);
    gap: 2.5rem;
    align-items: center;
    max-width: 1080px;
    margin: 0 auto;
    padding: 2rem 1.5rem;
    position: relative;
  }
  .acceso-profesor {
    margin-top: 0.8rem;
    width: 100%;
    justify-content: center;
    white-space: normal;
  }
  .logo {
    font-size: 3rem;
  }
  .portada h1 {
    font-size: 2rem;
  }
  .grupo {
    display: flex;
    flex-direction: column;
    gap: 0.1em;
    border-left: 4px solid var(--primario);
    padding: 0.4em 0.9em;
    background: var(--superficie);
    border-radius: 0 var(--radio-chico) var(--radio-chico) 0;
  }
  .pie {
    margin-top: 1.5em;
    font-size: 0.9em;
  }
  .pestanas {
    display: flex;
    gap: 0.25rem;
    border-bottom: 1px solid var(--borde);
    margin: -0.25rem -0.25rem 1rem;
    flex-wrap: wrap;
  }
  .pestanas button {
    border: none;
    border-bottom: 3px solid transparent;
    border-radius: 0;
    background: transparent;
    padding: 0.5em 0.7em;
    color: var(--texto-suave);
  }
  .pestanas button.activa {
    color: var(--primario);
    border-bottom-color: var(--primario);
    font-weight: 600;
  }
  .perfiles {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    margin-bottom: 1rem;
    max-height: 260px;
    overflow-y: auto;
  }
  .perfil {
    justify-content: flex-start;
    text-align: left;
    padding: 0.55em 0.75em;
  }
  .perfil.elegido {
    border-color: var(--primario);
    background: var(--primario-suave);
  }
  .avatar {
    width: 2.1em;
    height: 2.1em;
    border-radius: 50%;
    background: var(--primario);
    color: var(--primario-texto);
    display: grid;
    place-items: center;
    font-weight: 700;
    flex: none;
  }
  .datos {
    display: flex;
    flex-direction: column;
    line-height: 1.25;
  }
  .archivo {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 260px;
  }
  .codigo-recuperacion {
    font-family: var(--fuente-codigo);
    font-size: 1.35rem;
    letter-spacing: 0.06em;
    text-align: center;
    padding: 0.7em;
    background: var(--superficie-2);
    border-radius: var(--radio-chico);
    user-select: all;
  }
  .version {
    position: absolute;
    bottom: 0.5rem;
    right: 1.5rem;
    margin: 0;
    font-size: 0.8em;
  }
  @media (max-width: 860px) {
    .inicio {
      grid-template-columns: 1fr;
      gap: 1rem;
      padding: 1rem;
    }
    .version {
      position: static;
    }
  }
</style>
