# Proyecto: gato (tres en raya)

Dos jugadores, `X` y `O`, se turnan para marcar casillas de un tablero de 3 × 3. Gana quien
complete una fila, una columna o una diagonal. Si se llena el tablero sin ganador, es empate.

## El tablero

Una matriz de 3 × 3 donde `" "` es una casilla libre:

```python
tablero = []
for f in range(3):
    tablero.append([" ", " ", " "])
```

El jugador escribe fila y columna del **1 al 3** (como las personas), y el programa resta 1
para usarlas como posiciones de la lista.

## ¿Quién ganó?

Hay 8 líneas posibles: 3 filas, 3 columnas y 2 diagonales. Para una fila `f`:

```python
if tablero[f][0] != " " and tablero[f][0] == tablero[f][1] == tablero[f][2]:
    return tablero[f][0]
```

Python permite encadenar comparaciones: `a == b == c` significa `a == b and b == c`.

## Validar la jugada

- La fila y la columna deben ser `"1"`, `"2"` o `"3"`. Cuidado: `"12" in "123"` es `True`
  porque revisa **texto dentro de texto**; usa una lista: `fila in ["1", "2", "3"]`.
- La casilla debe estar libre.

## Cambiar de turno

```python
if turno == "X":
    turno = "O"
else:
    turno = "X"
```

## Resumen

- El tablero es una matriz; `" "` marca las casillas libres.
- Revisa las 8 líneas para saber si hay ganador.
- Valida la jugada antes de marcar y cambia de turno solo si la jugada fue válida.
