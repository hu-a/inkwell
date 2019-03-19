//! Chapter 1

extern crate itertools;
extern crate core;
extern crate inkwell;

use std::str::CharIndices;
use std::iter::Peekable;
use std::vec::IntoIter;

use itertools::Itertools;

use Token::*;
use Expr::*;
use core::fmt::Alignment::{Right, Left};
use inkwell::execution_engine::FunctionLookupError::FunctionNotFound;

/// Represents a primitive syntax token
#[derive(Debug, PartialEq, Clone)]
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

fn just_char((i, c): &(usize, char)) -> char {
    *c
}

pub struct Lexer<'a> {
    input: &'a str,
    iter: Peekable<CharIndices<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input_to_ref: &'a str) -> Lexer<'a> {
        Lexer {
            input: input_to_ref,
            iter: input_to_ref.char_indices().peekable(),
        }
    }

    /// Skips all whitespace the Lexer is currently pointing to and returns 
    /// the number of whitespace characters processed
    pub fn skip_whitespace(&mut self) -> usize {
        self.iter.by_ref()
            .peeking_take_while(|(_, x)| x.is_whitespace())
            .count()
    }

    /// Takes a string starting with an alphabetic character and returns
    /// a substring from 0th index until the last contiguous alphabetic char
    fn next_ident(&mut self) -> &str {
        let (start, _) = *self.iter.peek().unwrap();
        let (up_to, last) = self.iter.by_ref()
            .peeking_take_while(|(_, c)| c.is_alphabetic())
            .last().unwrap();
        &self.input[start..(up_to + last.len_utf8())]
    }

    /// Removes everything from '#' up to and including '\n'
    /// Returns number of bytes read
    /// Pre: self.iter.peek() returns '#'
    fn eat_comment(&mut self) -> usize {
        self.iter.by_ref()
            .take_while(|(_, c)| *c != '\n')
            .fold(0, |acc, (i, c)| acc + c.len_utf8())
    }

    /// Tries to lex the next number. If parsing fails, Err is returned
    /// Pre: self.iter.peek()  is in '0' ... '9'
    fn next_num(&mut self) -> Result<Token, &str> {
        let (start, _) = *self.iter.peek().unwrap();
        let len = self.iter.by_ref()
            .peeking_take_while(|(_, c)| c.is_numeric() || *c == '.')
            .fold(0, |acc, (i, c)| acc + c.len_utf8());
        match self.input[start..(start + len)].parse::<f64>() {
            Ok(n) => Ok(Num(n)),
            Err(_) => Err("Could not parse this number")
        }
    }

    /// Tries to parse the next Token, skipping whitespace and comments
    pub fn lex(&mut self) -> Result<Token, &str> {
        self.skip_whitespace();

        if self.iter.peek().is_none() {
            return Ok(EOF);
        }

        let result = match just_char(self.iter.peek().unwrap()) {
            '#' => {
                self.eat_comment();
                self.lex()?
            }
            '(' => {
                self.iter.next();
                LeftParen
            }
            ')' => {
                self.iter.next();
                RightParen
            }
            ',' => {
                self.iter.next();
                Comma
            }
            'a'...'z' | 'A'...'Z' => {
                let next_ident = self.next_ident();
                match next_ident {
                    "def" => Def,
                    "extern" => Extern,
                    ident => Ident(ident.to_owned()),
                }
            }
            '0'...'9' => {
                return self.next_num();
            }
            other => {
                if other.is_alphabetic() {
                    Ident(self.next_ident().to_owned())
                } else {
                    // assume for now that it is an infix operator
                    self.iter.next();
                    Op(other)
                }
            }
        };

        Ok(result)
    }

    pub fn test_peek(&mut self) -> char {
        just_char(self.iter.peek().unwrap())
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    /// Lexes the next `Token` and returns it.
    /// On EOF or failure, `None` will be returned.
    fn next(&mut self) -> Option<Self::Item> {
        match self.lex() {
            Ok(EOF) |
            Err(_) => None,
            Ok(token) => Some(token)
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    /// Numeric literal
    Number(f64),
    /// Variable: any ident not a keyword or function call
    Variable(String),
    /// Infix operator
    Binop(Box<Expr>, char, Box<Expr>),
    /// Function call
    Call(String, Vec<Expr>),
}

#[derive(Debug)]
pub struct Prototype {
    name: String,
    params: Vec<String>,
}

#[derive(Debug)]
pub struct FunctionDef {
    proto: Prototype,
    body: Expr,
}

pub struct Parser {
    iter: Peekable<IntoIter<Token>>,
}

impl Parser {
    pub fn new(input: String) -> Parser {
        let lex = Lexer::new(input.as_str());
        let tokens = lex.collect::<Vec<Token>>();
        Parser::from_tokens(tokens)
    }

    fn from_tokens(tok: Vec<Token>) -> Parser {
        Parser {
            iter: tok.into_iter().peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<FunctionDef, &str> {
        match self.curr()? {
            Def => self.parse_def(),
            Extern => self.parse_extern(),
            _ => self.parse_top_level()
        }
    }

    fn parse_def(&mut self) -> Result<FunctionDef, &str> {
        self.consume(Def);
        let proto = self.parse_prototype()?;
        let body = self.parse_expr()?;
        Ok(FunctionDef { proto, body })
    }

    fn parse_extern(&mut self) -> Result<FunctionDef, &str> {
        self.consume(Extern);
        let proto = self.parse_prototype()?;
        Ok(FunctionDef {
            proto: proto,
            body: Number(std::f64::NAN)
        })
    }

    fn parse_top_level(&mut self) -> Result<FunctionDef, &str> {
        Ok(FunctionDef {
            proto: Prototype {
                name: "anonymous".to_owned(),
                params: Vec::default(),
            },
            body: self.parse_expr()?,
        })
    }

    fn parse_number(&mut self) -> Result<Expr, &'static str> {
        match self.iter.next().unwrap() {
            Num(n) => Ok(Number(n)),
            _ => Err("expected number")
        }
    }

    /// Parses a parenthetical expression
    /// Pre: current token in left paren
    fn parse_paren(&mut self) -> Result<Expr, &'static str> {
        self.consume(LeftParen);
        let res = self.parse_expr();
        self.consume(RightParen);
        res
    }

    /// Parses function call arguments in tuple-ish syntax e.g. (a, 5 + 5, bar(42))
    fn parse_call_arguments(&mut self) -> Result<Vec<Expr>, &'static str> {
        self.consume(LeftParen);

        let mut args = Vec::new();

        // immediately encoutering right paren means no arguments, else loop to get args
        if self.curr()? == RightParen {
            self.consume(RightParen);
        } else {
            loop {
                args.push(self.parse_expr()?);
                if self.curr()? == Comma {
                    self.consume(Comma);
                } else {
                    // No comma after expression means the end of a comma separated list
                    self.consume(RightParen);
                    break;
                }
            }
        }
        Ok(args)
    }

    fn parse_id(&mut self) -> Result<Expr, &'static str> {
        let s = match self.iter.next().unwrap() {
            Ident(s) => s,
            _ => { return Err("Expected ident token"); }
        };

        if self.curr()? == LeftParen {
            // Function call
            let args = self.parse_call_arguments()?;
            Ok(Call(s, args))
        } else {
            // variable
            Ok(Variable(s))
        }
    }

    /// Errors if at end of input
    fn check_eof(&mut self) -> Result<(), &'static str> {
        match self.iter.peek() {
            None => { debug_assert!(false); Err("Unexpected end of tokens, no EOF") },
            Some(EOF) => { debug_assert!(false); Err("Unexpected EOF token") },
            _ => Ok(())
        }
    }

    /// Gets a clone of the current token
    /// Assumes that check_eof() is Ok(()), in debug mode will panic otherwise
    fn curr(&mut self) -> Result<Token, &'static str> {
        debug_assert!(self.check_eof()? == ());
        Ok(self.iter.peek().unwrap().clone())
    }

    fn parse_primary(&mut self) -> Result<Expr, &'static str> {
        match self.curr()? {
            Num(n) => self.parse_number(),
            LeftParen => self.parse_paren(),
            Ident(_) => self.parse_id(),
            _ => Err("not a primary expression")
        }
    }

    fn get_tok_precedence(&mut self) -> i32 {
        let op = match self.iter.peek() {
            Some(Op(op)) => op,
            _ => { return -1; }
        };
        match op {
            '<' => 10,
            '+' => 20,
            '-' => 20,
            '*' => 40,
            _ => -1
        }
    }

    /// Parses a series of (operator, primary expression) pairs, given a left hand side
    /// Examples: (remember the deeper nodes get evaluated first)
    /// 1 + 2     => (1,+,2)
    /// 1 + 2 + 3 => ((1,+,2),+,3)
    /// 1 + 2 * 3 => (1,+,(2,*,3))
    fn parse_binary(&mut self, left_prec: i32, mut left: Expr) -> Result<Expr, &'static str> {
        loop {
            let curr_prec = self.get_tok_precedence();

            if curr_prec < left_prec {
                return Ok(left);
            }

            let op = match self.curr()? {
                Op(op) => op,
                _ => return Err("Invalid operator.")
            };

            self.consume(Op(op));

            let mut right = self.parse_primary()?;

            let next_prec = self.get_tok_precedence();

            if curr_prec < next_prec {
                right = self.parse_binary(curr_prec + 1, right)?;
            }

            left = Binop(Box::from(left)
                         , op
                         , Box::from(right));
        }
    }

    /// Parse a single expression of any kind
    /// An expression is a primary expression followed by a series of
    /// (operator, primary expression) pairs
    pub fn parse_expr(&mut self) -> Result<Expr, &'static str> {
        match self.parse_primary() {
            Ok(left) => self.parse_binary(0, left),
            err => err
        }
    }

    fn parse_prototype(&mut self) -> Result<Prototype, &'static str> {
        let name = match self.curr()? {
            Ident(name) => name,
            _ => { return Err("expected identifier at start of function prototype"); }
        };
        self.consume(Ident(name.clone()));
        self.consume(LeftParen);

        let mut params = Vec::new();
        loop {
            let param_name = match self.curr()? {
                Ident(name) => name,
                RightParen => { self.consume(RightParen); break; }
                Comma => { return Err("THere's actually no commas in the parameter list"); },
                _ => { return Err("expected string for parameter name"); }
            };
            self.consume(Ident(param_name.clone()));
            params.push(param_name);
        }
        Ok(Prototype{ name, params })
    }

    /// Move the iterator forward without using its result
    fn consume(&mut self, expected: Token) {
        let actual = self.iter.next().unwrap();
        debug_assert!(actual == expected);
    }
}

fn repl() {
    loop {
        println!(">");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input);

        {
            let tokens = Lexer::new(&input).collect::<Vec<Token>>();
            if let Some(EOF) = tokens.get(0) {
                break;
            }
            println!("Lexed: {:?}", tokens);
        }

        let mut p = Parser::new(input);
        println!("Parsed: {:?}", p.parse());
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

    let mut p = Parser::new("   3.14 (42) 333".to_owned());
    println!("AST: {:?}", p.parse());
    println!("Number: {:?}", p.parse_number());

    repl();
}

#[cfg(test)]
mod test;