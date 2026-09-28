# Recorrer y acumular

Con `for` puedes visitar cada elemento de una lista. Junto con los **acumuladores** y
**contadores** de la unidad 3, esto resuelve la mayoría de los problemas con listas.

```python
precios = [12.5, 30, 7.25]
total = 0
for precio in precios:
    total += precio
print(total)          # 49.75
```

## Contar los que cumplen algo

```python
calificaciones = [85, 60, 92, 70, 45]
aprobados = 0
for c in calificaciones:
    if c >= 70:
        aprobados += 1
print(aprobados)      # 3
```

## Encontrar el mayor (sin max)

Supón que el primero es el mayor y compáralo con los demás:

```python
temperaturas = [21, 25, 19, 30, 24]
mayor = temperaturas[0]
for t in temperaturas:
    if t > mayor:
        mayor = t
print(mayor)          # 30
```

> ¿Por qué no empezar con `mayor = 0`? Si todos los números fueran negativos, el resultado
> sería 0, ¡que ni siquiera está en la lista!

## Funciones de Python que ya hacen esto

Python trae `sum(lista)`, `max(lista)` y `min(lista)`. Úsalas en tus programas, pero aprende
a hacerlo con un ciclo: así podrás resolver problemas para los que no existe una función
lista (por ejemplo, "el alumno con la calificación más alta").

## Listas vacías

Antes de dividir entre `len(lista)` o de usar `lista[0]`, piensa qué pasa si la lista está
vacía:

```python
if len(numeros) == 0:
    promedio = 0
else:
    promedio = sum(numeros) / len(numeros)
```

## Resumen

- `for elemento in lista:` visita cada elemento.
- Acumulador para sumar, contador para contar, "el mejor hasta ahora" para buscar máximos.
- Cuida el caso de la lista vacía.
