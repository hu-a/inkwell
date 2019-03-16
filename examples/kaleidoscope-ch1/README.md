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
    iter : Peekable<CharIndices<'a>>,
}
```

`CharIndices` is an iterator that returns a tuple of (index, character) when
next is called. The index corresponds to the index that this character is in
in the whole string.

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
    self.iter.by_ref()
                .peeking_take_while(|(_,x)| x.is_whitespace())
                .count()
}

pub fn lex(&mut self) -> Result<Token, &str> {
    self.skip_whitespace();
```

First, `self.iter` gets our iterator, which is a `Peekable<CharIndices<'a>>`. That just means that it's an iterator that implements all
the normal iterator methods as well as the `peek()` method, and it is wrapped around the `Chars<'a>` iterator. Since we're not 
using `peek()` in this function, you can imagine that `self.iter` is a `Iterator<char>`.

The `peeking_take_while` method is an iterator method defined in the `itertools` crate
that takes a predicate (a closure that takes one elemtnt and returns a boolean)
and returns a new iterator, a `PeekingTakeWhile<I>`. A PeekingTakeWhile is like a Peekable, it's a normal iterator, but slightly different. In
this case, a PeekingTakeWhile will stop (return `None` on `next()`) once the predicate is false.

This is different than the Rust standard library `take_while`, because that TakeWhile
goes one too far. After the predicate returns false, it will discard it.


Finally, the `count()` method keeps calling `next()` on the `TakeWhile` iterator until `None` is reached, and then returns the number
of things it skipped

### Check for EOF

Let's also check that we haven't reached the end of the input. If we peek and
there's nothing there, then there's no more tokens to lex, so we return
the EOF token

### Next Identifier

Now that we've skipped the whitespace, we need to check for a real token.
The first one we'll look at is identifiers. Identifiers are just strings of
characters. They could be variables, functions, or keywords.

Let's start by finding keywords first.

So, every identifier starts with a letter, so let's match on the current
character, and if it's a letter try to parse the ident

```rust
let result = match just_char(self.iter.peek().unwrap()) {
    'a' ... 'z' | 'A' ... 'Z' => {
        // parse ident
        // ...
    },
    _ => { return Err("not implemented yet!"); }
};
```

Since we only care about matching on the character and not the index, 
we will define a function `just_char` that takes a reference to the 
char index and returns just the character

```rust
// Outside of the Lexer impl
fn just_char((i, c) : &(usize, char)) -> char {
    *c
}
```

Now we're going to parse the whole identifier and check if it's a keyword
or not using a function we haven't written yet, `next_ident`

```rust
// fn next_ident(&mut self) -> &str

let result = match just_char(self.iter.peek().unwrap()) {
    'a' ... 'z' | 'A' ... 'Z' => {
        let next_ident = self.next_ident();
        match next_ident {
            "def" => Def,
            "extern" => Extern,
            ident => Ident(ident.to_owned()),
        }
    },
    _ => { return Err("not implemented yet!"); }
};
```

Note that even if it's not a keyword, we know it's an ident, so we just
wrap it in the Ident token and return it.

### Parsing the Identifier

Let's define that `next_ident`. It needs to return a `&str` that exactly
matches the next identifier, it can't have any extra characters at the end.

Also, remember that we don't want to consume too many characters like in
 eat_whitespace. Because of that, it's going to look similar

```rust
fn next_ident(&mut self) -> &str {
    let (start, _) = *self.iter.peek().unwrap();
    let (up_to, last) = self.iter.by_ref()
                        .peeking_take_while(|(_,c)| c.is_alphabetic())
                        .last().unwrap();
    &self.input[start .. (up_to + last.len_utf8())]
}
```

Remember that `peeking_take_while`? We're using it again, this time consuming
alphabetic characters. But we're using `last()` instead of `count()`

Couldn't we just use `.count()` like before? Well, this would have caused errors
if we took Unicode input, because characters can be longer than 1 byte, but
the index into `&str` is by bytes not characters.

Basically, we take the index of the last character and add its length. 
Importantly, the index returned by CharIndices is in bytes, not number 
of characters. Note that we're using the `..` syntax which is right 
exclusive (like `[)` in math)

### Eating Comments

Use `take_while` this time instead of `peeking_take_while`

Recursion

### Reading Numbers

This is very similar to reading idents, but after we've collected the part of the
input we want to read, we parse it using `str::parse()` and then process the
result so that we create a Token on Ok and a string on Err

### Tests

Remember when we tried avoiding unicode problems with identifiers using that fold?

It would have compiled just fine with `count()`, but failed at runtime when given
unicode. That's something we definitely want to avoid at all costs in Rust. 

A good way to do a sanity check on whether we even *need* that extra effort to make
it unicode safe, however, is to write some tests and see if they fail.

I've written some for this chapter in a file I named `test.rs`, but most importantly
I added the command `mod test;` to my `main.rs`, so cargo knows to build the file
`test.rs`

In fact, when I tested this I realized that the identifiers still won't work with
unicode unless they start with an alphabetic character. That's actually pretty lame.
I fixed it by checking in the last branch of the match if the character was 
alphabetic and then returning an ident if it was.

Writing those tests helps you figure out where the holes in your code are.