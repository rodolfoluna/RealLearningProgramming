# Validar datos de texto

Muchos programas reciben datos con un formato fijo: un número de control, un correo, una
contraseña… **Validar** es revisar que el dato cumpla las reglas **antes** de usarlo.

## Idea general

Escribe una función que responda `True` o `False`. Revisa las reglas una por una y, en cuanto
una no se cumpla, devuelve `False`. Si pasó todas, devuelve `True`:

```python
def es_codigo_postal(texto):
    if len(texto) != 5:
        return False
    if not texto.isdigit():
        return False
    return True
```

## Revisar cada carácter

Para reglas como "debe tener al menos una mayúscula", recorre el texto con una **bandera**
(una variable `True`/`False` que cambia cuando encuentras lo que buscas):

```python
def tiene_mayuscula(texto):
    encontrada = False
    for c in texto:
        if c.isupper():
            encontrada = True
    return encontrada
```

O, más corto, devuelve `True` en cuanto la encuentres:

```python
def tiene_mayuscula(texto):
    for c in texto:
        if c.isupper():
            return True
    return False
```

## Posiciones y partes

- `texto[0]` es el primer carácter; `texto[1:]` es todo **desde** la posición 1.
- `texto.count("@")` dice cuántas veces aparece algo.
- `texto.find("@")` da la posición de algo (o `-1`).
- `texto.split("@")` separa el texto en partes.

## Validar con un ciclo

Combina tu función con un ciclo para pedir el dato hasta que sea correcto:

```python
cp = input("Código postal: ")
while not es_codigo_postal(cp):
    print("Código postal inválido")
    cp = input("Código postal: ")
```

## Resumen

- Una función de validación devuelve `True` o `False`.
- Revisa las reglas en orden y devuelve `False` en cuanto una falle.
- Usa banderas o `return` dentro del ciclo para buscar caracteres especiales.
