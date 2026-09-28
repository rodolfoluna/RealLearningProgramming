# Definir y llamar funciones

Una **función** es un bloque de código con nombre que puedes ejecutar cuantas veces quieras.
Ya usaste muchas: `print()`, `input()`, `len()`, `int()`… Ahora vas a crear las tuyas.

```python
def saludar():
    print("¡Hola!")
    print("Bienvenido al curso")

saludar()
saludar()
```

- `def` indica que vas a **definir** una función; después va su nombre, paréntesis y dos puntos.
- Las instrucciones de la función (su **cuerpo**) llevan **sangría**, igual que en `if` y `while`.
- **Definir no ejecuta**: el cuerpo solo corre cuando **llamas** a la función escribiendo su
  nombre con paréntesis: `saludar()`.

## ¿Para qué sirven?

- **No repetir código**: si algo se hace varias veces, escríbelo una vez en una función.
- **Poner nombre a una idea**: `dibujar_linea()` se entiende mejor que el `print` que lleva dentro.
- **Probar por partes**: en este curso, muchas pruebas automáticas llaman a tus funciones
  directamente para revisar que cada una haga bien su trabajo.

## El orden importa

Python lee tu programa de arriba hacia abajo. Una función debe estar **definida antes** de
llamarla; si no, verás un `NameError`.

```python
linea()          # NameError: todavía no existe

def linea():
    print("-" * 20)
```

Por eso la costumbre es escribir **primero las funciones** y **al final el programa principal**.

## Nombres de funciones

Usa minúsculas y guiones bajos (`calcular_total`, `mostrar_menu`). Como una función **hace**
algo, suele ayudar que su nombre sea un verbo.

## Resumen

- `def nombre():` define una función; su cuerpo va con sangría.
- La función se ejecuta cada vez que la llamas: `nombre()`.
- Escribe las funciones arriba y el programa principal abajo.
