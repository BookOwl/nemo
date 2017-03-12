# nemo
A semi-functional (in more ways than one 😉) programming language.

## Example
```
range(max) = {
    n := 0;
    while n < max {
        push n;
        n +=1;
    }
}

map(func) = loop { push func(pull) }

filter(func) = loop {
    x := pull;
    if func(x) {
        push x;
    };
}


main() = range(10) | map(x -> x*x) | filter(x -> x % 2 == 0) | foreach(x -> display(x))
```

## License
nemo is [UNLICENSED](UNLICENSE).
