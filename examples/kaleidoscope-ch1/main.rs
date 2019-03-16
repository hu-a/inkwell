//! Chapter 1

extern crate itertools;

use std::str::CharIndices;
use std::iter::Peekable;

use itertools::Itertools;

use Token::*;

/// Represents a primitive syntax token
#[derive(Debug, PartialEq)]
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

pub struct Lexer<'a> {
    input : &'a str,
    iter : Peekable<CharIndices<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input_to_ref : &'a str) -> Lexer<'a> {
        Lexer { input: input_to_ref, 
                iter: input_to_ref.char_indices().peekable() }
    }

    /// Skips all whitespace the Lexer is currently pointing to and returns 
    /// the number of whitespace characters processed
    pub fn skip_whitespace(&mut self) -> usize {
        self.iter.by_ref()
                    .peeking_take_while(|(_,x)| x.is_whitespace())
                    .count()
    }

    /// Takes a string starting with an alphabetic character and returns
    /// a substring from 0th index until the last contiguous alphabetic char
    fn next_ident(&mut self) -> &str {
        let (start, _) = *self.iter.peek().unwrap();
        let len = self.iter.by_ref()
                            .peeking_take_while(|(_,c)| c.is_alphabetic())
                            .fold(0, |acc, (i,c)| acc + c.len_utf8());
        &self.input[start .. (start + len)]
    }

    fn just_char((i, c) : &(usize, char)) -> &char {
        c
    }

    /// Removes everything from '#' up to and including '\n'
    /// Returns number of bytes read
    /// Pre: self.iter.peek() returns '#'
    fn eat_comment(&mut self) -> usize {
        self.iter.by_ref()
                    .take_while(|(_,c)| *c != '\n')
                    .fold(0, |acc, (i,c)| acc + c.len_utf8())
    }

    /// Tries to lex the next number. If parsing fails, Err is returned
    /// Pre: self.iter.peek()  is in '0' ... '9'
    fn next_num(&mut self) -> Result<Token, &str> {
        let (start, _) = *self.iter.peek().unwrap();
        let len = self.iter.by_ref()
                            .peeking_take_while(|(_,c)| c.is_numeric() || *c == '.')
                            .fold(0, |acc, (i,c)| acc + c.len_utf8());
        match self.input[start .. (start + len)].parse::<f64>() {
            Ok(num) => Ok(Number(num)),
            Err(_) => Err("Could not parse this number")
        }
    }

    /// Tries to parse the next Token, skipping whitespace and comments
    pub fn lex(&mut self) -> Result<Token, &str> {
        self.skip_whitespace();

        if self.iter.peek().is_none() {
            return Ok(EOF);
        }

        let result = match Lexer::just_char(self.iter.peek().unwrap()) {
            '#' => {
                self.eat_comment();
                return self.lex();
            },
            'a' ... 'z' | 'A' ... 'Z' => {
                let next_ident = self.next_ident();
                match next_ident {
                    "def" => Def,
                    "extern" => Extern,
                    ident => Ident(ident.to_owned()),
                }
            },
            '0' ... '9' => {
                return self.next_num();
            }
            _ => { return Err("not implemented yet!"); }
        };

        Ok(result)
    }

    pub fn test_peek(&mut self) -> &char {
        Lexer::just_char(self.iter.peek().unwrap())
    }
}

fn main() {
    let mut l = Lexer::new("        def #    \n extern  3.14");
    println!("Skipping the whitespace: {}", l.skip_whitespace());
    println!("Peek: {}", l.test_peek());
    println!("Peek: {}", l.test_peek());
    println!("Next token: {:?}", l.lex());
    println!("Next token: {:?}", l.lex());
    println!("Next token: {:?}", l.lex());
}

#[cfg(test)]
mod test;