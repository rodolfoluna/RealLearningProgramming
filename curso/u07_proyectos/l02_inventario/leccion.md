# Proyecto: inventario de una tienda

Una papelería quiere saber qué productos tiene, venderlos y saber cuándo debe resurtir.

## Los datos

Cada producto es un registro `[nombre, cantidad, precio]`:

```python
inventario = [
    ["Cuaderno", 20, 35.0],
    ["Lápiz", 50, 6.5],
    ["Borrador", 4, 8.0],
]
```

Como los registros son listas, puedes **cambiar** la cantidad de un producto después de
encontrarlo:

```python
i = 1                      # posición del lápiz
inventario[i][1] -= 10     # vende 10 lápices
```

## Buscar sin importar mayúsculas

El usuario puede escribir `lápiz`, `Lápiz` o `LÁPIZ`. Compara ambos textos en minúsculas:

```python
if inventario[i][0].lower() == nombre.lower():
    ...
```

## Tablas en la consola

Recuerda los formatos de la unidad 5 para mostrar el inventario en columnas:

```python
print(f"{nombre:<12}{cantidad:>6}{precio:>10.2f}")
```

## Resumen

- Una lista de registros permite buscar un producto y modificar sus datos.
- Busca con una función que devuelva la **posición** (o `-1`).
- Cuida los casos de error: producto inexistente, cantidad insuficiente.
