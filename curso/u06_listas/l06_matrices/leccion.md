# Listas de listas (matrices)

Una lista puede contener otras listas. Así se representan **tablas**: cada lista interna es una
fila.

```python
tabla = [
    [1, 2, 3],
    [4, 5, 6],
]
print(tabla[0])        # [1, 2, 3]   (la fila 0)
print(tabla[1][2])     # 6           (fila 1, columna 2)
print(len(tabla))      # 2 filas
print(len(tabla[0]))   # 3 columnas
```

Primero va la **fila** y después la **columna**: `tabla[fila][columna]`.

## Recorrer una matriz

Con dos ciclos anidados, como en las figuras de la unidad 3:

```python
for fila in tabla:
    for valor in fila:
        print(valor, end=" ")
    print()
```

Si necesitas las posiciones:

```python
for f in range(len(tabla)):
    for c in range(len(tabla[f])):
        print(f"tabla[{f}][{c}] = {tabla[f][c]}")
```

## Crear una matriz

```python
# Tablero de 3 × 3 vacío (con espacios)
tablero = []
for f in range(3):
    tablero.append([" ", " ", " "])

tablero[1][1] = "X"
```

> No uses `[[" "] * 3] * 3`: crea tres referencias a **la misma** fila y al cambiar una
> cambian todas.

## Registros: una fila por cada cosa

Otra forma muy útil: cada fila guarda los datos de **un** elemento.

```python
alumnos = [
    ["Ana", 85],
    ["Luis", 60],
]
for registro in alumnos:
    nombre, calificacion = registro     # desempacar
    print(f"{nombre}: {calificacion}")
alumnos.append(["Sofía", 92])
```

## Resumen

- `matriz[fila][columna]` accede a un valor.
- Dos `for` anidados recorren toda la matriz.
- Una lista de registros guarda varios datos por cada elemento.
