<script lang="ts">
  import type { ResultadoPrueba, ResultadoPruebas } from "@rlp/python-worker";
  import ErrorPy from "./ErrorPython.svelte";

  let { resultado, enCurso, alIrALinea }: { resultado: ResultadoPruebas | null; enCurso: ResultadoPrueba[]; alIrALinea?: (l: number) => void } =
    $props();
  const lista = $derived(resultado?.resultados ?? enCurso);
  const texto = (v: unknown) => (Array.isArray(v) ? v.join(" … ") : String(v ?? ""));
</script>

<div class="resultados">
  {#if resultado}
    <p class="resumen" class:todo={resultado.pasadas === resultado.total}>
      {resultado.pasadas === resultado.total ? "🎉 ¡Todas las pruebas pasaron!" : `Pasaron ${resultado.pasadas} de ${resultado.total} pruebas.`}
    </p>
  {:else if enCurso.length}
    <p class="suave">Probando…</p>
  {:else}
    <p class="suave">Presiona <strong>✔ Probar</strong> (F6) para revisar tu programa con pruebas automáticas.</p>
  {/if}
  {#each lista as r (r.indice)}
    <details class="prueba" class:paso={r.paso} open={!r.paso && r === lista.find((x) => !x.paso)}>
      <summary>
        <span class="icono">{r.paso ? "✔" : "✖"}</span>
        <span>{r.nombre}</span>
        <span class="suave msg">{r.paso ? "" : r.mensaje}</span>
      </summary>
      <div class="detalle">
        {#if r.oculta && !r.paso}
          <p class="suave">Esta es una prueba oculta: revisa tu programa con otros datos.</p>
        {:else if r.tipo === "io"}
          {#if r.entrada}
            <div><span class="etq">Datos de entrada</span><pre>{r.entrada}</pre></div>
          {/if}
          <div><span class="etq">{r.modo === "contiene" ? "Se esperaba ver" : "Salida esperada"}</span><pre>{texto(r.esperado)}</pre></div>
          <div><span class="etq">Tu programa mostró</span><pre>{r.obtenido || "(nada)"}</pre></div>
        {:else}
          <div><span class="etq">Llamada</span><pre>{r.llamada}</pre></div>
          {#if r.entrada}
            <div><span class="etq">Datos de entrada</span><pre>{r.entrada}</pre></div>
          {/if}
          {#if r.esperado !== undefined}
            <div><span class="etq">Debe devolver</span><pre>{r.esperado}</pre></div>
            {#if r.obtenido !== undefined}<div><span class="etq">Tu función devolvió</span><pre>{r.obtenido}</pre></div>{/if}
          {/if}
          {#if r.salida_esperada !== undefined}
            <div><span class="etq">{r.modo === "contiene" ? "Debe mostrar" : "Salida esperada"}</span><pre>{texto(r.salida_esperada)}</pre></div>
            {#if r.salida_obtenida !== undefined}<div><span class="etq">Tu función mostró</span><pre>{r.salida_obtenida || "(nada)"}</pre></div>{/if}
          {/if}
        {/if}
        {#if !r.paso}<p class="mensaje">{r.mensaje}</p>{/if}
        {#if r.error}<ErrorPy error={r.error} {alIrALinea} />{/if}
      </div>
    </details>
  {/each}
</div>

<style>
  .resultados {
    padding: 0.6rem 0.8rem;
    overflow: auto;
    height: 100%;
  }
  .resumen {
    font-weight: 600;
    margin: 0 0 0.5rem;
    color: var(--aviso);
  }
  .resumen.todo {
    color: var(--exito);
  }
  .prueba {
    border: 1px solid var(--borde);
    border-radius: var(--radio-chico);
    margin-bottom: 0.4rem;
    background: var(--superficie);
  }
  summary {
    cursor: pointer;
    padding: 0.4em 0.7em;
    display: flex;
    gap: 0.5em;
    align-items: baseline;
  }
  .icono {
    color: var(--peligro);
    font-weight: 700;
  }
  .paso .icono {
    color: var(--exito);
  }
  .msg {
    font-size: 0.88em;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }
  .detalle {
    padding: 0 0.8em 0.7em;
    display: grid;
    gap: 0.4em;
  }
  .etq {
    font-size: 0.8em;
    font-weight: 600;
    color: var(--texto-suave);
  }
  pre {
    margin: 0.15em 0 0;
    background: var(--superficie-2);
    padding: 0.35em 0.6em;
    border-radius: 4px;
    white-space: pre-wrap;
    font-family: var(--fuente-codigo);
    font-size: 0.9em;
  }
  .mensaje {
    margin: 0.2em 0 0;
    color: var(--peligro);
  }
</style>
