# Buscar y filtrar

## Buscar un elemento

`in` dice **si** un elemento está en la lista, y `index()` dice **dónde**. Pero `index()`
produce un error si el elemento no existe. Buscar con un ciclo te da más control:

```python
def buscar(lista, valor):
    for i in range(len(lista)):
        if lista[i] == valor:
            return i      # lo encontró: termina de inmediato
    return -1             # recorrió todo y no lo encontró
```

Devolver `-1` es una costumbre para decir "no está": una búsqueda nunca encuentra algo en una
posición negativa, así que no se confunde con un resultado real.

## Recorrer con posición y valor

`enumerate()` te da ambos a la vez:

```python
nombres = ["Ana", "Luis", "Sofía"]
for i, nombre in enumerate(nombres):
    print(i, nombre)
```

## Filtrar: construir una lista nueva

Para quedarte solo con los elementos que cumplen una condición, crea una lista vacía y agrega
los que te interesan:

```python
calificaciones = [85, 60, 92, 70, 45]
aprobadas = []
for c in calificaciones:
    if c >= 70:
        aprobadas.append(c)
print(aprobadas)       # [85, 92, 70]
```

La lista original no cambia. Esto es mejor que borrar elementos mientras recorres la lista,
lo cual produce errores difíciles de encontrar.

## Listas paralelas

A veces se usan dos listas con datos relacionados en las mismas posiciones:

```python
alumnos = ["Ana", "Luis", "Sofía"]
califs = [85, 60, 92]
for i in range(len(alumnos)):
    if califs[i] >= 70:
        print(alumnos[i], "aprobó")
```

## Resumen

- Buscar: recorre y devuelve la posición en cuanto lo encuentres; `-1` si no está.
- Filtrar: lista nueva + `append` de los que cumplen.
- `enumerate(lista)` da la posición y el valor.
