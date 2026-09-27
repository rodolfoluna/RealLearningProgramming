# Proyecto: conversor de bases

Las computadoras guardan todo con **bits**: ceros y unos. Los números que usamos todos los días
están en **base 10** (decimal); los de la computadora, en **base 2** (binario).

## ¿Cómo se lee un número binario?

Cada posición vale el doble que la de su derecha: 1, 2, 4, 8, 16…

```texto
  1     0     1     1
  8  +  0  +  2  +  1  = 11
```

Un truco para convertir de binario a decimal recorriendo de izquierda a derecha: empieza en 0 y,
por cada dígito, **multiplica por 2 y suma el dígito**:

```texto
"1011":  0*2+1 = 1  →  1*2+0 = 2  →  2*2+1 = 5  →  5*2+1 = 11
```

## De decimal a binario

Divide entre 2 repetidamente. Los **residuos**, leídos de abajo hacia arriba, forman el número
binario:

```texto
11 ÷ 2 = 5, residuo 1
 5 ÷ 2 = 2, residuo 1
 2 ÷ 2 = 1, residuo 0
 1 ÷ 2 = 0, residuo 1     →  1011
```

En Python: `n % 2` es el residuo y `n // 2` el cociente. Como los residuos salen del último al
primero, agrégalos **al principio** de la cadena resultado.

> Python ya trae `bin(11)` y `int("1011", 2)`. En este proyecto hazlo tú con ciclos: es un
> excelente ejercicio de acumuladores.

## Resumen

- Binario → decimal: `valor = valor * 2 + digito` por cada dígito.
- Decimal → binario: residuos de dividir entre 2, en orden inverso.
- Cuidado con el 0: su binario es `"0"`.
