"""Valida el curso compilado ejecutando las soluciones con el mismo arnés que usa la app.

- Cada `solucion` debe pasar todas sus pruebas.
- Cada `codigo_inicial` NO debe pasarlas todas (si no, el ejercicio ya viene resuelto).
- Cada predicción debe coincidir con la salida real de su código.

Uso: python scripts/validar_curso.py [ruta a curso-profesor.json]
"""

import contextlib
import io
import json
import signal
import sys
from pathlib import Path

RAIZ = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(RAIZ / "packages" / "python-worker" / "src"))

import harness  # noqa: E402

TIEMPO_POR_PRUEBA = 3


def _alarma(_signum, _frame):
    raise KeyboardInterrupt


def _notificar(indice):
    signal.alarm(TIEMPO_POR_PRUEBA if indice >= 0 else 0)


def probar(codigo, pruebas):
    return json.loads(harness.probar(codigo, json.dumps(pruebas)))


def salida_de(codigo):
    buf = io.StringIO()
    signal.alarm(TIEMPO_POR_PRUEBA)
    try:
        with contextlib.redirect_stdout(buf):
            exec(compile(codigo, "programa.py", "exec"), {"__name__": "__main__"})
    finally:
        signal.alarm(0)
    return buf.getvalue()


def main():
    ruta = Path(sys.argv[1]) if len(sys.argv) > 1 else RAIZ / "packages/curso/generado/curso-profesor.json"
    curso = json.loads(ruta.read_text(encoding="utf-8"))
    signal.signal(signal.SIGALRM, _alarma)
    harness.configurar(notificar=_notificar)
    errores = []
    total = 0
    for unidad in curso["unidades"]:
        for leccion in unidad["lecciones"]:
            for a in leccion["actividades"]:
                total += 1
                donde = f"{a['id']} ({a['titulo']})"
                if a["tipo"] == "codigo":
                    r = probar(a["solucion"], a["pruebas"])
                    for res in r["resultados"]:
                        if not res["paso"]:
                            errores.append(f"{donde}: la solución falla «{res['nombre']}»: {res['mensaje']}")
                    r = probar(a.get("codigo_inicial", ""), a["pruebas"])
                    if r["total"] and r["pasadas"] == r["total"]:
                        errores.append(f"{donde}: el código inicial ya pasa todas las pruebas")
                elif a["tipo"] == "prediccion":
                    real = salida_de(a["codigo"])
                    if harness.normalizar(real) != harness.normalizar(a["salida_esperada"]):
                        errores.append(f"{donde}: la salida esperada no coincide.\n  real: {real!r}\n  esperada: {a['salida_esperada']!r}")
    if errores:
        print("Errores en el curso:")
        for e in errores:
            print(" -", e)
        sys.exit(1)
    print(f"Curso válido: {total} actividades revisadas.")


if __name__ == "__main__":
    main()
