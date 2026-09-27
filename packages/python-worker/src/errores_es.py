"""Explicaciones en español, pensadas para principiantes, de los errores más comunes de Python.

Este módulo no depende de Pyodide: se usa igual en el navegador y en CPython
(validación del curso).
"""

import re

# (tipo de excepción o None para cualquiera, patrón sobre el mensaje, explicación)
# La explicación puede usar grupos nombrados del patrón con {nombre}.
_REGLAS = [
    # --- Sintaxis -----------------------------------------------------------
    ("SyntaxError", r"expected ':'",
     "Falta el signo ':' al final de la línea. Las instrucciones if, elif, else, while, for y def "
     "terminan con ':'."),
    ("SyntaxError", r"Maybe you meant '==' or ':=' instead of '='|cannot assign to .*here\. Maybe you meant '=='",
     "Usaste '=' (asignar) donde va '==' (comparar). Para preguntar si dos valores son iguales usa '=='."),
    ("SyntaxError", r"unterminated string literal|EOL while scanning string literal",
     "Un texto (cadena) no se cerró: falta la comilla de cierre \" o '."),
    ("SyntaxError", r"unterminated triple-quoted string",
     "Un texto con triple comilla (\"\"\") nunca se cerró."),
    ("SyntaxError", r"'(?P<abre>[\(\[\{])' was never closed",
     "Abriste '{abre}' pero nunca lo cerraste. Revisa que cada paréntesis o corchete tenga su pareja."),
    ("SyntaxError", r"unmatched '(?P<cierra>[\)\]\}])'",
     "Hay un '{cierra}' de más: no tiene su pareja de apertura."),
    ("SyntaxError", r"closing parenthesis '(?P<cierra>.)' does not match opening parenthesis '(?P<abre>.)'",
     "Cerraste con '{cierra}' pero lo que estaba abierto era '{abre}'."),
    ("SyntaxError", r"invalid syntax\. Perhaps you forgot a comma",
     "Parece que falta una coma ',' entre dos elementos (por ejemplo entre los argumentos de print)."),
    ("SyntaxError", r"Missing parentheses in call to 'print'",
     "En Python 3, print necesita paréntesis: print(\"Hola\")."),
    ("SyntaxError", r"invalid character '(?P<car>.)'",
     "Hay un carácter no válido: '{car}'. Suele pasar con comillas tipográficas (“ ”) o símbolos copiados; "
     "escribe las comillas normales \" o '."),
    ("SyntaxError", r"invalid decimal literal",
     "Un nombre de variable no puede empezar con número, y los números no llevan letras pegadas."),
    ("SyntaxError", r"cannot assign to literal|cannot assign to expression",
     "Del lado izquierdo del '=' debe ir el nombre de una variable, no un valor ni una operación."),
    ("SyntaxError", r"'return' outside function",
     "'return' solo puede usarse dentro de una función (def)."),
    ("SyntaxError", r"'break' outside loop|'continue' not properly in loop",
     "'break' y 'continue' solo pueden usarse dentro de un ciclo (while o for)."),
    ("SyntaxError", r"expected 'except' or 'finally' block",
     "Después de un bloque 'try:' debe venir 'except:' (o 'finally:')."),
    ("SyntaxError", r"invalid syntax",
     "Python no entiende esta línea. Revisa signos faltantes o sobrantes: ':' , paréntesis, comillas o comas."),
    # --- Sangría --------------------------------------------------------------
    ("IndentationError", r"expected an indented block",
     "Después de una línea que termina en ':' las siguientes líneas deben ir con sangría (4 espacios)."),
    ("IndentationError", r"unexpected indent",
     "Esta línea tiene sangría (espacios al inicio) que no corresponde. Quítale los espacios de más."),
    ("IndentationError", r"unindent does not match any outer indentation level",
     "La sangría de esta línea no coincide con la de las líneas anteriores. Usa siempre múltiplos de 4 espacios."),
    ("TabError", r".*",
     "Mezclaste tabuladores y espacios para la sangría. Usa solo espacios."),
    # --- Nombres ----------------------------------------------------------------
    ("NameError", r"name '(?P<nombre>\w+)' is not defined\. Did you mean: '(?P<sugerencia>\w+)'",
     "Usas '{nombre}' pero no existe. ¿Quisiste decir '{sugerencia}'?"),
    ("NameError", r"name '(?P<nombre>\w+)' is not defined",
     "Usas '{nombre}' pero no existe. Revisa que esté bien escrito (mayúsculas y minúsculas cuentan), "
     "que le hayas dado un valor antes de usarlo, o si era un texto que debía ir entre comillas."),
    ("UnboundLocalError", r".*'(?P<nombre>\w+)'.*",
     "Usas la variable '{nombre}' dentro de la función antes de darle un valor."),
    # --- Tipos ------------------------------------------------------------------
    ("TypeError", r'can only concatenate str \(not "(?P<tipo>\w+)"\) to str',
     "Intentas unir un texto con un valor de tipo {tipo} usando '+'. Convierte el valor con str() "
     "o usa una f-string: f\"Total: {{total}}\"."),
    ("TypeError", r"unsupported operand type\(s\) for (?P<op>\S+): '(?P<a>\w+)' and '(?P<b>\w+)'",
     "No se puede aplicar '{op}' entre un valor {a} y uno {b}. Recuerda que input() siempre devuelve "
     "texto (str): conviértelo con int() o float() antes de hacer cuentas."),
    ("TypeError", r"'(?P<op>[<>]=?)' not supported between instances of '(?P<a>\w+)' and '(?P<b>\w+)'",
     "No se puede comparar con '{op}' un valor {a} con uno {b}. ¿Olvidaste convertir el input() a número?"),
    ("TypeError", r"'str' object cannot be interpreted as an integer",
     "Se esperaba un número entero pero se recibió texto. Convierte con int()."),
    ("TypeError", r"'float' object cannot be interpreted as an integer",
     "Se esperaba un número entero pero se recibió un decimal (float). Usa int() o //."),
    ("TypeError", r"'(?P<tipo>\w+)' object is not callable",
     "Estás usando un valor de tipo {tipo} como si fuera una función (con paréntesis). ¿Usaste como variable "
     "un nombre de función, por ejemplo 'input' o 'print'?"),
    ("TypeError", r"'(?P<tipo>\w+)' object is not subscriptable",
     "Un valor de tipo {tipo} no se puede indexar con [ ]."),
    ("TypeError", r"(?P<func>\w+)\(\) missing (?P<n>\d+) required positional argument",
     "Llamaste a {func}() con menos argumentos de los que necesita."),
    ("TypeError", r"(?P<func>\w+)\(\) takes (?P<n>\d+) positional arguments? but (?P<m>\d+) (were|was) given",
     "{func}() recibe {n} argumento(s) pero le diste {m}."),
    ("TypeError", r"'(?P<tipo>\w+)' object does not support item assignment",
     "Un valor de tipo {tipo} no se puede modificar por posición (los textos y tuplas son inmutables)."),
    # --- Valores ----------------------------------------------------------------
    ("ValueError", r"invalid literal for int\(\) with base 10: (?P<valor>.*)",
     "int() no pudo convertir {valor} en número entero. Verifica que el dato sea un entero sin letras, "
     "espacios de más ni punto decimal."),
    ("ValueError", r"could not convert string to float: (?P<valor>.*)",
     "float() no pudo convertir {valor} en número decimal. Usa punto (.) como separador decimal."),
    ("ValueError", r"math domain error",
     "Operación matemática fuera de su dominio (por ejemplo, raíz cuadrada de un número negativo)."),
    ("ZeroDivisionError", r".*",
     "Estás dividiendo entre cero. Antes de dividir, verifica que el divisor no sea 0."),
    ("IndexError", r"(list|string|tuple) index out of range",
     "Usaste una posición (índice) que no existe. Recuerda que las posiciones empiezan en 0 y la última "
     "es len(...) - 1."),
    ("KeyError", r".*",
     "La llave que buscas no existe en el diccionario."),
    ("AttributeError", r"'(?P<tipo>\w+)' object has no attribute '(?P<attr>\w+)'",
     "Los valores de tipo {tipo} no tienen '{attr}'. Revisa cómo se escribe el método."),
    ("RecursionError", r".*",
     "La función se llama a sí misma demasiadas veces. Revisa el caso base."),
    ("EOFError", r".*",
     "El programa pidió un dato con input() pero ya no había más datos de entrada."),
    ("KeyboardInterrupt", r".*",
     "El programa fue detenido."),
    ("OverflowError", r".*",
     "El resultado es demasiado grande para calcularse."),
    ("ModuleNotFoundError", r"No module named '(?P<mod>[\w\.]+)'",
     "El módulo '{mod}' no está disponible en este curso."),
]

_COMPILADAS = [(t, re.compile(p), e) for t, p, e in _REGLAS]


def explicar(tipo, mensaje):
    """Devuelve una explicación en español para (tipo, mensaje) o una genérica."""
    mensaje = mensaje or ""
    for t, patron, explicacion in _COMPILADAS:
        if t is not None and t != tipo:
            continue
        m = patron.search(mensaje)
        if m:
            try:
                return explicacion.format(**m.groupdict())
            except (KeyError, IndexError):
                return explicacion
    if tipo == "SyntaxError":
        return "Hay un error de escritura (sintaxis) en esta línea."
    return f"Ocurrió un error de tipo {tipo}."
