#[cfg(test)]
mod tests {
    use ::*;

    #[test]
    fn skip_beginning_whitespace() {
        let mut l = Lexer::new("  def");
        assert_eq!(l.skip_whitespace(), 2);
        assert_eq!(l.skip_whitespace(), 0);
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
}