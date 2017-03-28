## Welcome to the nemo tutorial!
This is a simple introduction to nemo that should get you up and running.

1. [Installing nemo](#installing)
2. [Starting the REPL](#starting-the-repl)
3. [Basic math](#basic-operators)
4. [Conditionals](#conditionals)
5. [Variables](#variables)
6. [Loops and Blocks](#loops-and-blocks)
7. [Functions](#functions)
8. [Pipes](#pipes)
9. [Running a program from a file](#running-a-file)

<a id="installing"></a>
### Installing
Once nemo gets to a more useable state I'll provide pre-compiled binaries, but for right now you will need to compile nemo yourself:
```bash
$ git clone https://github.com/BookOwl/nemo
$ cd nemo
$ rustup override set nightly
$ cargo build
$ cp target/debug/nemo /usr/local/bin/nemo # puts nemo on your path
```

<a id="starting-the-repl"></a>
### Starting the REPL
The nemo REPL (read-eval-print loop) can be started by just running the nemo program without any arguments.
```bash
$ nemo
```
You can also pass the `--repl` flag to start the REPL.

<a id="basic-operators"></a>
### Basic operators
We can now start coding! 🎉

Start the REPL and follow along.

We can create numbers by just typing them in:
```
> 3
3
> 42
42
> 3.14159
3.14159
```

We can also add, subtract, multiply and divide them:
```
> 3 + 1
4
> 3.14 * 2
6.28
> 57 - 2 * 5
47
> 22 / 11
2
```
The operator precedence is just like you learned in school. First comes multiplication and division, then addition and subtraction. You can use parentheses to adjust the precedence:
```
> (57 - 2) / 5
11
```

nemo also has booleans which can be entered as `true` and `false`:

```
> true
true
> false
false
```

There are also operators that work on bools:

```
> true and true
true
> false and true
false
> true or false
true
> false or false
false
```

nemo also has comparison operators that create bools:

```
> 1 > 5
false
> 3.14 > 2
true
> 5 = 5
true
> 5 != 5
false
```

Note that in nemo `=` means `is equal to`, not assignment.

<a id="conditionals"></a>
### Conditionals
Now that we have these bools, what can we do with them? Probably the simplest thing is to create conditional expressions, or conditionals for short. In nemo, conditionals are entered as `if some_val then if_true else if_false`. This is much like if..else blocks in other languages, but there are two major differences.

1. Like the name suggests, conditionals are expressions that produce a value, not control flow statements. In fact, there are no statements in nemo, just declarations and expressions.
2. The else clause is mandatory. This assures that the conditional always evaluates to _something_.

Let's try entering some conditionals and see what we get:

```
> if true then 1 else 2
1
```

Simple enough, the predicate was `true` so we get the if_true value.

```
> if 5 > 2 then 1 else 2
1
```

Again this is pretty simple. `5 > 2` evaluates to `true`, so we got 1.

```
> if 5 < 2 then 1 else 2
2
```

Same thing as before except the predicate evaluates to `false`, so we got 2.

```
> if 5 then 1 else 2
1
> if 0 then 1 else 2
```

These examples are more interesting. They show us that you don't have to only use bools as predicates in conditionals, and unlike many languages 0 is considered to be a truthy value. In nemo the only value that is falsy in a conditional is `false`.

<a id="variables"></a>
### Variables
A major part of programming is giving names to values. These names are called *variables*, and nemo alows you to create them with the assignment expression.

```
> x := 5
0
```
This example shows us a couple things. First, nemo uses `:=` for variable assignment, unlike most modern languages. This frees up the `=` operator for equality checking, which happens more often. Two, the result of the assignment expression is 0, not the value that was assigned. This may change soon though, so don't really on it.

We can now access the value of our variable by just typing its name:

```
> x
5
```

We can also use it in an expression:

```
> x * 2
10
```

<a id="loops-and-blocks"></a>
### Loops and Blocks
Blocks allow us to have multiple expression execute one after another. You create them by using curly putting your expressions in curly braces (`{` and `}`) separated by semicolons (`;`).

```
> {1; 2; 3}
3
```
Notice that the value produced by the block is the last expression in the block. Blocks are mainly used when you want multiple expressions to be run in a spot that only allows one expression, like the bodies of conditionals.

nemo also has a while loop. It runs the code in its body as long as evaluating its predicate returns a truthy value. They are entered as `while Predicate do Body`.

Here is a while loop that calculates the sum of the numbers from 1 to 10:

```
> i := 0
0
> x := 0
0
> while i < 10 do {i := i + 1; x := x + i}
0
> x
55
```

See how handy the block is? It allowed do assign values to both x and i in the same loop.

<a id="functions"></a>
### Functions
Like most languages, nemo has functions. However, its syntax for declaring functions is very different than most languages.

In nemo you declare a function as `function_name(arg1, arg2, ... argN) => body`. Functions can take any number of arguments.

Here is a function that returns the square of its argument:
```
> square(x) => x * x
```
Please note that even though we entered this function declaration at the REPL, function declarations are NOT expressions and are only valid at the top level of a program.

We can call our newly defined function by putting its name in front of parentheses and putting its argument in the parentheses:

```
> square(5)
25
> square(10)
100
```

Functions in nemo are first class, you can stuff them into a variable, pass them into other function, return them from functions, and create anonymous functions.

Anonymous functions (or closures as they are often called) are created in nemo by putting the argument(s) in front of an arrow and the body of the closure behind the arrow.

This closure will also return the square of its argument:

```
> x -> x * x
function lambda(x)
```
(The "lambda" part of that description is homage to Scheme, one of the first languages to have first class functions. In Scheme closures are called lambdas)

A closure that adds two numbers could like this:

```
> |a, b| -> a + b
function lambda(a, b)
```


We can put the closure in a variable and call it from that variable:

```
> foo := x -> x * x
0
> foo(5)
25
```

We can also pass it to a function:

```
> call_twice(func, input) => func(func(input))
> call_twice(x -> x + 1, 5)
7
```

<a id="pipes"></a>
### Pipes
nemo's most unique feature is pipes. Much like pipes in a Unix shell, pipes in nemo allow values to flow from one expression to another. You can create a pipe using the `|` operator, push values into the pipe with `push Expr`, and pull a value out of the pipe with `pull`.

Here is a simple pipeline that just moves a value from one expression to another one:

```
> {push 5} | {pull}
5
```
The 5 got pushed from the first block to the second block.

Like in the shell, pipes in nemo run in parallel.

nemo has several built in functions for working with pipes, check out the [builtins docs](standard-libary/buitlins.md) for more.


<a id="running-a-file"></a>
### Running programs from a file
You can run the nemo interpreter on the contents of a file by passing the file's name as an argument to the nemo binary like `nemo path_to_file.nemo`. It is customary to end nemo program files with the .nemo file extension.

When you run a program that is in a file nemo will try to run the `main()` function in that file.

Here is a program that outputs the 10th Fibonacci number:

```
fib(n) => if n < 2 then 1 else fib(n-1) + fib(n-2)
main() => print(fib(10))
```

If this is saved in a file called `fibo.nemo` and is run with `nemo fibo.nemo` you will get 89.
