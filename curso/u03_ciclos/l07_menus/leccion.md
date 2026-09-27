# Programas con menú

Muchos programas de consola muestran un **menú**, ejecutan la opción elegida y vuelven a
mostrar el menú hasta que el usuario decide salir. Ya tienes todo lo necesario: un ciclo,
condiciones y variables.

```python
opcion = ""
while opcion != "3":
    print("1) Saludar")
    print("2) Despedirse")
    print("3) Salir")
    opcion = input("Opción: ")
    if opcion == "1":
        print("¡Hola!")
    elif opcion == "2":
        print("¡Adiós!")
    elif opcion != "3":
        print("Opción no válida")
print("Programa terminado")
```

Consejos para programas más grandes:

- **Planea antes de escribir**: ¿qué datos guarda el programa? ¿qué hace cada opción?
- Escribe y prueba **una opción a la vez**.
- Usa nombres claros y comentarios para las partes importantes.
- Valida los datos del usuario (¿qué pasa si escribe una opción que no existe?).

> Para limpiar la pantalla entre menús puedes usar `import os` y `os.system("cls")`.
> Funciona en la consola de esta app y en Windows.

## Resumen

- Menú = ciclo + `input` + `if/elif`.
- Construye y prueba tu programa por partes.
