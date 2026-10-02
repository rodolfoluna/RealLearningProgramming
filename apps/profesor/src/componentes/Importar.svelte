<script lang="ts">
  import { mensajeError, Semaforo } from "@rlp/ui-comun";
  import { backend } from "../lib/backend";
  import { app, codigosIniciales } from "../lib/app.svelte";
  import type { ResultadoImportacion } from "../lib/tipos";

  let resultados: ResultadoImportacion[] = $state([]);
  let ocupado = $state(false);
  let error = $state("");

  async function importar(fn: () => Promise<ResultadoImportacion[] | null>) {
    error = "";
    ocupado = true;
    try {
      const r = await fn();
      if (r) resultados = r;
    } catch (e) {
      error = mensajeError(e);
    } finally {
      ocupado = false;
    }
  }

  const archivos = () =>
    importar(async () => {
      const b = await backend();
      const rutas = await b.elegirArchivos("Elige las entregas (.rlp)", "rlp", "Entregas de LP", true);
      return rutas.length ? b.importarEntregas(rutas, codigosIniciales) : null;
    });

  const carpeta = () =>
    importar(async () => {
      const b = await backend();
      const c = await b.elegirCarpeta("Elige la carpeta con las entregas (por ejemplo, tu memoria USB)");
      return c ? b.importarCarpeta(c, codigosIniciales) : null;
    });

  const nuevas = $derived(resultados.filter((r) => r.registro?.nueva).length);
</script>

<div class="importar">
  <h1>Importar entregas</h1>
  <p class="suave">
    Cada archivo se descifra con tus llaves y se verifica: firma de la app, integridad, historial encadenado y
    reconstrucción del código tecla a tecla. Las entregas repetidas se ignoran y se conserva el historial de cada alumno.
  </p>
  <div class="fila">
    <button class="primario" onclick={archivos} disabled={ocupado}>📄 Elegir archivos…</button>
    <button onclick={carpeta} disabled={ocupado}>📁 Importar una carpeta completa…</button>
    {#if ocupado}<span class="suave">Verificando…</span>{/if}
  </div>
  {#if error}<p class="error">{error}</p>{/if}

  {#if resultados.length}
    <p>{nuevas} entrega(s) nueva(s) de {resultados.length} archivo(s).</p>
    <table class="datos">
      <thead><tr><th>Archivo</th><th>Alumno</th><th>Integridad</th><th>Estado</th></tr></thead>
      <tbody>
        {#each resultados as r, i (i)}
          <tr>
            <td>{r.archivo}</td>
            <td>{r.registro ? `${r.registro.nombre} (${r.registro.numero_control})` : "—"}</td>
            <td>{#if r.registro}<Semaforo nivel={r.registro.nivel} />{/if}</td>
            <td>
              {#if r.error}
                <span class="mal">✖ {r.error}</span>
              {:else if r.registro?.nueva}
                ✔ Importada
                <button class="chico" onclick={() => (app.vista = { tipo: "detalle", entregaId: r.registro!.entrega_id })}>Ver</button>
              {:else}
                <span class="suave">Ya estaba importada</span>
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<style>
  .importar {
    padding: 1.2rem;
    max-width: 1100px;
  }
  .mal {
    color: var(--peligro);
    white-space: normal;
  }
  table {
    margin-top: 0.8rem;
  }
</style>
