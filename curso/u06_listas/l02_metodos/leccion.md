# Métodos de las listas

Las listas pueden crecer, encogerse y ordenarse. A diferencia de las cadenas, estos métodos
**modifican la lista** directamente.

## Agregar

```python
compras = ["pan"]
compras.append("leche")        # al final → ['pan', 'leche']
compras.insert(0, "huevos")    # en una posición → ['huevos', 'pan', 'leche']
```

Un patrón muy común: empezar con una lista vacía y llenarla en un ciclo.

```python
numeros = []
for i in range(3):
    numeros.append(int(input("Número: ")))
```

## Quitar

```python
compras.remove("pan")    # quita el primer "pan" (error si no existe)
ultimo = compras.pop()   # quita y devuelve el último
primero = compras.pop(0) # quita y devuelve el de la posición 0
```

## Preguntar

```python
print("leche" in compras)        # True o False
print(compras.count("pan"))      # cuántas veces aparece
print(compras.index("leche"))    # posición (error si no existe)
print(len(compras))
```

## Ordenar

```python
numeros = [5, 2, 9, 1]
numeros.sort()                 # la lista queda ordenada: [1, 2, 5, 9]
numeros.sort(reverse=True)     # de mayor a menor
otra = sorted([3, 1, 2])       # sorted() devuelve una lista NUEVA ordenada
```

> Cuidado: `sort()` ordena la lista y devuelve `None`. Escribir `numeros = numeros.sort()`
> ¡pierde la lista!

## Recorrer para mostrar

```python
for producto in compras:
    print("-", producto)

for i in range(len(compras)):
    print(f"{i + 1}. {compras[i]}")
```

## Resumen

- `append`, `insert` agregan; `remove`, `pop` quitan.
- `in`, `count`, `index`, `len` consultan.
- `sort()` ordena la misma lista; `sorted()` devuelve una nueva.
