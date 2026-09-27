# Crear listas y usar posiciones

Imagina guardar las calificaciones de 40 alumnos en 40 variables… Una **lista** guarda muchos
valores en una sola variable, en orden:

```python
calificaciones = [85, 92, 78, 100]
nombres = ["Ana", "Luis", "Sofía"]
vacia = []
mezcla = ["Ana", 20, True, 1.75]   # puede mezclar tipos, aunque casi nunca conviene
```

## Posiciones (índices)

Igual que en las cadenas, cada elemento tiene una **posición** que empieza en **0**, y los
índices negativos cuentan desde el final:

```python
nombres = ["Ana", "Luis", "Sofía"]
print(nombres[0])     # Ana
print(nombres[2])     # Sofía
print(nombres[-1])    # Sofía (el último)
print(len(nombres))   # 3
```

Si usas una posición que no existe (`nombres[3]`), Python muestra `IndexError`. Las posiciones
válidas van de `0` a `len(lista) - 1`.

## Las listas sí se pueden cambiar

A diferencia de las cadenas, puedes **reemplazar** un elemento:

```python
nombres[1] = "Luisa"
print(nombres)        # ['Ana', 'Luisa', 'Sofía']
```

## Usar una lista como tabla de consulta

Una lista sirve para convertir un número en un texto sin escribir muchos `if`:

```python
MESES = ["Enero", "Febrero", "Marzo", "Abril", "Mayo", "Junio",
         "Julio", "Agosto", "Septiembre", "Octubre", "Noviembre", "Diciembre"]
mes = int(input("Mes (1-12): "))
print(MESES[mes - 1])      # restamos 1 porque las posiciones empiezan en 0
```

> Las variables que no cambian se suelen escribir en MAYÚSCULAS para indicarlo.

## Resumen

- `[a, b, c]` crea una lista; `[]` crea una lista vacía.
- `lista[i]` lee o cambia el elemento en la posición `i` (desde 0); `lista[-1]` es el último.
- `len(lista)` dice cuántos elementos tiene.
