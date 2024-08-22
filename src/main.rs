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
    old: lexer::Token,
    new: lexer::Tokens,
) -> lexer::Tokens {
    let mut new_tokens = Vec::new();
    let mut iter = tokens.into_iter().peekable();

    while let Some(token) = iter.peek() {
        if token.kind == old.kind
        && token.value == old.value {
            iter.next();
            new_tokens.extend(new.clone());
        } else if token.kind == lexer::TokenKind::Stringify
        && &token.value[1..token.value.len()-1] == old.value.as_str() {
            iter.next();
            new_tokens.push(lexer::Token {
                kind: lexer::TokenKind::String,
                value: format!("{:?}", render_tokens(new.clone())),
            });
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
        if nesting_level <= 0 && token.kind == lexer::TokenKind::Delimiter {
            return Some(arg_tokens);
        }
        if nesters.contains(&token.value.as_str()) {
            nesting_level += 1;
        }
        if denesters.contains(&token.value.as_str()) {
            nesting_level -= 1;
            if nesting_level <= 0 {
                arg_tokens.push(token.clone());
                iter.next();
                return Some(arg_tokens);
            }
        }
        iter.next()?;
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
            iter.next();
            break
        }
        if token.kind == lexer::TokenKind::Delimiter {
            iter.next()?;
        }
        let arg = parse_func_arg(iter)?;
        args.push(arg);
    }
    Some(args)
}

fn process_tokens(
    tokens: lexer::Tokens,
    value_macros: &mut Vec<ValueMacro>,
    func_macros: &mut Vec<FuncMacro>,
) -> Result<lexer::Tokens, String> {
    let mut new_tokens = vec![];
    let mut iter = tokens.into_iter().peekable();

    while let Some(token) = iter.next() {
        if token.kind == lexer::TokenKind::Ifdef
        || token.kind == lexer::TokenKind::Ifndef {
            let name = iter.next().filter(|name|
                name.kind == lexer::TokenKind::Macro
            ).ok_or("Expected macro name in Ifdef".to_string())?;
            let in_values = value_macros.into_iter().find(|val_macro|
                val_macro.name == name.value
            ).is_some();
            let in_funcs = func_macros.into_iter().find(|func_macro|
                func_macro.name == name.value
            ).is_some();
            let mut body = vec![];
            let mut has_else = false;
            while let Some(next_token) = iter.next() {
                if next_token.kind == lexer::TokenKind::Else {
                    has_else = true;
                    break;
                } else if next_token.kind == lexer::TokenKind::Endif {
                    break;
                }
                body.push(next_token);
            }
            let mut else_body = vec![];
            if has_else {
                while let Some(next_token) = iter.next() {
                    if next_token.kind == lexer::TokenKind::Endif {
                        break;
                    }
                    else_body.push(next_token);
                }
            }
            let mut exists = in_values || in_funcs;
            if token.kind == lexer::TokenKind::Ifndef {
                exists = !exists
            }
            if exists {
                let result = process_tokens(body, value_macros, func_macros)?;
                new_tokens.extend(result);
            } else if has_else {
                let result = process_tokens(else_body, value_macros, func_macros)?;
                new_tokens.extend(result);
            }
        } else if token.kind == lexer::TokenKind::Endif {
            continue;
        } else if token.kind == lexer::TokenKind::Define {
            let name = iter.next().filter(|name|
                name.kind == lexer::TokenKind::Macro
            ).ok_or("Expected macro name".to_string())?;
            let eq_or_lparen = iter.next().ok_or("Expected '=' or '(' on macro declaration".to_string())?;
            match eq_or_lparen.value.as_str() {
                "=" => process_value_macro(&mut iter, value_macros, name).ok_or("Failed parsing value macro".to_string())?,
                "(" => process_func_macro(&mut iter, func_macros, name).ok_or("Failed to parse func macro".to_string())?,
                "#end" => value_macros.push(ValueMacro {
                    name: name.value,
                    tokens: vec![],
                }),
                _ => return Err("Expected '=', '(', or '#end'".to_string()),
            }
        } else if token.kind == lexer::TokenKind::Macro {
            let value_macro_opt = value_macros.clone().into_iter().find(|val_macro|
                val_macro.name == token.value
            );
            if let Some(value_macro) = value_macro_opt {
                let result = process_tokens(value_macro.tokens, value_macros, func_macros)?;
                new_tokens.extend(result);
                continue;
            }
            let func_macro_opt = func_macros.clone().into_iter().find(|func_macro|
                func_macro.name == token.value
            );
            if let Some(func_macro) = func_macro_opt {
                let args = parse_func_args(&mut iter).ok_or("Failed parsing arguments on macro invocation")?;
                let params = func_macro.params.into_iter()
                    .map(|s| lexer::Token {
                        kind: lexer::TokenKind::Name,
                        value: s,
                    }).collect::<Vec<_>>();
                let mut body = func_macro.tokens;
                for (arg, param) in args.into_iter().zip(&params) {
                    body = replace_tokens(body, param.clone(), arg);
                }
                let result = process_tokens(body, value_macros, func_macros)?;
                new_tokens.extend(result);
                continue;
            }
            return Err("Attempting to call non-existent macro".to_string())
        } else if token.kind == lexer::TokenKind::Undef {
            let name = iter.next().filter(|tok|
                tok.kind == lexer::TokenKind::Macro
            ).ok_or("Undef must be followed by a macro name")?;
            value_macros.retain(|val_macro|
                val_macro.name != name.value
            );
            func_macros.retain(|func_macro|
                func_macro.name != name.value
            );
        } else if token.kind == lexer::TokenKind::Include {
            let path = iter.next().filter(|tok|
                tok.kind == lexer::TokenKind::String
            ).ok_or("Include must be followed by a string literal")?;
            let result = process_file((&path.value[1..path.value.len()-1]).to_string())?;
            let processed = process_tokens(result, value_macros, func_macros)?;
            new_tokens.extend(processed);
        } else {
            new_tokens.push(token.clone());
        }
    }

    Ok(new_tokens)
}

fn apply_pastes_rest(
    iter: &mut Peekable<IntoIter<lexer::Token>>,
    parts: &mut Vec<String>,
) {
    while let Some(next_token) = iter.peek() {
        if next_token.kind == lexer::TokenKind::Paste {
            iter.next();
            if let Some(name_token) = iter.next() {
                if name_token.kind == lexer::TokenKind::Name {
                    parts.push(name_token.value);
                    continue 
                }
            }
        }
        break;
    }
}

fn apply_pastes(tokens: lexer::Tokens) -> lexer::Tokens {
    let mut iter = tokens.into_iter().peekable();
    let mut new_tokens = vec![];
    while let Some(token) = iter.next() {
        if token.kind == lexer::TokenKind::Name {
            let mut parts = vec![token.value.clone()];
            apply_pastes_rest(&mut iter, &mut parts);
            let string = parts.join("");
            new_tokens.push(lexer::Token {
                kind: lexer::TokenKind::Name,
                value: string,
            });
        } else {
            new_tokens.push(token.clone());
        }
    }
    new_tokens
}

fn render_tokens(tokens: lexer::Tokens) -> String {
    tokens.into_iter()
        .map(|token| token.value)
        .collect::<Vec<_>>()
        .join(" ")
}

fn add_header_guard(path: String, tokens: Vec<lexer::Token>) -> Vec<lexer::Token> {
    let mut new_tokens = vec![lexer::Token {
        kind: lexer::TokenKind::Ifndef,
        value: "#ifndef".to_string(),
    }, lexer::Token {
        kind: lexer::TokenKind::Macro,
        value: path.clone(),
    }, lexer::Token {
        kind: lexer::TokenKind::Define,
        value: "#define".to_string(),
    }, lexer::Token {
        kind: lexer::TokenKind::Macro,
        value: path,
    }, lexer::Token {
        kind: lexer::TokenKind::EndDefine,
        value: "#end".to_string(),
    }];
    new_tokens.extend(tokens);
    new_tokens.push(lexer::Token {
        kind: lexer::TokenKind::Endif,
        value: "#endif".to_string(),
    });
    new_tokens
}

fn process_file(path: String) -> Result<lexer::Tokens, String> {
    let mut input_file = File::open(path.clone()).map_err(|e| format!("{}", e))?;
    let mut input = String::new();
    input_file.read_to_string(&mut input).map_err(|e| format!("{}", e))?;
    let tokens = lexer::lex(input.clone()).ok_or("Tokenization failed")?;
    let tokens = add_header_guard(path, tokens);
    Ok(tokens)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let input_path = args.get(1).expect("Usage: luaproc <filename>");
    let processed = match process_file(input_path.to_string()) {
        Err(e) => {
            eprintln!("{}", e);
            return;
        },
        Ok(p) => p,
    };
    let processed = process_tokens(processed, &mut vec![], &mut vec![]).expect("Processing failed");
    let processed = apply_pastes(processed);
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

