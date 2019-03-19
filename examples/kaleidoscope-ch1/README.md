# Chapter 1

This is a tutorial on using the Inkwell bindings to create a simple language,
Kaleidoscope, which is based on the official LLVM Kaleidoscope tutorial.

Note, this version uses the `nom` crate for lexing and parsing instead of writing
and hand-written lexer/parser like the C++ tutorial. Although, the OCaml tutorial
also uses a parser combinator library, so I guess it'll be closer to that. (Still
haven't learned OCaml yet, though, so I can't tell you how similar it is in design)

## Kaleidoscope Language

The Kaleidoscope language is a simple functional language where the only
primitive is floating point numbers (no first-class functions for now)

Here's an example

```
def fib(x)
  if x < 2
    x
  else
    fib(x - 1) + fib(x - 2)
;
```

It looks kind of similar to an ML family language except that all functional
invocations require parentheses in an imperitive style invocations

## Lexing

The first step to implementing our language is breaking up the text into tokens

```rust
pub enum Token {
    EOF,
    // commands
    Def,
    Extern,
    // any string
    Ident(String),
    // our primitive value
    Num(f64),
    // punctuation
    LeftParen,
    RightParen,
    Comma,
    // if unknown
    Op(char),
}
```

### Nom Basics

Instead of writing our own lexer, we're going to use the `nom` crate. This should
be bundled with inkwell, but if it isn't, put `nom = 4.2.4` in your Cargo.toml

First thing we need in our `main.rs` is 

```rust
extern crate nom;

use nom::*;
```

The way that nom works is that you use macros to define parsers. The `named!` macro
creates a parser like so

```rust
named!(lex_foo<&str, &str>, tag!("foo"));
//     ^^^^^^^-- parser name
//             ^^^^-- input type
//                   ^^^^^-- output type (before wrapping in IResult)
//                          ^^^^^^^^^^^-- body

fn call_lexer() {
  assert_eq!(lex_foo("foo bar"), Ok((" bar", "foo")));
}
```

The `tag!` macro recognizes a string exactly. So, since we have the `def` keyword in our language, we should be able
to create a lexer to match `def`, and then transform it into the Token representing Def like so:

```rust
named!(lex_def<&str,Token>, map!(tag!("def"), |_| Def));
```

First, notice that we changed the output type to Token. Next, we wrapped the tag! in a map!. What this does is
*if* the tag!("def") succeeds, then it calls the function in the second argument with the result of the first parser.

Our second argument is the closure `|_| Def`, which takes one argument, throws it away with the pattern `_` and returns
the value Def.

## Testing

We want to make sure that our lexer works as we want it to. So, let's write some tests in a separate file.

At the bottom of main.rs, I added the lines

```rust
#[cfg(test)]
mod test;
```

The `mod test;` means to find the file `test.rs` and import its contents, basically. The directive `#[cfg(test)]` means
to only compile this is running with `cargo test`

Now, we need to make a new file `test.rs`. In it I put this:

```rust
#[cfg(test)]
mod lex_tests {
    use ::*;
    use ::Token::Def;

    #[test]
    fn call_lexer() {
        assert_eq!(lex_def("def bar"), Ok((" bar", Def)));
    }
}
```

## Lexing Numbers and Identifiers

This is similar to how we lexed `def`, but we can use the built in `alpha` and `double` parsers to recognize them

```rust
named!(lex_num<&str,Token>, map!(double, |d| Num(d)));
named!(lex_ident<&str,Token>, map!(alpha, |s| Ident(s.to_owned())));
```

alpha takes a &str as input and Ok()s when it hits the end of the first sequence of alphabetic characters, including
Unicode! To turn this into an Ident(), we need to actually use the value recognized by alpha. That is, the slice
at the start of the input we gave it that is an identifier. So, we promote it to owned and wrap it in the Ident() token.

Remember to add tests! In particular, for this one, I wanted to make sure that unicode identifiers could be recognized

TODO: Make note about nom incomplete input

## Lexing One Token

Now that we have a couple lexers for different tokens, let's try to create a lexer that can recognize any of the tokens
that we've defined so far

```rust
named!(lex_tok<&str,Token>,
    ws!(alt!(
        lex_def |
        lex_extern |
        lex_num |
        lex_ident |
    ))
);
```

First thing to notice is that it's bigger than the rest, and there's these pipe characters at the end of some of the lines.

The pipe characters are a part of the alt! macro. Rust macros are cool because they can allow you to create your own syntax
to make an embedded domain specific language, which is great for parsers because we can make it look like a BNF grammar
where the production rules(?) are separated by pipes

Anyway, alt! let's you try the first parser and if it doesn't work, try the next one, and so on. So, this lex_tok can
parse either a def, an extern, a num, or an ident.

Finally, we wrap it in ws! which allows any of the parsers it surrounds to be separated by whitespace.

Of course, let's write a test for it! How would you write this test? I just copied the lex_def test, but made
four different ones. It mostly worked, except I forgot about the whitespace removal, so I had to adjust the remainder
in the Ok()

## Lexing Punctuation

There are three character in punctuation: left paren, right paren, and comma. These are so simple, we can just throw them
into our lex_tok definition

```rust
named!(lex_tok<&str,Token>,
    ws!(alt!(
        lex_def |
        lex_extern |
        map!(tag!("("), |_| LeftParen) |
        map!(tag!(")"), |_| RightParen) |
        map!(tag!(","), |_| Comma) |
        lex_num |
        lex_ident |
    ))
);
```

## Lexing Comments

Since the end of a function definition in our Kaleidoscope is indicated by a semicolon, like in an SML toplevel, we're
going to allow commented lines (unlike in SML). These are just from the "#" character until the "\n" character, delete
everything.

So, the interesting thing about this is... no token is generated.

```rust
named!(lex_comment<&str,()>,
    do_parse!(
        tag!("#") >>
        take_until_and_consume!("\n") >>
        (())
    )
);
```

This is the introduction of the do_parse! macro which will allow us again more embedded DSL syntax to do multiple
parses in sequence. The last line just means return whatever's inside of the tuple, and I put unit, so it looks kinda funny

But how will we integrate this into our lex_tok? Previously, everything returned a token, but this one doesn't. The
answer is to use do_parse again!

```rust
named!(lex_tok<&str,Token>,
    ws!(alt!(
        lex_def |
        lex_extern |
        map!(tag!("("), |_| LeftParen) |
        map!(tag!(")"), |_| RightParen) |
        map!(tag!(","), |_| Comma) |
        lex_num |
        lex_ident |
        do_parse!(lex_comment >> res: lex_tok >> (res)) |
    ))
);
```

Again more syntax in this do_parse. The `res: lex_tok` binds the variable `res` to the result of recursively calling
lex_tok. This looks pretty weird though, so if anybody has any recommendations on making this better, please tell me!

More tests! This time, I just added another line to my old test_tok

## Lexing an Infix Operator

In our language, an operator is any single character that isn't anything else. This is surprisingly simple to implement,
we just need to make sure taht we insert it into lex_tok in the right place

```rust
named!(lex_op<&str,Token>, map!(anychar, |c| Op(c)));

named!(lex_tok<&str,Token>,
    ws!(alt!(
        lex_def |
        lex_extern |
        map!(tag!("("), |_| LeftParen) |
        map!(tag!(")"), |_| RightParen) |
        map!(tag!(","), |_| Comma) |
        lex_num |
        lex_ident |
        do_parse!(lex_comment >> res: lex_tok >> (res)) |
        lex_op
    ))
);
```

Make sure it goes at the very end.

Do more testing!

## Lexing a Whole Line

Finally, let's put this all together and parse an entire line! Remember in our language, a definition ends with
a semicolon, so we can basically parse many tokens until we hit a semicolon. And that's exactly the parser:

```rust
named!(lex_line<&str,Vec<Token>>,
    map!(many_till!(lex_tok, tag!(";")), |(res,_)| res)
);
```

Note that many_till returns a tuple, but we only care about the left

Add more tests!

## Lexing REPL

Finally, we can make our REPL! It only lexes the tokens right now, but it's pretty impressive that we can do all this
in under 100 LoC.

```rust
fn main() {
    loop {
        println!(">");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        while !input.contains(";") {
            let mut next_line = String::new();
            std::io::stdin().read_line(&mut next_line).unwrap();
            input.push_str(next_line.as_str());
        }

        println!("lexing: {:?}", lex_line(input.as_str()));
    }
}
```