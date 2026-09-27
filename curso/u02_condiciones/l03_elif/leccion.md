# Varias opciones con elif

Cuando hay **más de dos casos**, se encadenan condiciones con `elif` ("si no, si..."). Python
revisa las condiciones de arriba hacia abajo y ejecuta **solo el primer bloque** cuya condición
sea verdadera:

```python
calificacion = 85
if calificacion >= 90:
    print("Excelente")
elif calificacion >= 80:
    print("Muy bien")
elif calificacion >= 70:
    print("Bien")
else:
    print("Necesitas estudiar más")
```

Con 85, la primera condición es falsa, la segunda es verdadera → muestra "Muy bien" y **ya no
revisa las demás**.

> El orden importa. Si pusiéramos primero `calificacion >= 70`, ¡un 95 mostraría "Bien"!

## Resumen

- `if` → `elif` (las que necesites) → `else` (opcional).
- Solo se ejecuta el primer bloque verdadero.
- Ordena las condiciones de la más específica a la más general.
