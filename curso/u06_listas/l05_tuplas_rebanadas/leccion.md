# Tuplas, rebanadas y listas por comprensión

## Rebanadas

Igual que en las cadenas, puedes tomar un **pedazo** de una lista con `[inicio:fin]`. El
elemento en `fin` **no** se incluye:

```python
letras = ["a", "b", "c", "d", "e", "f"]
print(letras[1:4])    # ['b', 'c', 'd']
print(letras[:2])     # ['a', 'b']      (desde el principio)
print(letras[3:])     # ['d', 'e', 'f'] (hasta el final)
print(letras[-2:])    # ['e', 'f']      (los dos últimos)
print(letras[::2])    # ['a', 'c', 'e'] (de 2 en 2)
print(letras[::-1])   # al revés
```

Una rebanada es una lista **nueva**. Con `+` puedes unir listas: `[1, 2] + [3]` da `[1, 2, 3]`.

## Tuplas

Una **tupla** es como una lista que **no se puede modificar**. Se escribe con paréntesis:

```python
punto = (3, 5)
fecha = (10, "mayo", 2024)
print(punto[0])        # 3
# punto[0] = 7         # TypeError: las tuplas no se modifican
```

Se usan para datos que van juntos y no deben cambiar. Además, una función puede **devolver
varios valores** en una tupla, y puedes "desempacarlos" en varias variables:

```python
def dividir(a, b):
    return a // b, a % b      # devuelve la tupla (cociente, residuo)

cociente, residuo = dividir(17, 5)
print(cociente, residuo)      # 3 2
```

## Listas por comprensión

Es una forma corta de construir una lista a partir de otra:

```python
numeros = [1, 2, 3, 4, 5]
dobles = [n * 2 for n in numeros]             # [2, 4, 6, 8, 10]
pares = [n for n in numeros if n % 2 == 0]    # [2, 4]
cuadrados = [i ** 2 for i in range(1, 4)]     # [1, 4, 9]
```

Se lee: "`n * 2` **para cada** `n` **en** `numeros`". Es equivalente a crear una lista vacía y
hacer `append` en un `for`; usa la forma que te resulte más clara.

## Convertir a lista

`list()` convierte otras secuencias en listas:

```python
print(list(range(5)))    # [0, 1, 2, 3, 4]
print(list("hola"))      # ['h', 'o', 'l', 'a']
```

## Resumen

- `lista[a:b]` toma de la posición `a` hasta antes de `b`; `[::-1]` invierte.
- Las tuplas `( )` no se modifican; sirven para devolver varios valores.
- `[expresión for x in lista if condición]` construye una lista nueva.
