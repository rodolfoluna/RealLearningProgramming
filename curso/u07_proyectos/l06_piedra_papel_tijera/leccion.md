# Proyecto: piedra, papel o tijera

Juegas contra la computadora, que elige al azar. Piedra gana a tijera, tijera gana a papel y
papel gana a piedra.

## Números y elecciones al azar

El módulo `random` genera valores aleatorios:

```python
import random

print(random.randint(1, 6))                          # un dado: 1 a 6
print(random.choice(["piedra", "papel", "tijera"]))  # un elemento al azar
```

Cada vez que ejecutes el programa obtendrás resultados distintos. Por eso las pruebas
automáticas revisan la función que decide el ganador (que **no** depende del azar) por
separado del juego completo.

## Separa la lógica del azar

La regla del juego es una función que recibe las dos elecciones y dice quién gana:

```python
def ganador(jugador, computadora):
    ...   # devuelve "jugador", "computadora" o "empate"
```

Así la puedes probar con todas las combinaciones sin depender de la suerte.

## Un marcador

Usa tres contadores (ganadas, perdidas, empates) y muéstralos al final.

## Resumen

- `random.choice(lista)` elige un elemento al azar; `random.randint(a, b)` un entero.
- Pon las reglas en una función sin azar para poder probarlas.
- Contadores para el marcador.
