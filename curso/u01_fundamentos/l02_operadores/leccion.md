# Operadores aritméticos

Python puede usarse como una calculadora muy potente:

| Operador | Significado          | Ejemplo    | Resultado |
|----------|----------------------|------------|-----------|
| `+`      | suma                 | `7 + 2`    | `9`       |
| `-`      | resta                | `7 - 2`    | `5`       |
| `*`      | multiplicación       | `7 * 2`    | `14`      |
| `/`      | división             | `7 / 2`    | `3.5`     |
| `//`     | división entera      | `7 // 2`   | `3`       |
| `%`      | residuo (módulo)     | `7 % 2`    | `1`       |
| `**`     | potencia             | `7 ** 2`   | `49`      |

```python
total = 250 * 3
print("Total:", total)
print(17 // 5, "sobran", 17 % 5)
```

- `/` **siempre** da un `float` (`6 / 2` da `3.0`).
- `//` da la parte entera y `%` lo que sobra. Son muy útiles para repartir cosas o para saber
  si un número es par (`n % 2 == 0`).

## Orden de las operaciones

Igual que en matemáticas: primero `**`, luego `*`, `/`, `//`, `%` y al final `+` y `-`.
Usa paréntesis para cambiar el orden o para que tu código se entienda mejor:

```python
print(2 + 3 * 4)
print((2 + 3) * 4)
```

## Asignación compuesta

Para modificar una variable con su propio valor hay atajos:

```python
saldo = 100
saldo += 50    # igual que saldo = saldo + 50
saldo -= 30    # igual que saldo = saldo - 30
print(saldo)
```

## Resumen

- `/` divide con decimales; `//` y `%` dan cociente y residuo enteros.
- Respeta la jerarquía de operaciones o usa paréntesis.
- `+=`, `-=`, `*=` modifican una variable con su propio valor.
