# Chapter 2

## Abstract Syntax Tree

We need an AST to represent expressions in our language. The only kind
of values (expressions that evaluate to themselves) are floating point numbers.

This means that we have no ints, strings, or first-class functions.

Other than numbers, there are variables, calls to previously defined functions,
and binary infix operators. Note that the only way to define variables right
now is through the function parameters. This is a very minimalist grammar, which
we will expand later. For now, here is an enum representing the expressions we
can create

```rust
pub enum Expr {
    /// Numberic literal
    Number(f64),
    /// Variable: any ident not a keyword or function call
    Variable(String),
    /// Infix operator
    Binary(Expr, char, Expr),
    /// Function call
    Call(String, Vec<Expr>),
}
```

## Function Definitions

Remember that everything in our language is inside a function definition, even 
evaluating an expression is just the definition of an anonymous function.

So, we need to be able to define functions, this consists of the function signature
(the name of the function and its parameter names) as well as the body (an expression).
They are defined as such

```rust
pub struct Prototype {
    name : String,
    params : Vec<String>,
}

pub struct FunctionDef {
    prot : Prototype,
    body : Expr,
}
```

## Basic Expression Parsing

We will need a parser to start parsing. Parsing is kind of like lexing,
so we'll want to have our input (in the form of tokens) accessible through
an iterator

TODO: Add code

## Parsing API

In the end, we want our parser to expose a nice `parse` function that will
return a FunctionDef for the biding that the user just typed in.

I'm going to start off by implementing this public parse function with a hack
just so that we keep the API consistent. It will only create anonymous functions
so we can still evaluate expressions

```rust
pub fn parse(&mut self) -> Result<FunctionDef, &str> {
    FunctionDef {
        prot : Prototype {
            name : "anonymous",
            params : Vec::default()
        },
        body : self.parse_expr()
    }
}
```

Note that the body of the function definition is just one expression, so let's define a public expression parsing API

```rust
/// Parse a single expression of any kind
pub fn parse_expr(&mut self) -> Result<Expr, &'static str> {
    if self.iter.peek().is_none() {
        return Err("unexpected end of tokens");
    }

    match *self.iter.peek().unwrap() {
        Num(n) => self.parse_number(),
        _ => Err("not implemented yet")
    }
}
```

## Parsing Numbers

You can see in our `parse_expr`, if we encounter the number token, we will parse a number. We're just unwrapping the
*token* Num and replacing it with the *expression* Number wrapped in an Ok for the Result

```rust
fn parse_number(&mut self) -> Result<Expr, &'static str> {
    match self.iter.next().unwrap() {
        Num(n) => Ok(Number(n)),
        _ => Err("expected number")
    }
}
```

## Adding Parentheses


### Lexing Parentheses

So, we forgot to implement parenthetical expressions last time, so we're just going to throw them in now. Put these lines
in `lex()` function to match and eat the parens.

```rust
pub enum Token {
    //...

    LeftParen,
    RightParen,

}

//...
pub fn lex(&mut self) -> Result<Token, &str> {
    //...


    let result = match just_char(self.iter.peek().unwrap()) {
        '#' => {
            self.eat_comment();
            self.lex()?
        }
        '(' => { self.iter.next(); LeftParen }
        ')' => { self.iter.next(); RightParen }
        //...
}
```

## Parsing Parentheses

Now, to parse parens, add this line to your parse_expr

```rust
// in parse_expr
match *self.iter.peek().unwrap() {
    Num(n) => self.parse_number(),
    LeftParen => self.parse_paren(),
    _ => Err("not implemented yet")
}
```

Now let's define the `parse_paren` method:

TODO: change debug_asserts to consume(Token)

TODO: add comma lexing

```rust
/// Parses a parenthetical expression
/// Pre: current token in left paren
fn parse_paren(&mut self) -> Result<Expr, &'static str> {
    debug_assert!(*self.iter.peek().unwrap() == LeftParen);
    self.consume();
    let res = self.parse_expr();
    debug_assert!(*self.iter.peek().unwrap() == RightParen);
    self.consume();
    res
}
```

You'll notice these `debug_assert!()` macros, they're like doing a little test. When you build this code in debug
mode, they will run as asserts and panic if they're expression is false, but when you build for release, they
go away, so you don't have to be worried about this slowing down your production code.

Here what we're saying is "The character we're on should be a left paren". Then we call consume which is just
`self.iter.next();` and then we call `parse_expr()` again, so we can recursively parse expressions inside of
parentheses.

## Idents as Variables and Function Calls

We'd like to be able to use variables, so if the variable defines a function like

```
def double(x) x * 2
#   ^^^^^^^^^-- definition
#             ^^^^^---- body expression
```

Then we should be able to use the `x` in the body since it was defined in the definition. When we lexed this function,
We would find that x is an Ident. Unlike the Num token and Number expression, there isn't a direct translation.

This is because an Ident could be either a variable or the name of function being called

```
def fib(x)
  if x < 3 then 1
#    ^-- using an ident as a variable
  else fib(x - 1) + fib(x - 2)
#      ^^^-- using an ident as a function call
```

For now, let's naively assume that there are no function calls, and interpret idents as variables.

## Parsing Variables

Let's make a `parse_id` function that will parse our variables to start

```rust
fn parse_id(&mut self) -> Result<Expr, &'static str> {
    let s = match self.iter.next().unwrap() {
        Ident(s) => s,
        _ => { return Err("Expected ident token"); }
    };

    if self.curr() == LeftParen {
        // Function call
        Err("not implemented yet")
    } else {
        // variable
        Ok(Variable(s))
    }
}
```

## Checking for Unexpected End of Input

TODO: correctness stuff

## Infix Operator Lexing

## Infix Operator Parsing

TODO: Make a version of this with Nom