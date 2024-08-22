use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::{iter::Peekable, vec::IntoIter};
use std::process::Command;
mod lexer;

#[derive(Debug, Clone)]
struct ValueMacro {
    name: String,
    tokens: lexer::Tokens,
}

#[derive(Debug, Clone)]
struct FuncMacro {
    name: String,
    params: Vec<String>,
    tokens: lexer::Tokens,
}

fn replace_tokens(
    tokens: lexer::Tokens,
    old: lexer::Tokens,
    new: lexer::Tokens,
) -> lexer::Tokens {
    let mut new_tokens = Vec::new();
    let mut iter = tokens.into_iter().peekable();

    while let Some(_) = iter.peek() {
        if iter.clone()
                .take(old.len())
                .zip(&old)
                .all(|(t, o)| t.kind == o.kind && t.value == o.value) {
            for _ in 0..old.len() {
                iter.next();
            }
            new_tokens.extend(new.clone());
        } else {
            new_tokens.push(iter.next().unwrap());
        }
    }

    new_tokens
}

fn process_value_macro(
    iter: &mut Peekable<IntoIter<lexer::Token>>,
    value_macros: &mut Vec<ValueMacro>,
    name: lexer::Token,
) -> Option<()> {
    let value = iter.clone().take_while(|tok|
        tok.kind != lexer::TokenKind::EndDefine
    );
    for _ in 0..value.clone().collect::<Vec<_>>().len() {
        iter.next();
    }
    iter.next().filter(|end| {
        end.kind == lexer::TokenKind::EndDefine
    })?;
    value_macros.push(ValueMacro {
        name: name.value,
        tokens: value.collect(),
    });
    Some(())
}

fn parse_func_params_rest(
    iter: &mut Peekable<IntoIter<lexer::Token>>,
    args: &mut Vec<String>,
) -> Option<()> {
    while let Some(next_token) = iter.peek() {
        if next_token.value.as_str() == ")" {
            iter.next();
            break;
        }
        if next_token.kind != lexer::TokenKind::Delimiter {
            return None;
        }
        iter.next()?;
        if let Some(name_token) = iter.next() {
            if name_token.kind != lexer::TokenKind::Name {
                return None;
            }
            args.push(name_token.value);
        }
    }
    Some(())
}

fn parse_func_params(
    iter: &mut Peekable<IntoIter<lexer::Token>>,
) -> Option<Vec<String>> {
    let mut args = vec![];
    let name = iter.next().filter(|name|
        name.kind == lexer::TokenKind::Name
    )?;
    args.push(name.clone().value);
    parse_func_params_rest(iter, &mut args)?;
    Some(args)
}

fn process_func_macro(
    iter: &mut Peekable<IntoIter<lexer::Token>>,
    func_macros: &mut Vec<FuncMacro>,
    name: lexer::Token,
) -> Option<()> {
    let params = parse_func_params(iter)?;
    iter.next().filter(|eq|
        eq.value.as_str() == "="
    )?;
    let value = iter.clone().take_while(|tok|
        tok.kind != lexer::TokenKind::EndDefine
    );
    for _ in 0..value.clone().collect::<Vec<_>>().len() {
        iter.next();
    }
    iter.next().filter(|end| {
        end.kind == lexer::TokenKind::EndDefine
    })?;
    func_macros.push(FuncMacro {
        name: name.value,
        params: params,
        tokens: value.collect(),
    });
    Some(())
}

fn parse_func_arg(
    iter: &mut Peekable<IntoIter<lexer::Token>>,
) -> Option<lexer::Tokens> {
    let mut arg_tokens = vec![];
    let mut nesting_level = 0;
    let nesters = vec!["(", "[", "{"];
    let denesters = vec![")", "]", "}"];
    while let Some(token) = iter.clone().peek() {
        if nesting_level == 0 && token.kind == lexer::TokenKind::Delimiter {
            return Some(arg_tokens);
        }
        iter.next()?;
        if nesters.contains(&token.value.as_str()) {
            nesting_level += 1;
        }
        if denesters.contains(&token.value.as_str()) {
            if nesting_level == 0 {
                return Some(arg_tokens);
            }
        }
        arg_tokens.push(token.clone());
    }
    None
}

fn parse_func_args(
    iter: &mut Peekable<IntoIter<lexer::Token>>,
) -> Option<Vec<Vec<lexer::Token>>> {
    iter.next().filter(|lparen|
        lparen.value.as_str() == "("
    )?;
    let mut args = vec![];
    while let Some(token) = iter.clone().peek() {
        if token.value.as_str() == ")" {
            break
        }
        let arg = parse_func_arg(iter)?;
        args.push(arg);
    }
    Some(args)
}

fn process_tokens(tokens: lexer::Tokens) -> Option<lexer::Tokens> {
    let mut new_tokens = vec![];
    let mut value_macros = vec![];
    let mut func_macros = vec![];
    let mut iter = tokens.into_iter().peekable();

    while let Some(token) = iter.next() {
        if token.kind == lexer::TokenKind::Define {
            let name = iter.next().filter(|name|
                name.kind == lexer::TokenKind::Macro
            )?;
            let eq_or_lparen = iter.next()?;
            match eq_or_lparen.value.as_str() {
                "=" => process_value_macro(&mut iter, &mut value_macros, name)?,
                "(" => process_func_macro(&mut iter, &mut func_macros, name)?,
                _ => return None,
            }
        } else if token.kind == lexer::TokenKind::Macro {
            let value_macro_opt = value_macros.clone().into_iter().find(|val_macro|
                val_macro.name == token.value
            );
            if let Some(value_macro) = value_macro_opt {
                new_tokens.extend(value_macro.tokens);
                continue;
            }
            let func_macro_opt = func_macros.clone().into_iter().find(|func_macro|
                func_macro.name == token.value
            );
            if let Some(func_macro) = func_macro_opt {
                let args = parse_func_args(&mut iter)?;
                let params = func_macro.params.into_iter()
                    .map(|s| lexer::Token {
                        kind: lexer::TokenKind::Name,
                        value: s,
                    }).collect::<Vec<_>>();
                let mut body = func_macro.tokens;
                for (arg, param) in args.into_iter().zip(&params) {
                    body = replace_tokens(body, vec![param.clone()], arg);
                }
                new_tokens.extend(body);
                continue;
            }
            return None
        } else {
            new_tokens.push(token.clone());
        }
    }

    Some(new_tokens)
}

fn render_tokens(tokens: lexer::Tokens) -> String {
    tokens.into_iter()
        .fold("".to_string(), |acc, token|
            acc + " " + &token.value
        )
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let input_path = args.get(1).expect("Usage: luaproc <filename>");
    let mut input_file = File::open(input_path).expect("Could not open file");
    let mut input = String::new();
    match input_file.read_to_string(&mut input) {
        Err(e) => eprintln!("Error: {}", e),
        Ok(_) => {},
    };
    let tokens = lexer::lex(input).expect("Tokenization failed");
    let processed = process_tokens(tokens).expect("Processing failed");
    let string = render_tokens(processed);
    let mut output_file = File::create("out.lua").expect("Could not create file");
    match output_file.write_all(string.as_bytes()) {
        Err(e) => eprintln!("Error: {}", e),
        Ok(_) => {},
    };
    let _ = Command::new("stylua")
        .arg("out.lua")
        .output();
}

