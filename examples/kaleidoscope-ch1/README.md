# Chapter 1

When it comes to implementing a language, the first thing needed is the ability to process a text file and recognize what it says. The traditional way to do this is to use a “lexer” (aka ‘scanner’) to break the input up into “tokens”. Each token returned by the lexer includes a token code and potentially some metadata (e.g. the numeric value of a number). First, we define the possibilities:

```rust
pub enum Token {
    EOF,

    // commands
    Def,
    Extern,

    // primary
    Ident(String),
    Number(f64),

    // if unknown
    Op(char),
}
```

## Lexer

The actual implementation of the lexer is a struct that holds the input characters and processes them, turning them into Tokens.

The lexer is only borrowing the input to look at it while it generates the tokens. It stores an iterator, but also the whole
buffer so that we can look forward and backward when we need to.

```rust
pub struct Lexer<'a> {
    input : &'a str,
    iter : Peekable<Chars<'a>>,
}
```

The lexer has a method to get the next token, `lex`. I'll start off with an incomplete and incorrect implementation and then expand
it. The first thing that we'll have to do to lex the next token is to get rid of whitespace. Whitespace is irrelevant in the
Kaleidoscope language.

```rust
    pub fn skip_whitespace(&mut self) -> usize { /* ??? */ }

    pub fn lex(&mut self) -> Result<Token, &str> {
        self.skip_whitespace();
```

### Skip Whitespace

The really cool thing here is that we're only on our first function and we can already start taking advantage of Rust's ecosystem
around its standard library when implementing `skip_whitespace`. Remember how we also stored an iterator to the input? We can
use Rust's iterator methods to skip the whitespace in one line!

```rust
    /// Skips all whitespace the Lexer is currently pointing to and returns 
    /// the number of whitespace characters processed
    pub fn skip_whitespace(&mut self) -> usize {
        self.iter.take_while(|x| x.is_whitespace()).count()
    }

    pub fn lex(&mut self) -> Result<Token, &str> {
        self.skip_whitespace();
```

First, `self.iter` gets our iterator, which is a `Peekable<Chars<'a>>`. That just means that it's an iterator that implements all
the normal iterator methods as well as the `peek()` method, and it is wrapped around the `Chars<'a>` iterator. Since we're not 
using `peek()` in this function, you can imagine that `self.iter` is a `Iterator<char>`.

The `take_while` method is a normal iterator method that takes a predicate (a closure that takes one elemtnt and returns a boolean)
and returns a new iterator, a `TakeWhile<I>`. A TakeWhile is like a Peekable, it's a normal iterator, but slightly different. In
this case, a TakeWhile will stop (return `None` on `next()`) once the predicate is false.

TODO: Fix for `PeekingTakeWhile`

Finally, the `count()` method keeps calling `next()` on the `TakeWhile` iterator until `None` is reached, and then returns the number
of things it skipped

### Check for EOF

### Next Identifier

Now that we've skipped the whitespace, we need to check for a real token.
The first one we'll look at is identifiers. Identifiers are just strings of
characters. They could be variables, functions, or keywords.

Let's start by finding keywords first.

TODO: Fill this in

### Eating Comments

Use `take_while` this time instead of `peeking_take_while`

Recursion

### Reading Numbers

This is very similar to reading idents, but after we've collected the part of the
input we want to read, we parse it using `str::parse()` and then process the
result so that we create a Token on Ok and a string on Err

### Tests

