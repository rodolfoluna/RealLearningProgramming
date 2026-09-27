# Ciclos con centinela

A veces no sabemos cuántos datos va a capturar el usuario. Un **valor centinela** es un dato
especial que indica "ya terminé" (por ejemplo, `0` o la palabra `fin`):

```python
total = 0
numero = int(input("Número (0 para terminar): "))
while numero != 0:
    total += numero
    numero = int(input("Número (0 para terminar): "))
print("Total:", total)
```

Observa que se pide el primer dato **antes** del ciclo y el siguiente **al final** de cada vuelta.

Otra forma común es un ciclo "infinito" que se rompe con `break` (lo verás en la lección 5):

```python
while True:
    palabra = input("Palabra (fin para salir): ")
    if palabra == "fin":
        break
    print(palabra.upper())
```

## Validar hasta que el dato sea correcto

El mismo patrón sirve para insistir hasta que el usuario escriba un dato válido:

```python
edad = int(input("Edad (0 a 120): "))
while edad < 0 or edad > 120:
    print("Edad inválida")
    edad = int(input("Edad (0 a 120): "))
print("Gracias")
```

## Resumen

- El centinela es un valor que termina la captura.
- Pide el primer dato antes del ciclo y el siguiente al final de cada vuelta.
