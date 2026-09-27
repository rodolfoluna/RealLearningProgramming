# Contadores y acumuladores

Dos patrones aparecen en casi todos los programas con ciclos:

- Un **contador** cuenta cuántas veces pasa algo: empieza en 0 y suma 1.
- Un **acumulador** va juntando valores: empieza en 0 (o en 1 si multiplica) y suma cada valor.

```python
n = 5
suma = 0        # acumulador
i = 1           # contador
while i <= n:
    suma += i
    i += 1
print("La suma de 1 a", n, "es", suma)
```

Con ellos podemos calcular promedios de cualquier cantidad de datos:

```python
cantidad = 3
total = 0
i = 1
while i <= cantidad:
    total += 10 * i   # aquí normalmente pediríamos el dato con input()
    i += 1
print("Promedio:", total / cantidad)
```

## Resumen

- Inicializa contadores y acumuladores **antes** del ciclo.
- Actualízalos **dentro** del ciclo.
- Usa el resultado **después** del ciclo.
