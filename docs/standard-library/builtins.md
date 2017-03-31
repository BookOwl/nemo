## nemo builtins.

* [print](#print)
* [input](#input)
* [range](#range)
* [map](#map)
* [filter](#filter)
* [reduce](#reduce)
* [show_pipe](#show_pipe)

<a id="print"></a>
### print
The `print` function displays the arguments passed to it on stdout.

Example:
```
print(1)
print(1, 2, 3)
```

<a id="input"></a>
### print
The `input` function reads a line of text from stdin and returns it.

Example:
```
x := input()
```

<a id="range"></a>
### range
The `range` function pushes all the integers from 0 to `n`-1 into the pipeline.  

Example:
```
range(10) | show_pipe() # displays 0 to 9 on stdout
```

<a id="map"></a>
### map
The `map` function adapts a pipeline by converting each object in the stream to a new object
by calling the passed in function on the object.

Example:
```
range(10) | map(x -> x * x) # converts the pipeline of number 0-9 to the squares of those numbers
```


<a id="filter"></a>
### filter
The `filter` function adapts a pipeline by passing on only the objects that calling the passed in
predicate returns truthy for.

Example:
```
range(10) | filter(x -> x % 2 = 0) # filters the pipeline to only have even numbers
```

<a id="reduce"></a>
### reduce
The `show_pipe` function uses the supplied function to combine the elements of the pipe to a single output value, starting with the passed in start value.

Example:
```
range(11) | reduce(|acc, x| -> acc + x, 0) # evaluates to 55
```

[This webpage](https://www.martinfowler.com/articles/collection-pipeline/reduce.html) explains `reduce` pretty well.

<a id="show_pipe"></a>
### show_pipe
The `show_pipe` function consumes the pipeline and outputs everything in it to stdout.

Example:
```
range(10) | show_pipe() # displays 0 to 9 on stdout
```
