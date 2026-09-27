# Decisiones con if y else

`if` ejecuta un bloque de código **solo si** una condición es verdadera. `else` ejecuta otro
bloque cuando es falsa:

```python
edad = int(input("Edad: "))
if edad >= 18:
    print("Eres mayor de edad")
else:
    print("Eres menor de edad")
print("Fin del programa")
```

Fíjate en tres detalles:

1. La línea del `if` y la del `else` terminan con **dos puntos** `:`.
2. Las instrucciones del bloque llevan **sangría** (4 espacios). El editor la agrega sola al
   presionar Enter después de `:`.
3. Lo que ya no tiene sangría (`print("Fin del programa")`) se ejecuta siempre.

El `else` es opcional:

```python
temperatura = 35
if temperatura > 30:
    print("¡Hace calor! Toma agua.")
print("Que tengas buen día")
```

## Resumen

- `if condición:` + bloque con sangría.
- `else:` para el caso contrario (opcional).
- La sangría indica qué instrucciones pertenecen a cada bloque.
