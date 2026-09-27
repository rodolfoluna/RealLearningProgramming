# Recorrer con for y range()

Cuando sabemos **cuántas veces** repetir, `for` es más cómodo que `while`. `range()` genera
una secuencia de números:

```python
for i in range(5):
    print("Vuelta", i)
```

| Llamada             | Números que genera       |
|---------------------|--------------------------|
| `range(5)`          | 0, 1, 2, 3, 4            |
| `range(1, 6)`       | 1, 2, 3, 4, 5            |
| `range(0, 11, 2)`   | 0, 2, 4, 6, 8, 10        |
| `range(10, 0, -1)`  | 10, 9, 8, …, 1           |

> El segundo número **no se incluye**: `range(1, 6)` llega hasta 5.

`for` también recorre cadenas, letra por letra:

```python
for letra in "Hola":
    print(letra)
```

El mismo acumulador de la lección anterior, ahora con `for`:

```python
suma = 0
for i in range(1, 101):
    suma += i
print(suma)
```

## Resumen

- `for variable in range(inicio, fin, paso):` repite un número conocido de veces.
- `fin` no se incluye; `paso` puede ser negativo.
