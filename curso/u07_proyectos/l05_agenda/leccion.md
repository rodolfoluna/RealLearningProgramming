# Proyecto: agenda de contactos

Una agenda guarda nombres y teléfonos, permite buscarlos, verlos en orden y borrarlos.

## Los datos

Cada contacto es `[nombre, telefono]`. El teléfono se guarda como **texto**, no como número:
no vas a sumar teléfonos, y así no se pierden ceros a la izquierda.

```python
agenda = [["Ana López", "5512345678"], ["Luis Pérez", "3312345678"]]
```

## Buscar por una parte del nombre

Con `in` puedes saber si un texto contiene a otro; conviértelos a minúsculas para que no
importen las mayúsculas:

```python
if "ana" in "Ana López".lower():
    print("Coincide")
```

Una búsqueda puede encontrar **varios** contactos, así que la función devuelve una lista.

## Ordenar registros

`sorted()` también ordena listas de listas: compara el primer elemento de cada registro (y si
empatan, el segundo). Como el primer elemento es el nombre, queda en orden alfabético:

```python
for nombre, telefono in sorted(agenda):
    print(f"{nombre}: {telefono}")
```

## Eliminar

`del lista[i]` borra el elemento en la posición `i`. Busca primero la posición y luego bórralo.

## Resumen

- Guarda los teléfonos como texto y valídalos con `isdigit()` y `len()`.
- Una búsqueda que puede tener varios resultados devuelve una lista.
- `sorted(agenda)` ordena por nombre; `del agenda[i]` elimina.
