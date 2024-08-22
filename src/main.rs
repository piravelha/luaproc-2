use regex::Regex;

#[derive(Debug, Clone)]
enum TokenKind {
    Number,
    String,
    Boolean,
    Nil,
    Name,
}

#[derive(Debug, Clone)]
struct Token {
    kind: TokenKind,
    value: String,
}

type Patterns = Vec<(Regex, TokenKind)>;
type Tokens = Vec<Token>;

fn get_lex_patterns() -> Patterns {
    vec![(
        Regex::new(r"^-?\d+(\.\d+)?").unwrap(),
        TokenKind::Number,
    ), (
        Regex::new(r#"^"([^"\\]|\\.)*""#).unwrap(),
        TokenKind::String,
    ), (
        Regex::new(r"^(true|false)").unwrap(),
        TokenKind::Boolean,
    ), (
        Regex::new(r"^(nil)").unwrap(),
        TokenKind::Nil,
    ), (
        Regex::new(r"^([a-zA-Z_]\w*)").unwrap(),
        TokenKind::Name,
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

fn main() {
    let input = "local var is 5 or nil";
    let tokens = lex(input.to_string())
        .expect("Tokenization failed");
    tokens.into_iter()
        .for_each(|token| println!("{:?}", token));
}


