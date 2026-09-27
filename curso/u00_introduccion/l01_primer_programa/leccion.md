# Tu primer programa

**Programar** es escribir instrucciones precisas para que la computadora las siga, una por una,
de arriba hacia abajo. Un **programa** es un archivo de texto con esas instrucciones, escrito en
un **lenguaje de programación**. En este curso usaremos **Python**, uno de los lenguajes más usados
en el mundo por ser claro y fácil de leer.

## Cómo usar esta app

- A la izquierda está el **temario**: unidades, lecciones y actividades.
- En cada actividad escribes tu código en el **editor**.
- **▶ Ejecutar** (F5) corre tu programa en la **consola**, como en una terminal.
- **✔ Probar** (F6) revisa tu programa con pruebas automáticas y te dice qué falta.
- Si te atoras, pide una **pista**. No hay castigo, pero tu profesor verá cuántas usaste.

> **Importante:** en el editor **no se puede pegar texto**. Aprender a programar se parece a
> aprender a tocar un instrumento: hay que practicar con tus propias manos. La app cuenta las
> veces que copias o intentas pegar, y tanto tú como tu profesor pueden ver esas estadísticas.

## Mostrar texto con `print()`

La instrucción `print()` muestra en la consola lo que pongas entre los paréntesis.
El texto va entre comillas (dobles `"` o sencillas `'`):

```python
print("Hola, mundo")
print('Estoy aprendiendo Python')
```

Presiona **Probar** en el ejemplo de arriba para verlo funcionar. Cada `print()` escribe una
línea nueva. Si quieres una línea en blanco, usa `print()` sin nada.

También puedes mostrar varias cosas separadas por comas; Python pone un espacio entre ellas:

```python
print("Tengo", 18, "años")
```

## Comentarios

Todo lo que va después de `#` es un **comentario**: Python lo ignora. Sirve para explicar tu código
a otras personas (¡y a ti mismo en el futuro!).

```python
# Este programa saluda
print("Hola")  # esto también es un comentario
```

## Los errores son normales

Si escribes algo que Python no entiende, aparece un **error**. La app te dirá **en qué línea**
está y te explicará en español qué significa. Por ejemplo, olvidar cerrar las comillas:

```python
print("Hola)
```

Python distingue **mayúsculas y minúsculas**: `print` funciona, pero `Print` no.

## Resumen

- `print(...)` muestra texto o valores en la consola.
- El texto va entre comillas.
- `#` inicia un comentario.
- Los errores indican la línea donde está el problema: léelos con calma.
