# Funciones matemáticas y redondeo

Python trae **módulos**: colecciones de herramientas listas para usar. El módulo `math` tiene
funciones y constantes matemáticas. Para usarlo, se **importa** al inicio del programa:

```python
import math

print(math.sqrt(49))     # raíz cuadrada
print(math.pi)           # π
print(math.pow(2, 10))   # potencia (da float)
```

## Redondear

- `round(x)` redondea al entero más cercano; `round(x, 2)` a dos decimales.
- `math.floor(x)` redondea hacia abajo y `math.ceil(x)` hacia arriba.
- `abs(x)` da el valor absoluto.

```python
import math

x = 7.456
print(round(x, 1), math.floor(x), math.ceil(x))
print(abs(-12))
```

## Un ejemplo completo

```python
import math

radio = 3
area = math.pi * radio ** 2
print(f"Un círculo de radio {radio} tiene un área de {area:.2f}")
```

## Resumen

- `import math` al inicio para usar `math.sqrt`, `math.pi`, `math.floor`, `math.ceil`…
- `round(x, n)` redondea a `n` decimales; `f"{x:.2f}"` solo cambia cómo se muestra.
