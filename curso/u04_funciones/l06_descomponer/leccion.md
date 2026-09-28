# Descomponer un programa en funciones

Los programas grandes se construyen **por partes**. Antes de escribir código, pregúntate:
¿qué tareas tiene que hacer mi programa? Cada tarea puede ser una función.

Por ejemplo, un programa que calcula el promedio de un alumno necesita:

1. Pedir una calificación válida (y repetir si el dato está mal) → `pedir_calificacion()`
2. Calcular el promedio → `promedio(a, b, c)`
3. Decidir si aprobó → `estado(prom)`

```python
def pedir_calificacion(mensaje):
    while True:
        try:
            valor = float(input(mensaje))
        except ValueError:
            print("Escribe un número.")
            continue
        if 0 <= valor <= 100:
            return valor
        print("Debe estar entre 0 y 100.")


def promedio(a, b, c):
    return (a + b + c) / 3


def estado(prom):
    if prom >= 70:
        return "Aprobado"
    return "Reprobado"


# Programa principal
c1 = pedir_calificacion("Parcial 1: ")
c2 = pedir_calificacion("Parcial 2: ")
c3 = pedir_calificacion("Parcial 3: ")
prom = promedio(c1, c2, c3)
print(f"Promedio: {prom:.1f} - {estado(prom)}")
```

Observa que el `return` dentro del `while True` termina la función (y el ciclo) en cuanto el
dato es válido.

## La estructura recomendada

1. `import` (si necesitas módulos).
2. Todas las **funciones**.
3. El **programa principal** al final.

> Las pruebas automáticas de este curso llaman a tus funciones una por una. Tu programa
> principal se detiene en su primer `input()`, así que **define las funciones antes** del
> programa principal.

## Ventajas

- Cada función es corta y fácil de entender.
- Puedes probar cada parte por separado (¡como lo hacen las pruebas!).
- Si algo falla, sabes dónde buscar.
- Puedes reutilizar funciones en otros programas.

## Resumen

- Divide el problema en tareas; cada tarea, una función con un nombre claro.
- Funciones arriba, programa principal abajo.
- Una función que valida datos puede repetir con `while True` y salir con `return`.
