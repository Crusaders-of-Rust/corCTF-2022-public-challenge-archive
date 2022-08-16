Solves: 0

Author: chop0

Description:

typer hardening and prime numbers, what more could you want

24 hr hint drop: citrus orwell has two main parts.
Part one is finding the secret prime modulus.  Range[n, n] is folded to Constant[n % secret_modulus], which you can use to leak two numbers % the secret prime modulus.
Part two is bypassing typer hardening;  Look at old v8 bug reports for inspiration :^)

Flag: `corctf{typ3r_hard3ning_1s_p3p3ga}`
