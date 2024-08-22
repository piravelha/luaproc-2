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
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub value: String,
}

pub type Patterns = Vec<(Regex, TokenKind)>;
pub type Tokens = Vec<Token>;

fn new_pattern(pattern: &str) -> Regex {
    Regex::new(&("^".to_string() + pattern)).unwrap()
}

fn get_lex_patterns() -> Patterns {
    vec![(
        new_pattern(r"-?\d+(\.\d+)?"),
        TokenKind::Number,
    ), (
        new_pattern(r#""([^"\\]|\\.)*""#),
        TokenKind::String,
    ), (
        new_pattern(r"(true|false)"),
        TokenKind::Boolean,
    ), (
        new_pattern(r"([a-zA-Z_]\w*!)"),
        TokenKind::Macro,
    ), (
        new_pattern(r"(#include)"),
        TokenKind::Include,
    ), (
        new_pattern(r"(#ifndef)"),
        TokenKind::Ifndef,
    ), (
        new_pattern(r"(#ifdef)"),
        TokenKind::Ifdef,
    ), (
        new_pattern(r"(#endif)"),
        TokenKind::Endif,
    ), (
        new_pattern(r"(#else)"),
        TokenKind::Else,
    ), (
        new_pattern(r"(#define)"),
        TokenKind::Define,
    ), (
        new_pattern(r"(#end)"),
        TokenKind::EndDefine,
    ), (
        new_pattern(r"(#undef)"),
        TokenKind::Undef,
    ), (
        new_pattern(r"(nil)"),
        TokenKind::Nil,
    ), (
        new_pattern(r"(#[a-zA-Z_]\w*#)"),
        TokenKind::Stringify,
    ), (
        new_pattern(r"##"),
        TokenKind::Paste,
    ), (
        new_pattern(r"([a-zA-Z_]\w*)"),
        TokenKind::Name,
    ), (
        new_pattern(r"([+\-*/!@#$%&|:<>=?~^.]+)"),
        TokenKind::Special,
    ), (
        new_pattern(r"[,;]"),
        TokenKind::Delimiter,
    ), (
        new_pattern(r"[()\[\]{}]"),
        TokenKind::Brace,
    )]
}

fn apply_pattern(
    pattern: Regex,
    kind: TokenKind,
    tokens: &mut Tokens,
    input: &str,
) -> Option<String> {
    let capture = pattern.captures(input)?;
    let full = &capture[0];
    tokens.push(Token {
        kind: kind.clone(),
        value: full.to_string(),
    });
    return Some(input[full.len()..].to_string());
}

fn apply_patterns(
    patterns: &Patterns,
    tokens: &mut Tokens,
    input: &str,
) -> Option<String> {
    for (pattern, kind) in patterns {
        let result = apply_pattern(
            pattern.clone(),
            kind.clone(),
            tokens,
            input,
        );
        match result {
            None => continue,
            some @ Some(_) => return some,
        }
    }
    None
}

pub fn lex(mut input: String) -> Option<Tokens> {
    input = Regex::new("--.*").unwrap().replace_all(&input, "").to_string();
    let patterns = get_lex_patterns();
    let mut tokens = vec![];
    input = input.trim_start().to_string();

    while !input.is_empty() {
        input = apply_patterns(
            &patterns,
            &mut tokens,
            &input
        )?;
        input = input.trim_start().to_string();
    }

    Some(tokens)
}
