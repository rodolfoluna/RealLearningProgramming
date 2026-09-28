# Métodos de las cadenas

Un **método** es una función que pertenece a un valor y se llama con un punto:
`texto.upper()`. Las cadenas tienen muchos. Recuerda: como las cadenas no se modifican, los
métodos **devuelven una cadena nueva**; si quieres conservar el cambio, guárdalo.

```python
nombre = "ana"
nombre.upper()          # no cambia nombre
nombre = nombre.upper() # ahora sí: "ANA"
```

## Mayúsculas y espacios

| Método | Ejemplo | Resultado |
|---|---|---|
| `upper()` | `"Hola".upper()` | `"HOLA"` |
| `lower()` | `"Hola".lower()` | `"hola"` |
| `title()` | `"ana lópez".title()` | `"Ana López"` |
| `capitalize()` | `"hola MUNDO".capitalize()` | `"Hola mundo"` |
| `strip()` | `"  hola  ".strip()` | `"hola"` |

## Buscar y reemplazar

| Método | Ejemplo | Resultado |
|---|---|---|
| `count(x)` | `"banana".count("a")` | `3` |
| `find(x)` | `"banana".find("n")` | `2` (posición; `-1` si no está) |
| `replace(a, b)` | `"banana".replace("a", "o")` | `"bonono"` |
| `startswith(x)` | `"python".startswith("py")` | `True` |
| `endswith(x)` | `"foto.png".endswith(".png")` | `True` |

## Preguntar qué contiene

| Método | Es `True` si… |
|---|---|
| `isdigit()` | todos los caracteres son dígitos (`"2024"`) |
| `isalpha()` | todos son letras (`"Ana"`) |
| `isupper()` / `islower()` | todas las letras son mayúsculas / minúsculas |
| `isspace()` | todos son espacios |

Estos métodos funcionan también con un solo carácter, por ejemplo `letra.isdigit()`.

## Separar y unir

`split()` separa un texto en **partes** (por defecto, en los espacios) y `join()` las une.
Las partes forman una **lista**, que verás a detalle en la siguiente unidad:

```python
palabras = "hola   mundo  cruel".split()
print(palabras)            # ['hola', 'mundo', 'cruel']
print(len(palabras))       # 3
for p in palabras:
    print(p[0])            # h, m, c

print("-".join(palabras))  # hola-mundo-cruel
print("2024-05-10".split("-"))   # ['2024', '05', '10']
```

Un truco útil: `" ".join(texto.split())` quita los espacios repetidos.

## Resumen

- Los métodos se llaman con punto y **devuelven** una cadena nueva.
- `split()` separa en palabras; `"sep".join(partes)` las une.
- `isdigit()`, `isalpha()`, `startswith()`… responden `True` o `False`.
