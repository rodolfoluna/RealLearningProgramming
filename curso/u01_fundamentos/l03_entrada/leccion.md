# Pedir datos con input()

Hasta ahora nuestros programas siempre hacen lo mismo. Con `input()` el programa **pregunta**
algo al usuario y espera a que escriba y presione Enter:

```python
nombre = input("¿Cómo te llamas? ")
print("Mucho gusto,", nombre)
```

El texto entre paréntesis es el **mensaje** que se muestra. Deja un espacio al final para que
lo que escriba el usuario no quede pegado.

## input() siempre devuelve texto

Aunque el usuario escriba un número, `input()` lo entrega como **texto** (`str`). Si intentas
hacer cuentas con él, falla o da resultados raros:

```python
edad = input("Edad: ")
print(edad * 2)       # ¡repite el texto! "1515"
```

Para convertir el texto en número se usan `int()` (enteros) y `float()` (decimales):

```python
edad = int(input("Edad: "))
print("El próximo año tendrás", edad + 1)

precio = float(input("Precio: "))
print("Con IVA:", precio * 1.16)
```

| Función   | Convierte a        | Ejemplo                  |
|-----------|--------------------|--------------------------|
| `int()`   | número entero      | `int("42")` → `42`       |
| `float()` | número decimal     | `float("3.5")` → `3.5`   |
| `str()`   | texto              | `str(42)` → `"42"`       |

> Si el usuario escribe algo que no es número (por ejemplo `hola`), `int()` produce un
> `ValueError`. En la Unidad 2 aprenderás a manejar ese caso.

## Resumen

- `input("mensaje")` muestra el mensaje y devuelve lo que el usuario escribe, como texto.
- Convierte con `int()` o `float()` antes de hacer cálculos.
