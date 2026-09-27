# Controlar ciclos con break y continue

- `break` **termina** el ciclo en ese momento.
- `continue` **salta** el resto de la vuelta actual y pasa a la siguiente.

```python
for n in range(1, 10):
    if n == 5:
        break
    print(n)
print("Salí del ciclo")
```

```python
for n in range(1, 8):
    if n % 3 == 0:
        continue      # los múltiplos de 3 no se muestran
    print(n)
```

## Buscar algo y detenerse

`break` es ideal para **buscar**: en cuanto encuentras lo que buscas, ya no tiene caso seguir.

```python
n = 91
divisor = 2
while divisor < n:
    if n % divisor == 0:
        print(n, "es divisible entre", divisor)
        break
    divisor += 1
```

## Resumen

- `break` termina el ciclo; `continue` salta a la siguiente vuelta.
- Úsalos con moderación: un ciclo con demasiados `break` es difícil de leer.
