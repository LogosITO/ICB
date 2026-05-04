type
  Calculator = object
    value: int

proc add(calc: Calculator, a, b: int): int =
  result = a + b