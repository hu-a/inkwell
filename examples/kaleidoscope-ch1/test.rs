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
        assert_eq!(l.lex().unwrap(), Def);
        assert_eq!(l.lex().unwrap(), EOF);
    }
}