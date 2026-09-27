"""Pruebas del arnés (harness.py) en CPython.

Uso: python3 -m unittest discover packages/python-worker/pruebas
"""

import json
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "src"))

import harness  # noqa: E402


def probar(codigo, *pruebas):
    return json.loads(harness.probar(codigo, json.dumps(list(pruebas))))["resultados"]


PROGRAMA = '''
def saludar(nombre, saludo="Hola"):
    print(f"{saludo}, {nombre}")

def doble(x):
    return x * 2

def mal_doble(x):
    print(x * 2)

def pedir_edad():
    return int(input("Edad: "))

# Programa principal con menú: las pruebas de función se detienen en el primer input().
while True:
    opcion = input("Opción: ")
    if opcion == "salir":
        break
    print("Elegiste", opcion)

def definida_despues():
    return 1
'''


class PruebasDeFuncion(unittest.TestCase):
    def test_valor_devuelto(self):
        (r,) = probar(PROGRAMA, {"funcion": "doble", "args": [21], "esperado": 42})
        self.assertTrue(r["paso"], r["mensaje"])
        self.assertEqual(r["llamada"], "doble(21)")

    def test_salida_de_la_funcion(self):
        ok, mal = probar(
            PROGRAMA,
            {"funcion": "saludar", "args": ["Ana"], "salida": "Hola, Ana"},
            {"funcion": "saludar", "args": ["Ana"], "salida": "Adiós, Ana"},
        )
        self.assertTrue(ok["paso"], ok["mensaje"])
        self.assertEqual(ok["salida_obtenida"], "Hola, Ana\n")
        self.assertNotIn("esperado", ok)  # solo se pidió revisar la salida
        self.assertFalse(mal["paso"])
        self.assertIn("no mostró lo esperado", mal["mensaje"])

    def test_argumentos_con_nombre(self):
        (r,) = probar(PROGRAMA, {"funcion": "saludar", "args": ["Luis"], "kwargs": {"saludo": "Buenas"},
                                 "salida": "Buenas, Luis"})
        self.assertTrue(r["paso"], r["mensaje"])
        self.assertEqual(r["llamada"], "saludar('Luis', saludo='Buenas')")

    def test_entrada_dentro_de_la_funcion(self):
        (r,) = probar(PROGRAMA, {"funcion": "pedir_edad", "entrada": "17\n", "esperado": 17, "salida": "Edad:"})
        self.assertTrue(r["paso"], r["mensaje"])
        self.assertEqual(r["salida_obtenida"], "Edad: 17\n")

    def test_faltan_datos_de_entrada(self):
        (r,) = probar(PROGRAMA, {"funcion": "pedir_edad", "esperado": 17})
        self.assertFalse(r["paso"])
        self.assertEqual(r["error"]["tipo"], "EOFError")

    def test_print_en_lugar_de_return(self):
        (r,) = probar(PROGRAMA, {"funcion": "mal_doble", "args": [3], "esperado": 6})
        self.assertFalse(r["paso"])
        self.assertIn("print en lugar de return", r["mensaje"])

    def test_funcion_despues_del_programa_principal(self):
        (r,) = probar(PROGRAMA, {"funcion": "definida_despues", "esperado": 1})
        self.assertFalse(r["paso"])
        self.assertIn("antes del programa principal", r["mensaje"])

    def test_los_argumentos_no_se_comparten_entre_pruebas(self):
        codigo = "def agrega(lista):\n    lista.append(1)\n    return lista\n"
        prueba = {"funcion": "agrega", "args": [[0]], "esperado": [0, 1]}
        a, b = probar(codigo, prueba, prueba)
        self.assertTrue(a["paso"] and b["paso"])

    def test_except_del_alumno_no_atrapa_la_detencion(self):
        codigo = (
            "def f():\n    return 5\n"
            "try:\n    n = input()\nexcept Exception:\n    print('atrapado')\n"
        )
        (r,) = probar(codigo, {"funcion": "f", "esperado": 5})
        self.assertTrue(r["paso"], r["mensaje"])


class PruebasDeEntradaSalida(unittest.TestCase):
    def test_programa_completo_con_menu(self):
        (r,) = probar(PROGRAMA, {"entrada": "1\nsalir\n", "salida": "Elegiste 1"})
        self.assertTrue(r["paso"], r["mensaje"])
        self.assertIn("Opción: 1", r["obtenido"])  # la vista de consola incluye lo tecleado

    def test_el_eco_no_aprueba_la_prueba(self):
        (r,) = probar("x = input('Dato: ')\n", {"entrada": "42\n", "salida": "42"})
        self.assertFalse(r["paso"])


if __name__ == "__main__":
    unittest.main()
