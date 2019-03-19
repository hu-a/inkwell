//! Chapter 1

extern crate itertools;
extern crate nom;

use Token::*;

use nom::*;

named!(lex_def<&str,Token>, map!(tag!("def"), |_| Def));
named!(lex_extern<&str,Token>, map!(tag!("extern"), |_| Extern));
named!(lex_num<&str,Token>, map!(double, |d| Num(d)));
named!(lex_ident<&str,Token>, map!(alpha, |s| Ident(s.to_owned())));
named!(lex_op<&str,Token>, map!(anychar, |c| Op(c)));
named!(lex_comment<&str,()>, 
    do_parse!(
        tag!("#") >>
        take_until_and_consume!("\n") >>
        (())
    )
);
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
named!(lex_line<&str,Vec<Token>>,
    map!(many_till!(lex_tok, tag!(";")), |(res,_)| res)
);

/// Represents a primitive syntax token
#[derive(Debug, PartialEq)]
pub enum Token {
    EOF,

    // commands
    Def,
    Extern,

    // primary
    Ident(String),
    Num(f64),
    
    LeftParen,
    RightParen,
    Comma,

    // if unknown
    Op(char),
}

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

#[cfg(test)]
mod test;