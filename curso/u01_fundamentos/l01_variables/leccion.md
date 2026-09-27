# Variables y tipos de datos

Una **variable** es un nombre que guarda un valor para usarlo después. Se crea con el signo `=`
(se lee "toma el valor de"):

```python
nombre = "Ana"
edad = 17
estatura = 1.62
print(nombre, "tiene", edad, "años")
```

El valor de una variable puede **cambiar**. Python siempre usa el valor más reciente:

```python
puntos = 10
puntos = puntos + 5   # toma el valor anterior y le suma 5
print(puntos)
```

## Reglas para los nombres

- Pueden tener letras, números y guion bajo `_`, pero **no pueden empezar con número**.
- No llevan espacios ni acentos: usa `precio_total`, no `precio total`.
- Mayúsculas y minúsculas son distintas: `Edad` y `edad` son dos variables diferentes.
- Elige nombres que expliquen qué guardan: `promedio` es mejor que `x`.

## Tipos de datos básicos

| Tipo    | Qué guarda           | Ejemplos              |
|---------|----------------------|-----------------------|
| `int`   | números enteros      | `7`, `-3`, `2026`     |
| `float` | números con decimal  | `3.14`, `-0.5`, `2.0` |
| `str`   | texto (cadenas)      | `"Hola"`, `'123'`     |
| `bool`  | verdadero o falso    | `True`, `False`       |

La función `type()` te dice el tipo de un valor:

```python
print(type(25))
print(type(2.5))
print(type("25"))
print(type(True))
```

> Ojo: `"25"` (con comillas) es **texto**, no un número. Esa diferencia será muy importante cuando
> pidamos datos al usuario.

## Resumen

- `variable = valor` guarda un valor; se puede volver a asignar.
- Los tipos básicos son `int`, `float`, `str` y `bool`.
- Los nombres de variables deben ser claros y sin espacios.
