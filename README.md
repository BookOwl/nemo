![nemo logo](https://raw.githubusercontent.com/BookOwl/nemo/master/nemo%20logo.png)

_fish by [-stache-](https://scratch.mit.edu/users/-stache-)_

## Using
**Warning!** nemo is in pre-pre-alpha, everything may change at a moment's notice or stop working at all.
You can build nemo with cargo:

```bash
$ git clone https://github.com/nemo-lang/nemo
$ cd nemo
$ rustup override set nightly # nemo requires nightly Rust to build
$ cargo build
```

You can start the REPL by either passing no arguments or the `--repl` flag to nemo:

```bash
$ ./target/debug/nemo # --repl is optional
```

You can run a file by passing its name as an argument:

```bash
$ ./target/debug/nemo example.nemo
```

## Examples
See the [examples directory](examples/) for some example nemo programs.

## License
nemo is [UNLICENSED](UNLICENSE).
