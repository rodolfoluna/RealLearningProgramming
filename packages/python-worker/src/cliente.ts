// Cliente (hilo principal) del worker de Python.

import {
  CONTROL,
  type ErrorPython,
  type EventosEjecucion,
  type MensajeAlWorker,
  type MensajeDelWorker,
  type Prueba,
  type ResultadoEjecucion,
  type ResultadoPrueba,
  type ResultadoPruebas,
} from "./tipos";

/** Entrada síncrona servida por la app (para WebView sin SharedArrayBuffer). */
export interface PuenteEntrada {
  /** URL base (terminada en "/") del protocolo del puente. */
  url: string;
  enviar(linea: string): Promise<unknown>;
  cancelar(): Promise<unknown>;
}

export interface OpcionesEjecutor {
  /** URL (terminada en "/") donde están pyodide.mjs, pyodide.asm.wasm y python_stdlib.zip. */
  indexURL: string;
  /** Tiempo máximo por prueba automática. */
  timeoutPruebaMs?: number;
  /** Tiempo de gracia tras interrumpir antes de terminar el worker a la fuerza. */
  graciaMs?: number;
  /** Se usa cuando no hay SharedArrayBuffer. */
  puenteEntrada?: PuenteEntrada;
  /** Permite inyectar el worker (pruebas). */
  crearWorker?: () => Worker;
}

type Operacion =
  | { tipo: "ejecutar"; id: number; eventos: EventosEjecucion; resolver: (r: ResultadoEjecucion) => void }
  | { tipo: "probar"; id: number; resolver: (r: ResultadoPrueba | null) => void; resultado: ResultadoPrueba | null }
  | { tipo: "sintaxis"; id: number; resolver: (r: ErrorPython | null) => void };

const MENSAJE_TIEMPO = "Tu programa tardó demasiado. ¿Hay un ciclo que nunca termina?";

/**
 * Ejecuta código Python del alumno en un Web Worker con Pyodide. Solo corre una operación a la vez.
 */
export class EjecutorPython {
  private worker?: Worker;
  private listo?: Promise<void>;
  private readonly control: SharedArrayBuffer | null;
  private readonly interrupcion: SharedArrayBuffer | null;
  private readonly ctrl: Int32Array | null;
  private siguienteId = 1;
  private actual?: Operacion;
  private detenidoPorUsuario = false;
  private tiempoAgotado = false;
  private temporizadores: ReturnType<typeof setTimeout>[] = [];
  private cola: Promise<unknown> = Promise.resolve();

  version = "";
  esperandoEntrada = false;

  constructor(private readonly opciones: OpcionesEjecutor) {
    if (EjecutorPython.disponible()) {
      this.control = new SharedArrayBuffer(CONTROL.DATOS + CONTROL.TAMANO_DATOS);
      this.interrupcion = new SharedArrayBuffer(1);
      this.ctrl = new Int32Array(this.control, 0, 4);
    } else {
      this.control = this.interrupcion = this.ctrl = null;
    }
  }

  /** ¿El entorno permite memoria compartida? (si no, se usa el puente de entrada). */
  static disponible(): boolean {
    return typeof SharedArrayBuffer !== "undefined" && globalThis.crossOriginIsolated === true;
  }

  /** "memoria" (SharedArrayBuffer), "puente" (entrada vía la app) o "limitado" (sin input()). */
  get modo(): "memoria" | "puente" | "limitado" {
    return this.control ? "memoria" : this.opciones.puenteEntrada ? "puente" : "limitado";
  }

  get ocupado(): boolean {
    return this.actual !== undefined;
  }

  iniciar(): Promise<void> {
    if (!this.listo) this.listo = this.crear();
    return this.listo;
  }

  private crear(): Promise<void> {
    return new Promise((resolver, rechazar) => {
      const w = this.opciones.crearWorker
        ? this.opciones.crearWorker()
        : new Worker(new URL("./worker.ts", import.meta.url), { type: "module" });
      this.worker = w;
      w.onmessage = (e: MessageEvent<MensajeDelWorker>) => {
        const m = e.data;
        if (m.tipo === "listo") {
          this.version = m.version;
          resolver();
        } else if (m.tipo === "error_inicio") {
          this.listo = undefined;
          rechazar(new Error(`No se pudo iniciar Python: ${m.mensaje}`));
        } else {
          this.alMensaje(m);
        }
      };
      w.onerror = (e) => {
        this.listo = undefined;
        rechazar(new Error(`No se pudo iniciar Python: ${e.message}`));
      };
      const mensaje: MensajeAlWorker = {
        tipo: "iniciar",
        indexURL: this.opciones.indexURL,
        control: this.control ?? undefined,
        interrupcion: this.interrupcion ?? undefined,
        puente: this.control ? undefined : this.opciones.puenteEntrada?.url,
      };
      w.postMessage(mensaje);
    });
  }

  private alMensaje(m: MensajeDelWorker) {
    const op = this.actual;
    if (!op || !("id" in m) || m.id !== op.id) return;
    switch (m.tipo) {
      case "salida":
        if (op.tipo === "ejecutar") op.eventos.salida?.(m.texto, m.flujo);
        break;
      case "entrada":
        this.esperandoEntrada = true;
        if (op.tipo === "ejecutar") op.eventos.entradaSolicitada?.();
        break;
      case "limpiar":
        if (op.tipo === "ejecutar") op.eventos.limpiar?.();
        break;
      case "prueba_inicio":
        this.limpiarTemporizadores();
        if (m.indice >= 0 && op.tipo === "probar") this.armarTimeoutPrueba();
        break;
      case "prueba_fin":
        if (op.tipo === "probar") op.resultado = JSON.parse(m.resultado) as ResultadoPrueba;
        break;
      case "fin":
        this.terminarOperacion(m.resultado, m.salida_excedida);
        break;
    }
  }

  private terminarOperacion(json: string, salidaExcedida?: boolean) {
    const op = this.actual;
    if (!op) return;
    this.limpiarTemporizadores();
    this.actual = undefined;
    this.esperandoEntrada = false;
    if (op.tipo === "ejecutar") {
      const r = JSON.parse(json) as ResultadoEjecucion;
      if (salidaExcedida) {
        r.salida_excedida = true;
        r.estado = "detenido";
        r.error = null;
      } else if (this.detenidoPorUsuario && r.estado !== "ok") {
        r.estado = "detenido";
        r.error = null;
      }
      op.resolver(r);
    } else if (op.tipo === "probar") {
      const r = JSON.parse(json) as ResultadoPruebas;
      op.resolver(r.resultados[0] ?? op.resultado);
    } else {
      op.resolver(JSON.parse(json) as ErrorPython | null);
    }
  }

  private limpiarTemporizadores() {
    this.temporizadores.forEach(clearTimeout);
    this.temporizadores = [];
  }

  /** Pide al programa que se detenga (KeyboardInterrupt) si hay memoria compartida. */
  private interrumpir(): boolean {
    if (!this.interrupcion || !this.ctrl) return false;
    new Uint8Array(this.interrupcion)[0] = 2;
    Atomics.store(this.ctrl, CONTROL.ESTADO_ENTRADA, 2);
    Atomics.notify(this.ctrl, CONTROL.ESTADO_ENTRADA);
    Atomics.notify(this.ctrl, CONTROL.DORMIR);
    return true;
  }

  private armarTimeoutPrueba() {
    this.temporizadores.push(
      setTimeout(() => {
        this.tiempoAgotado = true;
        this.detenerAhora();
      }, this.opciones.timeoutPruebaMs ?? 4000),
    );
  }

  /** Interrumpe; si no responde (o no hay memoria compartida), termina el worker. */
  private detenerAhora() {
    if (this.interrumpir()) {
      this.temporizadores.push(setTimeout(() => this.forzarTerminacion(), this.opciones.graciaMs ?? 1500));
    } else {
      void this.opciones.puenteEntrada?.cancelar().catch(() => undefined);
      this.forzarTerminacion();
    }
  }

  private forzarTerminacion() {
    const op = this.actual;
    this.worker?.terminate();
    this.worker = undefined;
    this.listo = undefined;
    this.actual = undefined;
    this.esperandoEntrada = false;
    this.limpiarTemporizadores();
    void this.iniciar().catch(() => undefined);
    if (!op) return;
    if (op.tipo === "ejecutar") op.resolver({ estado: "detenido", error: null, duracion_ms: 0 });
    else if (op.tipo === "probar") op.resolver(op.resultado);
    else op.resolver(null);
  }

  private encolar<T>(fn: () => Promise<T>): Promise<T> {
    const p = this.cola.then(fn, fn);
    this.cola = p.catch(() => undefined);
    return p;
  }

  private correr<T>(op: (id: number, resolver: (v: T) => void) => Operacion, mensaje: (id: number) => MensajeAlWorker): Promise<T> {
    return new Promise<T>((resolver, rechazar) => {
      this.iniciar()
        .then(() => {
          const id = this.siguienteId++;
          if (this.interrupcion) new Uint8Array(this.interrupcion)[0] = 0;
          if (this.ctrl) Atomics.store(this.ctrl, CONTROL.ESTADO_ENTRADA, 0);
          this.actual = op(id, resolver);
          this.worker!.postMessage(mensaje(id));
        })
        .catch(rechazar);
    });
  }

  /** Ejecuta un programa de forma interactiva (input() pide datos a la interfaz). */
  ejecutar(codigo: string, eventos: EventosEjecucion = {}): Promise<ResultadoEjecucion> {
    return this.encolar(() => {
      this.detenidoPorUsuario = false;
      return this.correr<ResultadoEjecucion>(
        (id, resolver) => ({ tipo: "ejecutar", id, eventos, resolver }),
        (id) => ({ tipo: "ejecutar", id, codigo }),
      );
    });
  }

  /** Entrega una línea al input() que está esperando. */
  enviarEntrada(linea: string): void {
    if (!this.esperandoEntrada) return;
    this.esperandoEntrada = false;
    if (this.control && this.ctrl) {
      const bytes = new TextEncoder().encode(linea).slice(0, CONTROL.TAMANO_DATOS);
      new Uint8Array(this.control, CONTROL.DATOS, CONTROL.TAMANO_DATOS).set(bytes);
      Atomics.store(this.ctrl, CONTROL.LONGITUD, bytes.length);
      Atomics.store(this.ctrl, CONTROL.ESTADO_ENTRADA, 1);
      Atomics.notify(this.ctrl, CONTROL.ESTADO_ENTRADA);
    } else {
      void this.opciones.puenteEntrada?.enviar(linea);
    }
  }

  /** Detiene lo que se esté ejecutando. */
  detener(): void {
    if (!this.actual) return;
    this.detenidoPorUsuario = true;
    this.limpiarTemporizadores();
    this.detenerAhora();
  }

  /**
   * Corre las pruebas automáticas de una actividad, una por una: si una no termina, se corta
   * y se continúa con las demás.
   */
  probar(codigo: string, pruebas: Prueba[], alProgreso?: (r: ResultadoPrueba) => void): Promise<ResultadoPruebas> {
    return this.encolar(async () => {
      this.detenidoPorUsuario = false;
      const resultados: ResultadoPrueba[] = [];
      for (const [i, prueba] of pruebas.entries()) {
        const nombre = prueba.nombre ?? `Prueba ${i + 1}`;
        let r: ResultadoPrueba | null = null;
        if (!this.detenidoPorUsuario) {
          this.tiempoAgotado = false;
          r = await this.correr<ResultadoPrueba | null>(
            (id, resolver) => ({ tipo: "probar", id, resolver, resultado: null }),
            (id) => ({ tipo: "probar", id, codigo, pruebas: JSON.stringify([prueba]) }),
          );
        }
        const agotado = this.tiempoAgotado;
        const final: ResultadoPrueba = r
          ? { ...r, indice: i, nombre, oculta: !!prueba.oculta }
          : {
              indice: i,
              nombre,
              oculta: !!prueba.oculta,
              tipo: prueba.funcion ? "funcion" : "io",
              paso: false,
              tiempo_agotado: agotado,
              mensaje: agotado ? MENSAJE_TIEMPO : "No se ejecutó.",
            };
        resultados.push(final);
        alProgreso?.(final);
      }
      return { pasadas: resultados.filter((r) => r.paso).length, total: resultados.length, resultados };
    });
  }

  /** Revisa la sintaxis sin ejecutar. Si hay algo corriendo, no revisa (devuelve null). */
  async sintaxis(codigo: string): Promise<ErrorPython | null> {
    if (this.ocupado) return null;
    return this.encolar(() =>
      this.correr<ErrorPython | null>(
        (id, resolver) => ({ tipo: "sintaxis", id, resolver }),
        (id) => ({ tipo: "sintaxis", id, codigo }),
      ),
    );
  }

  destruir(): void {
    this.worker?.terminate();
    this.worker = undefined;
    this.listo = undefined;
  }
}
