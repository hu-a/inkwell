#[cfg(test)]
mod tests {
    use ::*;
    use ::Expr::{Number, Variable, Call, Binop};
    use ::Token::{LeftParen, Num, RightParen, Comma, EOF, Ident};

    #[test]
    fn skip_beginning_whitespace() {
        let mut l = Lexer::new("  def");
        assert_eq!(l.skip_whitespace(), 2);
        assert_eq!(l.skip_whitespace(), 0);
    }

    #[test]
    fn variable_eof() {
        let mut l = Lexer::new("a");
        assert_eq!(l.lex(), Ok(Ident("a".to_owned())));
        assert_eq!(l.lex(), Ok(EOF));
    }

    #[test]
    fn identify_def() {
        let mut l = Lexer::new("  def");
        assert_eq!(l.lex(), Ok(Def));
        assert_eq!(l.lex(), Ok(EOF));
    }

    #[test]
    fn unicode_ident_start_ascii() {
        let mut l = Lexer::new("  matz松本 行弘");
        assert_eq!(l.lex(), Ok(Ident("matz松本".to_owned())));
    }

    #[test]
    fn unicode_ident() {
        let mut l = Lexer::new("  松本 行弘");
        assert_eq!(l.lex(), Ok(Ident("松本".to_owned())));
    }

    #[test]
    fn lex_parens() {
        let mut l = Lexer::new("   (  5 , )  ");
        assert_eq!(l.lex(), Ok(LeftParen));
        assert_eq!(l.lex(), Ok(Num(5.0)));
        assert_eq!(l.lex(), Ok(Comma));
        assert_eq!(l.lex(), Ok(RightParen));
    }

    #[test]
    fn parse_var() {
        let mut p = Parser::new("a foo()".to_owned());
        assert_eq!(p.parse_expr(), Ok(Variable("a".to_owned())));
        assert_eq!(p.parse_expr(), Ok(Call("foo".to_owned(), vec![])));
    }

    #[test]
    fn lex_op() {
        let l = Lexer::new("5 + 5");
        assert_eq!(l.collect::<Vec<Token>>(), vec![Num(5.0), Op('+'), Num(5.0)]);
    }

    #[test]
    fn parse_call_args() {
        let mut p = Parser::new("foo() bar(a) baz(a, b, c)".to_owned());
        assert_eq!(p.parse_expr(), Ok(Call("foo".to_owned(), vec![])));
        assert_eq!(p.parse_expr(), Ok(Call("bar".to_owned(), vec![Variable("a".to_owned())])));
        assert_eq!(p.parse_expr(), Ok(Call("baz".to_owned(), vec![Variable("a".to_owned())
                                                                  , Variable("b".to_owned())
                                                                  , Variable("c".to_owned())])));
    }

    fn make_binop(left : Expr, op : char, right : Expr) -> Expr {
        Binop (Box::from(left)
               ,op
               , Box::from(right))
    }

    #[test]
    fn parse_binop() {
        let mut p = Parser::new("1 + 2    1 + 2 + 3    1 + 2 * 3".to_owned());
        assert_eq!(p.parse_expr(), Ok(Binop(Box::from(Number(1.0)), '+', Box::from(Number(2.0)))));
        assert_eq!(p.parse_expr(), Ok(Binop(Box::from(Binop(Box::from(Number(1.0)), '+', Box::from(Number(2.0))))
                                            ,'+', Box::from(Number(3.0)))));
        assert_eq!(p.parse_expr(), Ok(make_binop(Number(1.0),'+', make_binop(Number(2.0), '*', Number(3.0)))));
    }

    #[test]
    fn parser_basic() {
        let mut p = Parser::new("  1234 (5) a 5 * 2".to_owned());
        assert_eq!(p.parse_number(), Ok(Number(1234.0)));
        assert_eq!(p.parse_paren(), Ok(Number(5.0)));
        assert_eq!(p.parse_expr(), Ok(Variable("a".to_owned())));
    }

    #[test]
    fn top_level() {
        let mut p = Parser::new("5 + a".to_owned());
        assert_eq!(p.parse_expr(), Ok(make_binop(Number(5.0), '+', Variable("a".to_string()))));
    }
}