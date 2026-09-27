# Proyecto: punto de venta

El proyecto final junta todo: un catálogo de productos, un carrito que crece con cada venta,
un ticket con formato y el cobro con cambio.

## El catálogo: una lista de tuplas

Los productos del catálogo no cambian durante la venta, así que cada uno es una **tupla**
`(codigo, nombre, precio)`:

```python
CATALOGO = [
    ("A1", "Refresco", 18.0),
    ("A2", "Papas", 15.5),
]
for codigo, nombre, precio in CATALOGO:
    print(codigo, nombre, precio)
```

## El carrito: una lista de listas

Cada venta agrega `[nombre, cantidad, precio]` al carrito.

## Devolver "nada"

Si una búsqueda no encuentra el producto puede devolver `None`, y quien la llama lo revisa:

```python
producto = buscar_en_catalogo(CATALOGO, codigo)
if producto is None:
    print("Código no encontrado")
```

## Devolver varios valores

Una función puede calcular varias cosas y devolverlas juntas en una tupla:

```python
subtotal, iva, total = totales(carrito)
```

## El ticket

Usa los formatos de la unidad 5 para alinear las columnas. Los textos a la izquierda y los
números a la derecha:

```texto
------------------------------
Refresco           2     36.00
Papas              1     15.50
------------------------------
Subtotal                 51.50
IVA                       8.24
Total                    59.74
```

¡Felicidades por llegar hasta aquí! Con lo que aprendiste ya puedes crear tus propios
programas. Recuerda que puedes convertir cualquiera en un ejecutable con **Crear .exe**.

## Resumen

- Tuplas para datos fijos (catálogo), listas para datos que cambian (carrito).
- `None` para decir "no encontrado"; tuplas para devolver varios valores.
- Formato de columnas para un ticket legible.
