# Repetir con while

Un **ciclo** repite un bloque de instrucciones. `while` lo repite **mientras** una condición sea
verdadera:

```python
contador = 1
while contador <= 5:
    print("Vuelta", contador)
    contador += 1
print("Terminé")
```

Cómo funciona:

1. Revisa la condición (`contador <= 5`).
2. Si es verdadera, ejecuta el bloque y **vuelve al paso 1**.
3. Si es falsa, sigue con lo que está después del ciclo.

## Cuidado con los ciclos infinitos

Si nada dentro del ciclo hace que la condición se vuelva falsa, el ciclo **nunca termina**:

```python
n = 1
while n > 0:
    n += 1   # n siempre es positivo: ¡no termina!
```

Si te pasa, presiona **■ Detener** (o `Esc`). Las pruebas automáticas también cortan los
programas que tardan demasiado.

## Resumen

- `while condición:` repite mientras la condición sea verdadera.
- Asegúrate de que algo dentro del ciclo cambie la condición.
