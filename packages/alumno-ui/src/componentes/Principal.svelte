<script lang="ts">
  import { Modal, mensajeError } from "@rlp/ui-comun";
  import { backend } from "../lib/backend";
  import { app, aplicarTema, avisar, cargarEstadoApp, curso, esWeb, exportarConNombre, fechaArchivo, puedeEscanearQr, salir } from "../lib/app.svelte";
  import Temario from "./Temario.svelte";
  import Bienvenida from "./Bienvenida.svelte";
  import Leccion from "./Leccion.svelte";
  import Actividad from "./Actividad.svelte";
  import Estadisticas from "./Estadisticas.svelte";
  import { ubicar } from "@rlp/curso";
  import { tick } from "svelte";

  const alumno = $derived(app.alumno!);
  const g = $derived(alumno.estadisticas.global);
  let menuAbierto = $state(false);
  let temarioAbierto = $state(false);

  // Exportar
  let exportando = $state(false);
  let exportado = $state("");
  let errorModal = $state("");

  // Cuenta
  let modalClave = $state(false);
  let actual = $state("");
  let nueva = $state("");
  let nueva2 = $state("");

  const seleccionActividad = $derived(
    app.seleccion.tipo === "actividad" ? ubicar(curso, app.seleccion.id) : null,
  );
  const seleccionLeccion = $derived(
    app.seleccion.tipo === "leccion"
      ? curso.unidades.flatMap((u) => u.lecciones.map((l) => ({ u, l }))).find((x) => x.l.id === (app.seleccion as { id: string }).id)
      : null,
  );

  async function exportar() {
    errorModal = "";
    menuAbierto = false;
    try {
      const b = await backend();
      if (exportarConNombre()) {
        const nombre = `${alumno.perfil.numero_control.replace(/[^\p{L}\p{N}]/gu, "")}_${fechaArchivo()}.rlp`;
        const destino = await b.elegirDestino("Guardar mi entrega", nombre);
        if (!destino) return;
        exportando = true;
        await b.exportarA(destino);
        exportado = nombre;
        return;
      }
      const carpeta = await b.elegirCarpeta("¿Dónde guardo tu entrega? (por ejemplo, tu memoria USB)");
      if (!carpeta) return;
      exportando = true;
      exportado = await b.exportar(carpeta);
    } catch (e) {
      errorModal = mensajeError(e);
      exportado = "error";
    } finally {
      exportando = false;
    }
  }

  async function importarAvances() {
    menuAbierto = false;
    try {
      const b = await backend();
      const r = await b.elegirArchivo("Elige el archivo .rlp de tu otro equipo", "rlp", "Avances de LP");
      if (!r) return;
      // Cierra la actividad abierta (guarda lo pendiente) para que el editor recargue lo importado.
      app.seleccion = { tipo: "inicio" };
      await tick();
      const res = await b.importarAvances(r);
      app.alumno = await b.estado();
      const partes = [`${res.eventos_nuevos} registros nuevos`, `${res.actividades_actualizadas.length} actividades actualizadas`];
      if (res.conflictos.length) partes.push(`${res.conflictos.length} conflicto(s)`);
      avisar(`Avances importados: ${partes.join(", ")}.`, 6000);
    } catch (e) {
      avisar(mensajeError(e), 6000);
    }
  }

  async function importarRetroalimentacion() {
    menuAbierto = false;
    try {
      const b = await backend();
      const r = await b.elegirArchivo("Elige el archivo de retroalimentación de tu profesor", "rlpr", "Retroalimentación de LP");
      if (!r) return;
      const retro = await b.importarRetroalimentacion(r);
      app.alumno = await b.estado();
      const n = Object.keys(retro.actividades).length;
      avisar(`Retroalimentación de ${retro.profesor}: ${n} actividad(es) con calificación o comentario.`, 6000);
    } catch (e) {
      avisar(mensajeError(e), 6000);
    }
  }

  async function unirseGrupo() {
    menuAbierto = false;
    try {
      const b = await backend();
      const r = await b.elegirArchivo("Elige el archivo de grupo (.rlpg)", "rlpg", "Grupo de LP");
      if (!r) return;
      const info = await b.unirseGrupo(r);
      app.alumno = await b.estado();
      await cargarEstadoApp();
      avisar(`Ahora perteneces al grupo ${info.nombre}.`);
    } catch (e) {
      avisar(mensajeError(e), 6000);
    }
  }

  async function unirseConQr() {
    menuAbierto = false;
    try {
      const b = await backend();
      const texto = await b.escanearQr();
      if (!texto) return;
      const info = await b.unirseGrupoQr(texto);
      app.alumno = await b.estado();
      await cargarEstadoApp();
      avisar(`Ahora perteneces al grupo ${info.nombre}.`);
    } catch (e) {
      avisar(mensajeError(e), 6000);
    }
  }

  async function cambiarClave() {
    errorModal = "";
    try {
      if (nueva !== nueva2) throw new Error("Las contraseñas nuevas no coinciden.");
      await (await backend()).cambiarContrasena(actual, nueva);
      modalClave = false;
      actual = nueva = nueva2 = "";
      avisar("Contraseña actualizada.");
    } catch (e) {
      errorModal = mensajeError(e);
    }
  }

  function ir(sel: typeof app.seleccion) {
    app.seleccion = sel;
    temarioAbierto = false;
  }
</script>

<div class="principal">
  <header class="barra">
    <button class="fantasma solo-movil" aria-label="Temario" onclick={() => (temarioAbierto = !temarioAbierto)}>☰</button>
    <button class="fantasma marca" onclick={() => ir({ tipo: "inicio" })}>
      <span aria-hidden="true">🐍</span> <span class="titulo">{curso.titulo}</span>
    </button>
    <span class="espaciador"></span>
    <button class="fantasma contadores" onclick={() => ir({ tipo: "estadisticas" })} title="Tus estadísticas (tu profesor también las ve)">
      <span class="insignia" data-contador="copias">📋 Copias <strong>{g.copias}</strong></span>
      <span class="insignia" class:alerta={g.pegados_intentos > 0} data-contador="pegados">🚫 Intentos de pegar <strong>{g.pegados_intentos}</strong></span>
      <span class="insignia" data-contador="salidas">↗ Salidas <strong>{g.salidas}</strong></span>
    </button>
    <div class="cuenta">
      <button class="fantasma" onclick={() => (menuAbierto = !menuAbierto)} aria-expanded={menuAbierto}>
        <span class="avatar">{alumno.perfil.nombre.slice(0, 1).toUpperCase()}</span>
        <span class="nombre">{alumno.perfil.nombre}</span> ▾
      </button>
      {#if menuAbierto}
        <div class="menu" role="menu">
          <div class="suave quien">{alumno.perfil.numero_control}{alumno.grupo ? ` · ${alumno.grupo.nombre}` : " · sin grupo"}</div>
          <button role="menuitem" onclick={exportar}>📤 Exportar entrega</button>
          <button role="menuitem" onclick={importarAvances}>📥 Importar avances de otro equipo</button>
          <button role="menuitem" onclick={importarRetroalimentacion}>📬 Importar retroalimentación del profesor</button>
          <button role="menuitem" onclick={() => { menuAbierto = false; ir({ tipo: "estadisticas" }); }}>📊 Mis estadísticas</button>
          <button role="menuitem" onclick={unirseGrupo}>👥 {alumno.grupo ? "Cambiar de grupo" : "Unirme a un grupo"}</button>
          {#if puedeEscanearQr()}
            <button role="menuitem" onclick={unirseConQr}>📷 Unirme con el QR del grupo</button>
          {/if}
          <button role="menuitem" onclick={() => { menuAbierto = false; modalClave = true; }}>🔑 Cambiar contraseña</button>
          <button role="menuitem" onclick={() => aplicarTema(app.tema === "oscuro" ? "claro" : "oscuro")}>
            🌓 Tema {app.tema === "oscuro" ? "claro" : "oscuro"}
          </button>
          <button role="menuitem" onclick={salir}>🚪 Cerrar sesión</button>
        </div>
      {/if}
    </div>
  </header>

  <div class="cuerpo">
    <aside class="lateral" class:abierto={temarioAbierto}>
      <Temario {ir} />
    </aside>
    <main class="contenido">
      {#if app.seleccion.tipo === "actividad" && seleccionActividad}
        {#key seleccionActividad.actividad.id}
          <Actividad actividad={seleccionActividad.actividad} leccion={seleccionActividad.leccion} {ir} />
        {/key}
      {:else if seleccionLeccion}
        {#key seleccionLeccion.l.id}
          <Leccion leccion={seleccionLeccion.l} unidad={seleccionLeccion.u} {ir} />
        {/key}
      {:else if app.seleccion.tipo === "estadisticas"}
        <Estadisticas />
      {:else}
        <Bienvenida {ir} />
      {/if}
    </main>
  </div>
</div>

<Modal titulo="Exportar entrega" abierto={exportando || !!exportado} cerrar={() => (exportado = "")}>
  {#if exportando}
    <p>Generando tu archivo…</p>
  {:else if exportado === "error"}
    <p class="error">{errorModal}</p>
  {:else}
    <p class="exito-msg">{esWeb() ? "Listo. Se descargó tu entrega (búscala en Descargas):" : "Listo. Tu entrega se guardó en:"}</p>
    <p class="ruta">{exportado}</p>
    <p class="suave">
      Entrega este archivo a tu profesor. También te sirve para continuar en otro equipo o en tu celular
      (opción "Tengo mis avances en un archivo"). Solo tú (con tu contraseña) y tu profesor pueden abrirlo.
    </p>
  {/if}
  {#snippet acciones()}
    <button class="primario" onclick={() => (exportado = "")} disabled={exportando}>Cerrar</button>
  {/snippet}
</Modal>

<Modal titulo="Cambiar contraseña" abierto={modalClave} cerrar={() => (modalClave = false)}>
  <form id="form-clave" onsubmit={(e) => { e.preventDefault(); cambiarClave(); }}>
    <div class="campo"><label for="ca">Contraseña actual</label><input id="ca" type="password" bind:value={actual} /></div>
    <div class="campo"><label for="cn">Contraseña nueva</label><input id="cn" type="password" bind:value={nueva} /></div>
    <div class="campo"><label for="cn2">Repite la contraseña nueva</label><input id="cn2" type="password" bind:value={nueva2} /></div>
    {#if errorModal}<p class="error">{errorModal}</p>{/if}
  </form>
  {#snippet acciones()}
    <button onclick={() => (modalClave = false)}>Cancelar</button>
    <button class="primario" type="submit" form="form-clave">Guardar</button>
  {/snippet}
</Modal>

<style>
  .principal {
    height: 100%;
    display: flex;
    flex-direction: column;
  }
  .barra {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.4rem 0.75rem;
    background: var(--superficie);
    border-bottom: 1px solid var(--borde);
    min-height: 52px;
  }
  .marca {
    font-weight: 700;
    font-size: 1.02rem;
  }
  .contadores {
    gap: 0.35rem;
    padding: 0.2em;
  }
  .insignia.alerta {
    background: var(--aviso-suave);
    color: var(--aviso);
    border-color: color-mix(in srgb, var(--aviso) 30%, transparent);
  }
  .cuenta {
    position: relative;
  }
  .avatar {
    width: 1.8em;
    height: 1.8em;
    border-radius: 50%;
    background: var(--primario);
    color: var(--primario-texto);
    display: inline-grid;
    place-items: center;
    font-weight: 700;
  }
  .menu {
    position: absolute;
    right: 0;
    top: calc(100% + 4px);
    z-index: 50;
    background: var(--superficie);
    border: 1px solid var(--borde);
    border-radius: var(--radio);
    box-shadow: 0 10px 30px rgb(0 0 0 / 0.18);
    padding: 0.35rem;
    min-width: 270px;
    display: flex;
    flex-direction: column;
  }
  .menu button {
    border: none;
    justify-content: flex-start;
  }
  .quien {
    padding: 0.3em 0.9em 0.5em;
    font-size: 0.88em;
    border-bottom: 1px solid var(--borde);
    margin-bottom: 0.25rem;
  }
  .cuerpo {
    flex: 1;
    display: flex;
    min-height: 0;
  }
  .lateral {
    width: 300px;
    flex: none;
    border-right: 1px solid var(--borde);
    background: var(--superficie);
    overflow-y: auto;
  }
  .contenido {
    flex: 1;
    min-width: 0;
    overflow: auto;
  }
  .ruta {
    font-family: var(--fuente-codigo);
    word-break: break-all;
    background: var(--superficie-2);
    padding: 0.5em;
    border-radius: var(--radio-chico);
  }
  .solo-movil {
    display: none;
  }
  @media (max-width: 1100px) {
    .titulo,
    .nombre {
      display: none;
    }
  }
  @media (max-width: 900px) {
    .solo-movil {
      display: inline-flex;
    }
    .contadores .insignia:not([data-contador="pegados"]) {
      display: none;
    }
    .lateral {
      position: fixed;
      top: 52px;
      bottom: 0;
      left: 0;
      z-index: 40;
      transform: translateX(-100%);
      transition: transform 0.2s;
      box-shadow: 0 0 30px rgb(0 0 0 / 0.25);
    }
    .lateral.abierto {
      transform: none;
    }
  }
</style>
