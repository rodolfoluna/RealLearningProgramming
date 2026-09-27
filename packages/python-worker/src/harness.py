"""Arnés de ejecución del código del alumno.

Corre dentro de Pyodide (Web Worker) en las apps y en CPython para validar el curso.
Todas las funciones públicas devuelven JSON (str) para cruzar la frontera JS/Python sin
conversiones de tipos.
"""

import builtins
import contextlib
import copy
import io
import json
import linecache
import math
import re
import sys
import time
import traceback

import errores_es

ARCHIVO = "programa.py"
LIMITE_SALIDA_PRUEBA = 20_000

_config = {
    "notificar": None,  # callable(indice) antes de cada prueba (para armar timeouts)
    "reportar": None,  # callable(json) con el resultado de cada prueba al terminarla
}


def configurar(notificar=None, reportar=None, dormir=None, limpiar=None):
    """Instala ganchos del entorno anfitrión."""
    if notificar is not None:
        _config["notificar"] = notificar
    if reportar is not None:
        _config["reportar"] = reportar
    if dormir is not None:
        time.sleep = dormir
    if limpiar is not None:
        import os

        original = getattr(os, "system", None)

        def system(comando):
            if str(comando).strip().lower() in ("cls", "clear"):
                limpiar()
                return 0
            if original is not None:
                try:
                    return original(comando)
                except Exception:
                    return 1
            return 1

        os.system = system
    global _FLUJOS_ORIGINALES
    _FLUJOS_ORIGINALES = (sys.stdin, sys.stdout, sys.stderr)


# --------------------------------------------------------------------------- errores


def _registrar_fuente(codigo):
    linecache.cache[ARCHIVO] = (len(codigo), None, codigo.splitlines(True), ARCHIVO)


def _mensaje(exc):
    if isinstance(exc, SyntaxError):
        return exc.msg or ""
    try:
        ultima = traceback.format_exception_only(type(exc), exc)[-1].strip()
    except Exception:
        ultima = f"{type(exc).__name__}: {exc}"
    prefijo = type(exc).__name__ + ":"
    if ultima.startswith(prefijo):
        ultima = ultima[len(prefijo):].strip()
    elif ultima == type(exc).__name__:
        ultima = ""
    return ultima


def _sugerir_nombre(exc, mensaje):
    """Añade "Did you mean" (como Python 3.12+) cuando el intérprete no lo hizo."""
    import difflib

    m = re.search(r"name '(\w+)' is not defined", mensaje)
    tb = exc.__traceback__
    if not m or tb is None:
        return mensaje
    while tb.tb_next is not None:
        tb = tb.tb_next
    marco = tb.tb_frame
    candidatos = set(marco.f_locals) | set(marco.f_globals) | set(dir(builtins))
    cercanos = difflib.get_close_matches(m.group(1), [c for c in candidatos if not c.startswith("_")], n=1)
    if cercanos:
        return f"{mensaje}. Did you mean: '{cercanos[0]}'?"
    return mensaje


def info_error(exc, codigo):
    """Describe una excepción del programa del alumno (en español)."""
    tipo = type(exc).__name__
    mensaje = _mensaje(exc)
    linea = None
    columna = None
    marcos = []
    if isinstance(exc, SyntaxError):
        if exc.filename in (ARCHIVO, None, "<unknown>"):
            linea = exc.lineno
            columna = exc.offset
    else:
        for marco in traceback.extract_tb(exc.__traceback__):
            if marco.filename == ARCHIVO:
                linea = marco.lineno
                marcos.append(marco)
    if tipo == "NameError" and "Did you mean" not in mensaje:
        mensaje = _sugerir_nombre(exc, mensaje)
    lineas = codigo.splitlines()
    texto_linea = lineas[linea - 1] if linea and 0 < linea <= len(lineas) else ""
    traza = []
    for marco in marcos:
        donde = "el programa principal" if marco.name == "<module>" else f"la función {marco.name}()"
        traza.append(f"Línea {marco.lineno}, en {donde}: {(marco.line or '').strip()}")
    return {
        "tipo": tipo,
        "mensaje": mensaje,
        "linea": linea,
        "columna": columna,
        "texto_linea": texto_linea,
        "explicacion": errores_es.explicar(tipo, mensaje),
        "traza": traza,
    }


def _compilar(codigo):
    _registrar_fuente(codigo)
    return compile(codigo, ARCHIVO, "exec", dont_inherit=True)


def sintaxis(codigo):
    """Revisa la sintaxis sin ejecutar. Devuelve JSON con el error o "null"."""
    try:
        _compilar(codigo)
    except (SyntaxError, ValueError) as exc:
        if isinstance(exc, ValueError):  # p. ej. bytes nulos en la fuente
            exc = SyntaxError(str(exc))
        return json.dumps(info_error(exc, codigo), ensure_ascii=False)
    return "null"


_BUILTINS_ORIGINALES = dict(builtins.__dict__)
_FLUJOS_ORIGINALES = (sys.stdin, sys.stdout, sys.stderr)


def _restaurar_entorno():
    """Deshace cambios que un programa anterior pudo hacer a builtins o a sys."""
    for nombre in list(builtins.__dict__):
        if nombre not in _BUILTINS_ORIGINALES:
            del builtins.__dict__[nombre]
    builtins.__dict__.update(_BUILTINS_ORIGINALES)
    sys.stdin, sys.stdout, sys.stderr = _FLUJOS_ORIGINALES


def _espacio_nombres():
    _restaurar_entorno()
    return {"__name__": "__main__", "__builtins__": builtins}


# --------------------------------------------------------------------------- ejecución interactiva


def ejecutar(codigo):
    """Ejecuta el programa usando la entrada/salida estándar reales del anfitrión."""
    inicio = time.monotonic()
    resultado = {"estado": "ok", "error": None}
    try:
        objeto = _compilar(codigo)
        exec(objeto, _espacio_nombres())
    except SystemExit as exc:
        resultado["estado"] = "ok" if exc.code in (None, 0) else "salida"
        resultado["codigo_salida"] = exc.code if isinstance(exc.code, int) else None
    except KeyboardInterrupt:
        resultado["estado"] = "detenido"
    except BaseException as exc:  # noqa: BLE001 - reportamos cualquier error del alumno
        resultado["estado"] = "error"
        resultado["error"] = info_error(exc, codigo)
    finally:
        with contextlib.suppress(Exception):
            sys.stdout.flush()
            sys.stderr.flush()
        _restaurar_entorno()
    resultado["duracion_ms"] = int((time.monotonic() - inicio) * 1000)
    return json.dumps(resultado, ensure_ascii=False)


# --------------------------------------------------------------------------- comparación de salidas


def normalizar(texto):
    lineas = [ln.rstrip() for ln in texto.replace("\r\n", "\n").replace("\r", "\n").split("\n")]
    while lineas and not lineas[-1]:
        lineas.pop()
    while lineas and not lineas[0]:
        lineas.pop(0)
    return "\n".join(lineas)


def _laxo(texto):
    return re.sub(r"\s+", " ", texto).strip().casefold()


def _patron_fragmento(fragmento):
    base = re.escape(_laxo(fragmento))
    if re.match(r"-?\d", fragmento.strip()):
        base = r"(?<![\d.])" + base
    if re.search(r"\d$", fragmento.strip()):
        base = base + r"(?![\d])"
    return re.compile(base)


def comparar(obtenido, esperado, modo="contiene"):
    """Compara la salida del programa con la esperada. Devuelve (paso, mensaje)."""
    if modo == "contiene":
        fragmentos = esperado if isinstance(esperado, list) else [esperado]
        laxo = _laxo(obtenido)
        posicion = 0
        for fragmento in fragmentos:
            m = _patron_fragmento(str(fragmento)).search(laxo, posicion)
            if not m:
                if _patron_fragmento(str(fragmento)).search(laxo):
                    return False, f"«{fragmento}» aparece, pero no en el orden esperado."
                return False, f"Se esperaba ver «{fragmento}» en la salida."
            posicion = m.end()
        return True, "Correcto."
    if modo == "regex":
        patrones = esperado if isinstance(esperado, list) else [esperado]
        for patron in patrones:
            if not re.search(patron, obtenido, re.MULTILINE):
                return False, "La salida no tiene el formato esperado."
        return True, "Correcto."
    texto_esperado = "\n".join(esperado) if isinstance(esperado, list) else str(esperado)
    if modo == "termina":
        a = normalizar(obtenido).split("\n")
        b = normalizar(texto_esperado).split("\n")
        if len(a) >= len(b) and a[len(a) - len(b):] == b:
            return True, "Correcto."
        final = a[len(a) - len(b):] if len(a) >= len(b) else a
        for i, (la, lb) in enumerate(zip(final, b)):
            if la != lb:
                return False, f"Se esperaba la línea «{lb}» y tu programa mostró «{la}»."
        return False, "Las últimas líneas de la salida no son las esperadas."
    if modo == "exacta":
        a = obtenido.replace("\r\n", "\n")
        b = texto_esperado.replace("\r\n", "\n")
    else:  # normalizada
        a = normalizar(obtenido)
        b = normalizar(texto_esperado)
    if a == b:
        return True, "Correcto."
    lineas_a = a.split("\n")
    lineas_b = b.split("\n")
    for i in range(max(len(lineas_a), len(lineas_b))):
        la = lineas_a[i] if i < len(lineas_a) else None
        lb = lineas_b[i] if i < len(lineas_b) else None
        if la != lb:
            if la is None:
                return False, f"Faltan líneas: se esperaba la línea {i + 1}: «{lb}»."
            if lb is None:
                return False, f"Sobran líneas a partir de la línea {i + 1}: «{la}»."
            return False, f"La línea {i + 1} es diferente. Se esperaba «{lb}» y tu programa mostró «{la}»."
    return False, "La salida es diferente a la esperada."


def _iguales(a, b):
    if isinstance(a, bool) or isinstance(b, bool):
        return type(a) is type(b) and a == b
    if isinstance(a, (int, float)) and isinstance(b, (int, float)):
        return math.isclose(a, b, rel_tol=1e-9, abs_tol=1e-9)
    if isinstance(a, (list, tuple)) and isinstance(b, (list, tuple)):
        return len(a) == len(b) and all(_iguales(x, y) for x, y in zip(a, b))
    if isinstance(a, dict) and isinstance(b, dict):
        return a.keys() == b.keys() and all(_iguales(a[k], b[k]) for k in a)
    return a == b


# --------------------------------------------------------------------------- pruebas automáticas


class _SalidaLimitada(io.StringIO):
    def write(self, s):
        if self.tell() > LIMITE_SALIDA_PRUEBA:
            raise _DemasiadaSalida()
        return super().write(s)


class _DemasiadaSalida(Exception):
    pass


class _ProgramaDetenido(BaseException):
    """El programa principal pidió datos durante una prueba de función: ahí se detiene.

    Hereda de BaseException para que un `except Exception` del alumno no lo atrape.
    """


_ERROR_SALIDA_EXCESIVA = {
    "tipo": "SalidaExcesiva",
    "mensaje": "",
    "linea": None,
    "columna": None,
    "texto_linea": "",
    "explicacion": "Tu programa imprimió demasiado texto. ¿Hay un ciclo que nunca termina?",
    "traza": [],
}


class _EntradaSimulada:
    """Reemplazo de input(): toma las líneas de la prueba y lleva la cuenta de los ecos."""

    def __init__(self, entrada, salida):
        self.lineas = list(entrada.replace("\r\n", "\n").split("\n")) if entrada else []
        if self.lineas and self.lineas[-1] == "":
            self.lineas.pop()
        self.salida = salida
        self.ecos = []  # (posición en la salida, valor tecleado) para reconstruir la vista de consola
        self.detener = False  # True: input() detiene el programa (pruebas de función)

    def __call__(self, mensaje=""):
        self.salida.write(str(mensaje))
        if self.detener:
            raise _ProgramaDetenido()
        if not self.lineas:
            raise EOFError("Tu programa pidió más datos con input() de los que la prueba proporciona.")
        valor = self.lineas.pop(0)
        self.ecos.append((self.salida.tell(), valor))
        # En la salida "del programa" el dato tecleado no aparece, solo el salto de línea;
        # así una prueba no puede aprobarse con el eco de la entrada.
        self.salida.write("\n")
        return valor

    def consola(self):
        """La salida como se vería en la consola: con lo que se tecleó."""
        texto = self.salida.getvalue()
        for posicion, valor in reversed(self.ecos):
            texto = texto[:posicion] + valor + texto[posicion:]
        return texto


def _correr_capturando(codigo, entrada, detener_en_input=False):
    """Ejecuta el programa con entrada simulada.

    Con `detener_en_input`, el primer input() del programa principal lo detiene sin error
    (las pruebas de función solo necesitan las definiciones que hay antes).
    Devuelve (salida_programa, salida_consola, error, espacio_de_nombres, tiempo_agotado, detenido).
    """
    salida = _SalidaLimitada()
    ns = _espacio_nombres()
    entrada_simulada = _EntradaSimulada(entrada, salida)
    entrada_simulada.detener = detener_en_input
    ns["input"] = entrada_simulada
    error = None
    agotado = False
    detenido = False
    dormir_original = time.sleep
    time.sleep = lambda _segundos: None  # en las pruebas no se espera
    try:
        objeto = _compilar(codigo)
        with contextlib.redirect_stdout(salida), contextlib.redirect_stderr(salida):
            exec(objeto, ns)
    except SystemExit:
        pass
    except _ProgramaDetenido:
        detenido = True
    except KeyboardInterrupt:
        agotado = True
    except _DemasiadaSalida:
        error = dict(_ERROR_SALIDA_EXCESIVA)
    except BaseException as exc:  # noqa: BLE001
        error = info_error(exc, codigo)
    time.sleep = dormir_original
    _restaurar_entorno()
    return salida.getvalue(), entrada_simulada.consola(), error, ns, agotado, detenido


def _probar_io(codigo, prueba):
    entrada = prueba.get("entrada", "") or ""
    salida, consola, error, _, agotado, _ = _correr_capturando(codigo, entrada)
    modo = prueba.get("modo", "contiene")
    esperado = prueba.get("salida", "")
    resultado = {"tipo": "io", "entrada": entrada, "esperado": esperado, "obtenido": consola, "modo": modo}
    if agotado:
        return {**resultado, "paso": False, "tiempo_agotado": True,
                "mensaje": "Tu programa tardó demasiado. ¿Hay un ciclo que nunca termina?"}
    if error is not None:
        return {**resultado, "paso": False, "error": error,
                "mensaje": f"Tu programa terminó con un error ({error['tipo']}). {error['explicacion']}"}
    paso, mensaje = comparar(salida, esperado, modo)
    return {**resultado, "paso": paso, "mensaje": mensaje}


def _probar_funcion(codigo, prueba):
    """Llama a una función del alumno y revisa lo que devuelve y, si se pide, lo que imprime.

    Campos de la prueba: funcion, args, kwargs (argumentos con nombre), esperado (valor
    devuelto), entrada (datos para los input() dentro de la función), salida y modo (lo que la
    función debe imprimir). Sin `esperado` pero con `salida`, el valor devuelto no se revisa.
    """
    nombre = prueba["funcion"]
    args = copy.deepcopy(prueba.get("args", []))
    kwargs = copy.deepcopy(prueba.get("kwargs", {}))
    revisa_valor = "esperado" in prueba or "salida" not in prueba
    esperado = prueba.get("esperado")
    entrada = prueba.get("entrada", "") or ""
    modo = prueba.get("modo", "contiene")
    llamada = f"{nombre}({', '.join([repr(a) for a in args] + [f'{k}={v!r}' for k, v in kwargs.items()])})"
    resultado = {"tipo": "funcion", "llamada": llamada}
    if revisa_valor:
        resultado["esperado"] = repr(esperado)
    if entrada:
        resultado["entrada"] = entrada
    if "salida" in prueba:
        resultado["salida_esperada"] = prueba["salida"]
        resultado["modo"] = modo
    _, _, error, ns, agotado, detenido = _correr_capturando(codigo, "", detener_en_input=True)
    if agotado:
        return {**resultado, "paso": False, "tiempo_agotado": True,
                "mensaje": "Tu programa tardó demasiado. ¿Hay un ciclo que nunca termina?"}
    if error is not None:
        return {**resultado, "paso": False, "error": error,
                "mensaje": f"Tu programa terminó con un error ({error['tipo']}). {error['explicacion']}"}
    funcion = ns.get(nombre)
    if not callable(funcion):
        extra = (" Escribe tus funciones al principio, antes del programa principal que pide datos con input()."
                 if detenido else "")
        return {**resultado, "paso": False,
                "mensaje": f"No encontré la función {nombre}(). Revisa que la definas con def {nombre}(...):{extra}"}

    captura = _SalidaLimitada()
    entrada_simulada = _EntradaSimulada(entrada, captura)
    ns["input"] = entrada_simulada
    dormir_original = time.sleep
    time.sleep = lambda _segundos: None
    try:
        with contextlib.redirect_stdout(captura):
            obtenido = funcion(*args, **kwargs)
    except KeyboardInterrupt:
        return {**resultado, "paso": False, "tiempo_agotado": True,
                "mensaje": "Tu función tardó demasiado. ¿Hay un ciclo que nunca termina?"}
    except _DemasiadaSalida:
        return {**resultado, "paso": False, "error": dict(_ERROR_SALIDA_EXCESIVA),
                "mensaje": f"{llamada} imprimió demasiado texto. ¿Hay un ciclo que nunca termina?"}
    except BaseException as exc:  # noqa: BLE001
        err = info_error(exc, codigo)
        return {**resultado, "paso": False, "error": err,
                "mensaje": f"{llamada} produjo un error ({err['tipo']}). {err['explicacion']}"}
    finally:
        time.sleep = dormir_original
        _restaurar_entorno()

    resultado["obtenido"] = repr(obtenido)
    if "salida" in prueba:
        resultado["salida_obtenida"] = entrada_simulada.consola()
    if revisa_valor and not _iguales(obtenido, esperado):
        extra = ""
        if obtenido is None and esperado is not None:
            extra = " ¿Olvidaste usar return?" if not captura.getvalue() else \
                " ¿Usaste print en lugar de return? print muestra el valor, return lo devuelve."
        return {**resultado, "paso": False,
                "mensaje": f"{llamada} debía devolver {esperado!r} pero devolvió {obtenido!r}.{extra}"}
    if "salida" in prueba:
        paso, mensaje = comparar(captura.getvalue(), prueba["salida"], modo)
        if not paso:
            return {**resultado, "paso": False, "mensaje": f"{llamada} no mostró lo esperado. {mensaje}"}
    return {**resultado, "paso": True, "mensaje": "Correcto."}


def probar(codigo, pruebas_json):
    """Corre las pruebas de una actividad. `pruebas_json` es una lista JSON de pruebas."""
    pruebas = json.loads(pruebas_json)
    resultados = []
    notificar = _config["notificar"]
    for indice, prueba in enumerate(pruebas):
        if notificar is not None:
            notificar(indice)
        try:
            if "funcion" in prueba:
                r = _probar_funcion(codigo, prueba)
            else:
                r = _probar_io(codigo, prueba)
        except KeyboardInterrupt:
            r = {"tipo": "io", "paso": False, "tiempo_agotado": True,
                 "mensaje": "Tu programa tardó demasiado. ¿Hay un ciclo que nunca termina?"}
        r["indice"] = indice
        r["nombre"] = prueba.get("nombre") or f"Prueba {indice + 1}"
        r["oculta"] = bool(prueba.get("oculta", False))
        resultados.append(r)
        if _config["reportar"] is not None:
            _config["reportar"](json.dumps(r, ensure_ascii=False))
    if notificar is not None:
        notificar(-1)
    pasadas = sum(1 for r in resultados if r["paso"])
    return json.dumps({"pasadas": pasadas, "total": len(resultados), "resultados": resultados},
                      ensure_ascii=False)
