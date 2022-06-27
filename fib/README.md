## Fib

Small script to take a list of integers and print out the fibonacci numbers at those indexes

## Usage

```bash
fib 3 5 7
```

will produce
```
2 5 13
```

Invalid arguments (non ints, negative numbers) will result in an error message (printed to stderr).

Keep in mind that this implementation indexes beginning at 0 with the base values of `0 => 0, 1 => 1`