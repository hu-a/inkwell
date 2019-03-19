#[cfg(test)]
mod lex_tests {
    use ::*;
    use ::Token::*;

    #[test]
    fn call_lexer() {
        assert_eq!(lex_def("def bar"), Ok((" bar", Def)));
    }

    #[test]
    fn test_ident() {
        assert_eq!(lex_ident("hello world"), Ok((" world", Ident("hello".to_owned()))));
    }

    #[test]
    fn test_unicode() {
        assert_eq!(lex_ident("松本 行弘"), Ok((" 行弘", Ident("松本".to_owned()))));
    }

    #[test]
    fn test_tok() {
        assert_eq!(lex_tok("def bar"), Ok(("bar", Def)));
        assert_eq!(lex_tok("extern bar"), Ok(("bar", Extern)));
        assert_eq!(lex_tok("foo bar"), Ok(("bar", Ident("foo".to_owned()))));
        assert_eq!(lex_tok("3.14 bar"), Ok(("bar", Num(3.14))));
        assert_eq!(lex_tok("# comment \n 3.14 bar"), Ok(("bar", Num(3.14))));
        assert_eq!(lex_tok("+ bar"), Ok(("bar", Op('+'))));
        assert_eq!(lex_tok("$ bar"), Ok(("bar", Op('$'))));
    }

    #[test]
    fn test_line() {
        assert_eq!(lex_line("def inc(x) x + 1;"), Ok(("", vec![Def, Ident("inc".to_owned()), LeftParen, Ident("x".to_owned()), RightParen
                                                                , Ident("x".to_owned()), Op('+'), Num(1.0)])));
    }
}