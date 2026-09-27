# Parámetros y argumentos

Una función es más útil si puede trabajar con **datos distintos** cada vez. Para eso se le dan
**parámetros**: variables que se escriben entre los paréntesis de la definición.

```python
def saludar(nombre):
    print(f"¡Hola, {nombre}!")

saludar("Ana")      # ¡Hola, Ana!
saludar("Luis")     # ¡Hola, Luis!
```

- `nombre` es el **parámetro**: una variable que solo existe dentro de la función.
- `"Ana"` y `"Luis"` son los **argumentos**: los valores que se le dan al llamarla.

## Varios parámetros

Se separan con comas. Los argumentos se asignan **en el mismo orden**:

```python
def rectangulo(base, altura):
    print(f"Área: {base * altura}")

rectangulo(5, 3)     # base = 5, altura = 3
```

Si cambias el orden de los argumentos, cambian los valores que recibe cada parámetro. Y si
das más o menos argumentos de los que la función pide, obtendrás un `TypeError`.

## Los argumentos pueden ser expresiones

```python
precio = 120
rectangulo(precio / 10, 2 + 1)   # base = 12.0, altura = 3
```

Python calcula primero el valor de cada argumento y después llama a la función.

## Resumen

- Los **parámetros** van en la definición; los **argumentos**, en la llamada.
- Los argumentos se asignan a los parámetros en orden.
- Un parámetro es una variable que solo existe dentro de la función.
