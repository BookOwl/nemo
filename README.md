![nemo logo](https://cdn.rawgit.com/BookOwl/nemo/69da2588/nemo%20logo.svg)

_fish by [-stache-](https://scratch.mit.edu/users/-stache-)_

## Using
**Warning!** nemo is in pre-pre-alpha, everything may change at a moments notice or stop working at all.
You can build nemo with cargo:

```bash
$ git clone https://github.com/BookOwl/nemo
$ cd nemo
$ rustup overide set nightly # nemo requires nightly Rust to build
$ cargo build
```

You can start the REPL by either passing no arguments or the `--repl` flag to nemo:

```bash
$ nemo # --repl is optional
```

You can run a file by passing its name as an argument:

```bash
$ nemo example.nemo
```

## Examples

```
fact(x) => if x < 2 then 1 else x * fact(n - 1)

is_prime(n) => range(ceil(sqrt(n))) | map(x -> n % x != 0) | all

primes_up_to(n) => range(n) | filter(is_prime)
```

## License
nemo is [UNLICENSED](UNLICENSE).
