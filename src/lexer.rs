use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
  Number,
  String,
  Boolean,
  Nil,
  Name,
  Special,
  Delimiter,
  Brace,
  Macro,
  Include,
  Define,
  EndDefine,
  Undef,
  Ifdef,
  Ifndef,
  Endif,
  Else,
  Stringify,
  Paste,
  Vararg,
  StringifyVararg,
}

#[derive(Debug, Clone)]
pub struct Location {
  pub file: String,
  pub line: i32,
  pub column: i32,
}

#[derive(Debug, Clone)]
pub struct Token {
  pub kind: TokenKind,
  pub value: String,
  pub location: Location,
}

pub type Patterns = Vec<(Regex, TokenKind)>;
pub type Tokens = Vec<Token>;

fn new_pattern(pattern: &str) -> Regex {
  Regex::new(&("^".to_string() + pattern)).unwrap()
}

fn get_lex_patterns() -> Patterns {
  vec![
    (new_pattern(r"-?\d+(\.\d+)?"), TokenKind::Number),
    (new_pattern(r#""([^"\\]|\\.)*""#), TokenKind::String),
    (new_pattern(r"(true|false)"), TokenKind::Boolean),
    (new_pattern(r"([a-zA-Z_]\w*!)"), TokenKind::Macro),
    (new_pattern(r"(#include)"), TokenKind::Include),
    (new_pattern(r"(#ifndef)"), TokenKind::Ifndef),
    (new_pattern(r"(#ifdef)"), TokenKind::Ifdef),
    (new_pattern(r"(#endif)"), TokenKind::Endif),
    (new_pattern(r"(#else)"), TokenKind::Else),
    (new_pattern(r"(#define)"), TokenKind::Define),
    (new_pattern(r"(#end)"), TokenKind::EndDefine),
    (new_pattern(r"(#undef)"), TokenKind::Undef),
    (new_pattern(r"(nil)"), TokenKind::Nil),
    (new_pattern(r"(#[a-zA-Z_]\w*#)"), TokenKind::Stringify),
    (new_pattern(r"##"), TokenKind::Paste),
    (new_pattern(r"([a-zA-Z_]\w*)"), TokenKind::Name),
    (new_pattern(r"#\.\.\.#"), TokenKind::StringifyVararg),
    (new_pattern(r"#\.\.\."), TokenKind::Vararg),
    (
      new_pattern(r"([+\-*/!@#$%&|:<>=?~^.]+)"),
      TokenKind::Special,
    ),
    (new_pattern(r"[,;]"), TokenKind::Delimiter),
    (new_pattern(r"[()\[\]{}]"), TokenKind::Brace),
  ]
}

fn update_location(
  input: &str,
  location: &mut Location,
) {
  for char in input.chars() {
    if char == '\n' {
      location.line += 1;
      location.column = 1;
    } else {
      location.column += 1;
    }
  }
}

fn apply_pattern(
  pattern: Regex,
  kind: TokenKind,
  input: &str,
  location: &mut Location,
) -> Option<(Token, String)> {
  let capture = pattern.captures(input)?;
  let full = &capture[0];
  let token = Token {
    kind: kind.clone(),
    value: full.to_string(),
    location: location.clone(),
  };
  update_location(&input[..full.len()], location);
  return Some((token, input[full.len()..].to_string()));
}

fn apply_patterns(
  patterns: &Patterns,
  input: &str,
  location: &mut Location,
) -> Option<(Token, String)> {
  for (pattern, kind) in patterns {
    let result = apply_pattern(
      pattern.clone(),
      kind.clone(),
      input,
      location,
    );
    match result {
      None => continue,
      some @ Some(_) => return some,
    }
  }
  None
}

pub fn lex(file: &str, mut input: String) -> Option<Tokens> {
  input = Regex::new("--.*")
    .unwrap()
    .replace_all(&input, "")
    .to_string();
  let patterns = get_lex_patterns();
  let mut tokens = vec![];

  let mut location = Location {
    file: file.to_string(),
    line: 1,
    column: 1
  };

  while !input.is_empty() {
    update_location(
      &input[..input.len() - input.trim_start().len()],
      &mut location
    );
    input = input.trim_start().to_string();
    if input.is_empty() {
      break;
    }
    let (token, new_input) = apply_patterns(&patterns, &input, &mut location)?;
    input = new_input;
    tokens.push(token);
  }

  Some(tokens)
}
