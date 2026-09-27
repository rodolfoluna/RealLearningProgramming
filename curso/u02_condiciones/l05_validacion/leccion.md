# Validar datos con try / except

¿Qué pasa si pedimos un número y el usuario escribe `hola`? `int("hola")` produce un error
`ValueError` y el programa se detiene. Podemos **atrapar** el error con `try` / `except`:

```python
try:
    edad = int(input("Edad: "))
    print("El próximo año tendrás", edad + 1)
except ValueError:
    print("Eso no es un número entero")
```

- Python intenta (`try`) ejecutar el bloque.
- Si ocurre un `ValueError`, salta al bloque `except` en lugar de detenerse.

Puedes atrapar distintos errores con varios `except`:

```python
try:
    a = float(input("Dividendo: "))
    b = float(input("Divisor: "))
    print("Resultado:", a / b)
except ValueError:
    print("Escribe solo números")
except ZeroDivisionError:
    print("No se puede dividir entre cero")
```

> Usa `try` solo alrededor del código que puede fallar y atrapa errores **específicos**
> (`ValueError`, `ZeroDivisionError`…), no todos a la vez.

## Resumen

- `try:` código que puede fallar; `except TipoDeError:` qué hacer si falla.
- Así tus programas no "truenan" cuando el usuario escribe algo inesperado.
