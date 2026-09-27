# Comparaciones y valores booleanos

Para tomar decisiones, los programas hacen **preguntas** cuya respuesta es `True` (verdadero)
o `False` (falso). Esas preguntas se escriben con **operadores de comparación**:

| Operador | Pregunta               | Ejemplo    | Resultado |
|----------|------------------------|------------|-----------|
| `==`     | ¿es igual?             | `5 == 5`   | `True`    |
| `!=`     | ¿es diferente?         | `5 != 3`   | `True`    |
| `>`      | ¿es mayor?             | `2 > 7`    | `False`   |
| `<`      | ¿es menor?             | `2 < 7`    | `True`    |
| `>=`     | ¿es mayor o igual?     | `7 >= 7`   | `True`    |
| `<=`     | ¿es menor o igual?     | `8 <= 7`   | `False`   |

```python
edad = 17
print(edad >= 18)
print(edad == 17)
```

> **No confundas `=` con `==`.** Un solo `=` **asigna** un valor; dos `==` **comparan**.

También se pueden comparar textos: `==` distingue mayúsculas (`"Sí" == "sí"` es `False`), y
`<` compara en orden alfabético.

## Resumen

- Las comparaciones producen `True` o `False` (tipo `bool`).
- `==` compara, `=` asigna.
