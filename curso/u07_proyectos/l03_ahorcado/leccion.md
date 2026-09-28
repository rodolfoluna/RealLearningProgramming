# Proyecto: el ahorcado

El juego clásico: la computadora tiene una palabra secreta y tú adivinas letra por letra.
Tienes 6 vidas; cada letra que no está en la palabra te quita una.

```texto
_ _ _ _ _ _ _ _
Vidas: 6
Letra: a
¡Bien!
_ _ _ _ _ a _ a
Vidas: 6
Letra: e
No está
```

## Los datos

- La palabra secreta: una cadena, por ejemplo `"programa"`.
- Las letras que ya dijo el jugador: una **lista** que empieza vacía.
- Las vidas: un número.

## Las piezas

1. Una función que arme lo que ve el jugador: cada letra de la palabra si ya la adivinó, o `_`
   si no. Las separamos con espacios para que se lean mejor.
2. Una función que diga si ya se adivinaron **todas** las letras.
3. El ciclo del juego, que termina cuando ganas o cuando te quedas sin vidas.

## Condiciones de un ciclo con dos salidas

```python
while vidas > 0 and not palabra_completa(palabra, letras):
    ...
```

Al salir del ciclo, pregunta **por qué** salió para mostrar el mensaje correcto.

## Resumen

- Guarda las letras usadas en una lista y revisa con `in` si ya se usaron.
- Construye el texto a mostrar recorriendo la palabra.
- El ciclo principal tiene dos condiciones de término.
