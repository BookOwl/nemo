# nemo
A fishy programming language.

## Examples (may not all work yet)
```
fact(x) => if x < 2 then 1 else x * fact(n - 1)

is_prime(n) => range(ceil(sqrt(n))) | map(x => n % x != 0) | all

primes_up_to(n) => range(n) | filter(is_prime)
```

## License
nemo is [UNLICENSED](UNLICENSE).
