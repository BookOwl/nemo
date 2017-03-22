# nemo
A fishy programming language that loves pipes.

## Using
**Warning!** nemo is in pre-pre-alpha, everything may change at a moments notice or stop working at all.
You can build nemo with cargo:

```bash
$ git clone https://github.com/BookOwl/nemo
$ cd nemo
$ rustup overide set nightly # nemo requires nightly Rust to build
$ cargo build
```

You can start the REPL with `cargo run`. nemo can not currently run programs from files.

## Examples (may not all work yet)

```
fact(x) => if x < 2 then 1 else x * fact(n - 1)

is_prime(n) => range(ceil(sqrt(n))) | map(x -> n % x != 0) | all

primes_up_to(n) => range(n) | filter(is_prime)
```

## License
nemo is [UNLICENSED](UNLICENSE).

## Credits
I am using a custom fork of the excellent [coroutine](https://github.com/rustcc/coroutine-rs) licensed under the MIT license. It's license can be found [here](coro/LICENSE-MIT).
