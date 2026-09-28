# Tablas con formato y cifrado de mensajes

## Alinear texto en columnas

Dentro de las llaves de una f-string puedes indicar el **ancho** de un valor y hacia dónde se
alinea:

| Formato | Significa | Ejemplo | Resultado |
|---|---|---|---|
| `{x:<10}` | a la izquierda, ancho 10 | `f"[{'Ana':<6}]"` | `[Ana   ]` |
| `{x:>10}` | a la derecha, ancho 10 | `f"[{'Ana':>6}]"` | `[   Ana]` |
| `{x:^10}` | centrado, ancho 10 | `f"[{'Ana':^7}]"` | `[  Ana  ]` |
| `{x:>8.2f}` | número con 2 decimales, a la derecha | `f"[{3.14159:>8.2f}]"` | `[    3.14]` |
| `{n:03d}` | entero rellenando con ceros | `f"{7:03d}"` | `007` |

Con esto puedes hacer tablas que se vean ordenadas en la consola:

```python
print(f"{'Producto':<12}{'Precio':>8}")
print(f"{'Pan':<12}{12.5:>8.2f}")
print(f"{'Leche':<12}{27:>8.2f}")
```

```texto
Producto      Precio
Pan            12.50
Leche          27.00
```

> Los textos van a la izquierda y los números a la derecha: así las cifras quedan alineadas.

## Códigos de los caracteres

Cada carácter tiene un número. `ord()` da el número de un carácter y `chr()` hace lo contrario:

```python
print(ord("A"), ord("B"), ord("a"))   # 65 66 97
print(chr(67))                        # C
print(chr(ord("a") + 2))              # c
```

Las letras de la `a` a la `z` tienen números seguidos (97 a 122), igual que de la `A` a la `Z`
(65 a 90). Por eso podemos "avanzar" letras sumando.

## Cifrado César

Julio César cifraba sus mensajes **recorriendo cada letra** un número fijo de posiciones en el
alfabeto. Con un desplazamiento de 3: `a → d`, `b → e`, …, `x → a`, `y → b`, `z → c`.

Para dar la vuelta al llegar a la `z` se usa el residuo `% 26`:

```python
letra = "y"
posicion = ord(letra) - ord("a")          # 24
nueva = (posicion + 3) % 26               # 1
print(chr(ord("a") + nueva))              # b
```

Para **descifrar** se recorre en sentido contrario (desplazamiento negativo).

## Resumen

- `{valor:<ancho}`, `{valor:>ancho}`, `{valor:^ancho}` alinean; `.2f` fija los decimales.
- `ord()` convierte carácter → número y `chr()` número → carácter.
- `% 26` permite dar la vuelta al alfabeto.
