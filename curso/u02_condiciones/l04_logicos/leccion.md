# Operadores lógicos y condiciones anidadas

Para combinar condiciones se usan `and`, `or` y `not`:

| Operador | Es verdadero cuando…                    | Ejemplo                       |
|----------|------------------------------------------|-------------------------------|
| `and`    | **ambas** condiciones son verdaderas     | `edad >= 18 and tiene_ine`    |
| `or`     | **al menos una** es verdadera            | `dia == "sábado" or dia == "domingo"` |
| `not`    | invierte el valor                        | `not lloviendo`               |

```python
edad = 20
tiene_boleto = True
if edad >= 18 and tiene_boleto:
    print("Puedes entrar")
```

Para revisar si un valor está en un rango, Python permite encadenar comparaciones:

```python
nota = 85
if 0 <= nota <= 100:
    print("Calificación válida")
```

## Condiciones anidadas

Un `if` puede ir **dentro** de otro. La sangría indica a cuál pertenece cada bloque:

```python
usuario = "admin"
clave = "1234"
if usuario == "admin":
    if clave == "1234":
        print("Bienvenido")
    else:
        print("Contraseña incorrecta")
else:
    print("Usuario desconocido")
```

Muchas veces un `and` evita anidar, pero anidar es útil cuando cada caso da un mensaje distinto.

## Resumen

- `and`: todas; `or`: al menos una; `not`: lo contrario.
- `a <= x <= b` revisa un rango.
- Puedes poner un `if` dentro de otro cuidando la sangría.
