<script lang="ts">
  import { fecha, mensajeError, Modal } from "@rlp/ui-comun";
  import { backend } from "../lib/backend";
  import { app, avisar } from "../lib/app.svelte";
  import type { GrupoInfo, NuevoGrupo } from "../lib/tipos";

  let nuevo = $state(false);
  let error = $state("");
  let datos: NuevoGrupo = $state({
    nombre: "",
    materia: "Fundamentos de Programación",
    periodo: "",
    regex_control: "\\d{8}",
    politicas: { pegado: "bloquear", registrar_salidas: true },
    coprofesores: [],
  });
  let coprofes = $state("");
  let qr = $state<{ grupo: GrupoInfo; svg: string } | null>(null);

  async function mostrarQr(g: GrupoInfo) {
    try {
      qr = { grupo: g, svg: await (await backend()).qrGrupo(g.grupo_id) };
    } catch (e) {
      avisar(mensajeError(e), 6000);
    }
  }

  async function crear() {
    error = "";
    try {
      const d = { ...datos, coprofesores: coprofes.split(/\s+/).filter(Boolean) };
      const g = await (await backend()).crearGrupo(d);
      app.grupos = await (await backend()).grupos();
      nuevo = false;
      avisar(`Grupo «${g.nombre}» creado. Ahora entrega el archivo de grupo a tus alumnos.`, 6000);
    } catch (e) {
      error = mensajeError(e);
    }
  }

  async function exportar(g: GrupoInfo) {
    try {
      const b = await backend();
      const carpeta = await b.elegirCarpeta("¿Dónde guardo el archivo de grupo?");
      if (carpeta) avisar(`Guardado en ${await b.exportarGrupo(g.grupo_id, carpeta)}`, 6000);
    } catch (e) {
      avisar(mensajeError(e), 6000);
    }
  }

  async function instalar(g: GrupoInfo) {
    try {
      const b = await backend();
      const carpeta = await b.elegirCarpeta("Elige la carpeta de la App Alumno (la que contiene LP Alumno.exe)");
      if (carpeta) avisar(`Grupo instalado en ${await b.instalarGrupo(g.grupo_id, carpeta)}`, 6000);
    } catch (e) {
      avisar(mensajeError(e), 6000);
    }
  }
</script>

<div class="grupos">
  <div class="fila">
    <h1>Grupos</h1>
    <span class="espaciador"></span>
    <button class="primario" onclick={() => (nuevo = true)}>＋ Nuevo grupo</button>
  </div>
  <p class="suave">
    Cada grupo tiene un archivo <code>.rlpg</code> firmado con tus llaves. Tus alumnos lo importan al abrir la App Alumno
    (o lo instalas tú en la carpeta de la app antes de copiarla a las computadoras). Así sus entregas quedan cifradas
    para ti.
  </p>
  {#if !app.grupos.length}
    <div class="tarjeta vacio">Aún no tienes grupos.</div>
  {/if}
  <div class="lista">
    {#each app.grupos as g (g.grupo_id)}
      <div class="tarjeta grupo">
        <div>
          <h2>{g.nombre}</h2>
          <p class="suave">{g.materia} · {g.periodo || "sin periodo"} · creado el {fecha(g.creado)}</p>
          <p class="suave">
            Pegado: <strong>{g.politicas.pegado === "bloquear" ? "bloqueado siempre" : "permitido solo código propio"}</strong> ·
            Salidas de ventana: <strong>{g.politicas.registrar_salidas ? "se registran" : "no se registran"}</strong> ·
            Número de control: <code>{g.regex_control || "cualquiera"}</code> ·
            Profesores con acceso: {g.llaves_cifrado.length}
          </p>
        </div>
        <div class="fila">
          <button onclick={() => exportar(g)}>💾 Guardar archivo de grupo</button>
          <button onclick={() => instalar(g)}>📁 Instalar en carpeta de la App Alumno</button>
          <button onclick={() => mostrarQr(g)}>📱 QR para celulares</button>
          <button onclick={() => ((app.grupoId = g.grupo_id), (app.vista = { tipo: "tablero" }))}>📊 Ver tablero</button>
        </div>
      </div>
    {/each}
  </div>
  {#if app.estado?.llave_cifrado}
    <p class="suave llave">Tu llave pública (compártela con otro profesor para que te agregue a sus grupos):<br /><code>{app.estado.llave_cifrado}</code></p>
  {/if}
</div>

<Modal titulo={qr ? `QR del grupo ${qr.grupo.nombre}` : "QR del grupo"} abierto={qr !== null} cerrar={() => (qr = null)} ancho="520px">
  {#if qr}
    <!-- SVG generado por la App Profesor (Rust) a partir del grupo firmado. -->
    <div class="qr" data-qr>{@html qr.svg}</div>
    <p class="suave">
      Proyecta este código en el salón. En la App Alumno del celular: <strong>📷 Escanear QR del grupo</strong> (o en el menú,
      "Unirme con el QR del grupo"). El código contiene el mismo grupo firmado que el archivo <code>.rlpg</code>.
    </p>
  {/if}
  {#snippet acciones()}
    <button class="primario" onclick={() => (qr = null)}>Cerrar</button>
  {/snippet}
</Modal>

<Modal titulo="Nuevo grupo" abierto={nuevo} cerrar={() => (nuevo = false)} ancho="600px">
  <form id="form-grupo" onsubmit={(e) => { e.preventDefault(); crear(); }}>
    <div class="campo"><label for="gn">Nombre del grupo</label><input id="gn" bind:value={datos.nombre} placeholder="Programación 1A" /></div>
    <div class="fila dos">
      <div class="campo"><label for="gm">Materia</label><input id="gm" bind:value={datos.materia} /></div>
      <div class="campo"><label for="gp">Periodo</label><input id="gp" bind:value={datos.periodo} placeholder="Ago–Dic 2026" /></div>
    </div>
    <div class="campo">
      <label for="gr">Formato del número de control (expresión regular)</label>
      <input id="gr" bind:value={datos.regex_control} />
      <p class="ayuda"><code>\d{"{8}"}</code> = 8 dígitos. Déjalo vacío para aceptar cualquiera.</p>
    </div>
    <div class="campo">
      <label for="gpg">Pegado en el editor</label>
      <select id="gpg" bind:value={datos.politicas.pegado}>
        <option value="bloquear">Bloqueado siempre (recomendado)</option>
        <option value="propio">Permitir pegar solo lo que el alumno copió de su propio editor</option>
      </select>
    </div>
    <label class="fila casilla"><input type="checkbox" bind:checked={datos.politicas.registrar_salidas} /> Registrar cuando el alumno sale de la ventana durante una actividad</label>
    <div class="campo">
      <label for="gc">Llaves de coprofesores (opcional)</label>
      <textarea id="gc" rows="2" bind:value={coprofes} placeholder="Pega aquí la llave pública de otro profesor"></textarea>
    </div>
    {#if error}<p class="error">{error}</p>{/if}
  </form>
  {#snippet acciones()}
    <button onclick={() => (nuevo = false)}>Cancelar</button>
    <button class="primario" type="submit" form="form-grupo">Crear grupo</button>
  {/snippet}
</Modal>

<style>
  .qr {
    display: flex;
    justify-content: center;
    background: #fff;
    padding: 1rem;
    border-radius: var(--radio);
  }
  .qr :global(svg) {
    width: min(360px, 80vw);
    height: auto;
  }
  .grupos {
    padding: 1.2rem;
    max-width: 1100px;
  }
  h1 {
    margin: 0;
  }
  .lista {
    display: grid;
    gap: 0.8rem;
  }
  .grupo h2 {
    margin: 0 0 0.2rem;
    font-size: 1.15rem;
  }
  .grupo p {
    margin: 0.2rem 0;
  }
  .vacio {
    text-align: center;
    color: var(--texto-suave);
  }
  .dos > .campo {
    flex: 1;
  }
  .casilla {
    font-weight: normal;
    margin-bottom: 1rem;
  }
  .casilla input {
    width: auto;
  }
  .llave code {
    word-break: break-all;
  }
</style>
