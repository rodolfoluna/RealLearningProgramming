# Devolver resultados con return

Hasta ahora tus funciones **mostraban** cosas. Muchas veces lo que necesitas es que la función
**calcule** un valor y te lo **devuelva** para seguir usándolo. Para eso está `return`.

```python
def doble(n):
    return n * 2

resultado = doble(21)
print(resultado)           # 42
print(doble(5) + 1)        # 11
if doble(3) > 5:
    print("Es mayor")
```

La llamada `doble(21)` **se convierte** en el valor devuelto: puedes guardarlo en una variable,
usarlo en una operación, en un `if` o como argumento de otra función.

## print muestra, return devuelve

Son cosas muy diferentes:

| `print(valor)` | `return valor` |
|---|---|
| Muestra el valor en la pantalla. | Entrega el valor a quien llamó a la función. |
| El programa **no** puede usar lo que se mostró. | El programa puede guardarlo y seguir calculando. |
| La función sigue ejecutándose. | La función **termina** en ese momento. |

Si una función no tiene `return`, devuelve el valor especial **`None`** ("nada"):

```python
def mostrar_doble(n):
    print(n * 2)

x = mostrar_doble(4)   # muestra 8
print(x)               # None
```

## Funciones que responden sí o no

Una función puede devolver `True` o `False`. Esas funciones se pueden usar directamente en un `if`:

```python
def es_mayor_de_edad(edad):
    return edad >= 18

if es_mayor_de_edad(20):
    print("Puede votar")
```

## return termina la función

En cuanto se ejecuta un `return`, la función termina. Puedes tener varios `return`, por ejemplo
uno en cada rama de un `if`:

```python
def signo(n):
    if n > 0:
        return "positivo"
    if n < 0:
        return "negativo"
    return "cero"
```

## Resumen

- `return valor` termina la función y entrega el valor.
- Sin `return`, una función devuelve `None`.
- Usa `return` cuando el resultado se va a **usar**; usa `print` solo para **mostrar**.
