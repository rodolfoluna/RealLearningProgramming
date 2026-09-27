# Proyecto: control de calificaciones

En esta unidad harás programas **completos**, como los que usarías en la vida real. Son más
largos, así que conviene seguir un método.

## Cómo abordar un proyecto

1. **Entiende el problema.** Lee el enunciado completo y fíjate en los mensajes exactos que
   debe mostrar el programa.
2. **Decide qué datos guardar.** ¿Una lista? ¿Una lista de registros (listas dentro de una
   lista)?
3. **Divide en funciones.** Cada cálculo importante es una función que **devuelve** su
   resultado. Las pruebas automáticas revisan cada función por separado.
4. **Escribe y prueba una parte a la vez.** Primero una función, pruébala con ▶ Ejecutar o con
   ✔ Probar; luego la siguiente.
5. **Arma el programa principal** (normalmente un menú) usando tus funciones.
6. **Valida los datos**: ¿qué pasa si el usuario escribe texto donde va un número?

## Documentar funciones

Un texto entre triples comillas justo debajo del `def` se llama **docstring** y explica qué
hace la función. No cambia lo que hace el programa, pero ayuda a quien lo lee (¡incluido tú
en un mes!):

```python
def promedio_grupo(registros):
    """Devuelve el promedio de las calificaciones (0 si no hay alumnos)."""
    ...
```

## Los datos de este proyecto

Cada alumno es un **registro** `[nombre, calificacion]` y todos se guardan en una lista:

```python
registros = [["Ana", 85], ["Luis", 60]]
registros.append(["Sofía", 92])
for nombre, calificacion in registros:
    print(nombre, calificacion)
```

> Cuando termines un proyecto, usa **Crear .exe** para convertirlo en un programa que puedes
> abrir con doble clic y compartir (en computadoras con Windows).

## Resumen

- Planea: datos, funciones, programa principal.
- Construye y prueba por partes.
- Funciones arriba, programa principal (menú) abajo.
