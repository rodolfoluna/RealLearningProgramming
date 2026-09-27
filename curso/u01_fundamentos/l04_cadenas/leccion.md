# Cadenas de texto y f-strings

Las **cadenas** (`str`) son secuencias de caracteres. Python trae muchas herramientas para
trabajar con ellas.

## Unir y repetir

```python
nombre = "Ana"
print("Hola, " + nombre + "!")   # + une cadenas
print("-" * 20)                  # * repite
```

> `+` solo une **texto con texto**. Para unir un número, conviértelo con `str()` o usa f-strings.

## Longitud y posiciones

`len()` dice cuántos caracteres tiene una cadena. Cada carácter tiene una **posición** (índice)
que empieza en **0**; los índices negativos cuentan desde el final:

```python
palabra = "Python"
print(len(palabra))     # 6
print(palabra[0])       # P
print(palabra[-1])      # n
print(palabra[0:3])     # Pyt  (del 0 al 2)
```

## Métodos útiles

```python
texto = "  Hola Mundo  "
print(texto.upper())      # MAYÚSCULAS
print(texto.lower())      # minúsculas
print(texto.strip())      # quita espacios de los extremos
print(texto.replace("Mundo", "Python"))
```

## f-strings: la forma más cómoda de dar formato

Si pones una `f` antes de las comillas, puedes escribir variables y operaciones entre llaves `{}`:

```python
producto = "Cuaderno"
precio = 34.5
cantidad = 3
print(f"{cantidad} x {producto} = ${precio * cantidad}")
print(f"Total con dos decimales: ${precio * cantidad:.2f}")
```

El `:.2f` dentro de las llaves muestra el número con **2 decimales**.

## Resumen

- `+` une cadenas, `*` las repite, `len()` mide su longitud.
- Los índices empiezan en 0; `[-1]` es el último carácter.
- `f"...{variable}..."` inserta valores; `{valor:.2f}` muestra 2 decimales.
