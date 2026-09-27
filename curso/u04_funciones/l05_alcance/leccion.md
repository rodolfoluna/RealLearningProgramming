# Alcance de las variables

El **alcance** de una variable es la parte del programa donde existe.

## Variables locales

Las variables que creas **dentro** de una función (y sus parámetros) son **locales**: nacen
cuando la función se ejecuta y desaparecen cuando termina.

```python
def calcular():
    total = 5 * 3      # total es local
    return total

calcular()
print(total)           # NameError: total no existe aquí afuera
```

Para usar un resultado fuera de la función, **devuélvelo** y guárdalo:

```python
total = calcular()
```

## Una variable local puede tener el mismo nombre que una de afuera

Son variables **distintas**, aunque se llamen igual:

```python
x = 10

def cambiar():
    x = 99             # esta x es local; la de afuera no cambia
    print(x)

cambiar()              # 99
print(x)               # 10
```

## El error UnboundLocalError

Si dentro de una función **asignas** a una variable, Python la considera local en **toda** la
función. Por eso esto falla:

```python
contador = 0

def incrementar():
    contador = contador + 1   # UnboundLocalError

incrementar()
```

Python intenta leer la `contador` local antes de que tenga valor. La solución recomendada es
**recibir** el valor como parámetro y **devolver** el nuevo:

```python
def incrementar(contador):
    return contador + 1

contador = 0
contador = incrementar(contador)
```

> Existe la palabra `global` para modificar variables de afuera, pero hace los programas más
> difíciles de entender y de probar. Usa parámetros y `return`.

## Resumen

- Lo que se crea dentro de una función es **local**.
- Una variable local no cambia a otra del mismo nombre que esté afuera.
- Para "sacar" datos de una función, usa `return`; para "meterlos", usa parámetros.
