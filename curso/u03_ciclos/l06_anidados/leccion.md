# Ciclos anidados y figuras

Un ciclo puede ir **dentro** de otro. Por cada vuelta del ciclo externo, el interno se ejecuta
completo:

```python
for fila in range(1, 4):
    for columna in range(1, 4):
        print(fila * columna, end=" ")
    print()
```

`print(..., end=" ")` evita el salto de línea; el `print()` vacío del final termina la fila.

## Dibujar con asteriscos

Recuerda que `"*" * 5` da `"*****"`. Con eso se pueden dibujar figuras fácilmente:

```python
altura = 4
for i in range(1, altura + 1):
    print("*" * i)
```

Y combinando espacios y asteriscos, figuras más elaboradas:

```python
altura = 3
for i in range(1, altura + 1):
    print(" " * (altura - i) + "#" * i)
```

## Resumen

- En ciclos anidados, el interno se repite completo por cada vuelta del externo.
- `end=" "` cambia lo que `print` pone al final.
- `texto * n` repite un texto n veces: útil para figuras.
