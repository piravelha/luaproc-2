use crate::lexer::*;

#[cfg(test)]
mod tests {
  fn new_token(kind: TokenKind, value: &str) -> Token {
    Token {
      kind,
      value: value.to_string(),
      location: Location {
        file: "".to_string(),
        line: 0,
        column: 0,
      },
    }
  }

  #[test]
  fn test_replace_tokens() {
    let tokens = lex("<stdin>", "
      print(sum)
    ");
    let old = new_token(TokenKind::Name, "sum");
    let new = vec![
      new_token(TokenKind::Number, "1"),
      new_token(TokenKind::Special, "+"),
      new_token(TokenKind::Number, "2"),
    ];
    let result = replace_tokens(tokens, old, new);
    assert_eq!(
      render_tokens(result),
      "print ( 1 + 2 )",
    );
  }
}


