# Valores por defecto y argumentos con nombre

## Parámetros con valor por defecto

Puedes darle a un parámetro un valor que se usará **si no se envía** ese argumento:

```python
def saludar(nombre, saludo="Hola"):
    print(f"{saludo}, {nombre}")

saludar("Ana")                 # Hola, Ana
saludar("Luis", "Buenos días") # Buenos días, Luis
```

Los parámetros con valor por defecto van **al final**. `def f(a=1, b):` es un error de sintaxis.

## Argumentos con nombre

Al llamar una función puedes indicar **a qué parámetro** va cada valor. Así no importa el orden
y el código se lee mejor:

```python
def precio_final(precio, descuento=0, iva=0.16):
    ...

precio_final(100)                       # usa descuento=0 e iva=0.16
precio_final(100, iva=0)                # cambia solo el IVA
precio_final(descuento=10, precio=250)  # cualquier orden
```

¡Ya los usabas! `print` tiene parámetros con valor por defecto, como `sep` (lo que va entre
los valores) y `end` (lo que va al final):

```python
print("a", "b", "c", sep="-")   # a-b-c
print("Cargando", end="...")    # no salta de línea
print("listo")                  # Cargando...listo
```

## Redondear resultados

Para dinero conviene redondear: `round(valor, 2)` devuelve el número con 2 decimales.

```python
print(round(3.14159, 2))   # 3.14
```

## Resumen

- `def f(a, b=10):` → `b` vale 10 si no se envía.
- Los parámetros con valor por defecto van al final.
- `f(b=3, a=1)`: los argumentos con nombre pueden ir en cualquier orden.
