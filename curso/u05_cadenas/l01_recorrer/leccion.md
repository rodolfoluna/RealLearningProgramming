# Recorrer cadenas

Una cadena es una **secuencia** de caracteres, así que un `for` puede recorrerla letra por letra:

```python
for letra in "Hola":
    print(letra)
```

```texto
H
o
l
a
```

## Contar

Combina el recorrido con un contador y una condición:

```python
frase = "programar en python"
contador = 0
for letra in frase:
    if letra == "a":
        contador += 1
print(contador)   # 3
```

## El operador in

`in` pregunta si un texto está dentro de otro. Es muy útil para revisar si un carácter
pertenece a un grupo:

```python
print("a" in "aeiou")       # True
print("py" in "python")     # True
letra = "x"
if letra in "aeiouAEIOU":
    print("Es vocal")
```

## Construir una cadena nueva

Las cadenas **no se pueden modificar** (`texto[0] = "X"` es un error). Para "cambiarlas" se
construye una cadena nueva, empezando con `""` y agregando partes:

```python
original = "gato"
nuevo = ""
for letra in original:
    nuevo = nuevo + letra.upper() + "."
print(nuevo)   # G.A.T.O.
```

Si agregas cada letra **al principio** (`nuevo = letra + nuevo`), obtienes el texto al revés.

## Recorrer con posiciones

Si necesitas la posición de cada carácter, recorre con `range(len(texto))`:

```python
texto = "sol"
for i in range(len(texto)):
    print(i, texto[i])
```

## Resumen

- `for letra in texto:` recorre cada carácter.
- `x in texto` pregunta si `x` aparece en `texto`.
- Para transformar, construye una cadena nueva a partir de `""`.
