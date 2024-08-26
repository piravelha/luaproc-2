use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::process::exit;
use std::process::Command;
use std::process::Stdio;
use std::{iter::Peekable, vec::IntoIter};
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
  vararg: bool,
  tokens: lexer::Tokens,
}

fn replace_tokens(
  tokens: lexer::Tokens,
  old: lexer::Token,
  new: lexer::Tokens,
) -> lexer::Tokens {
  let mut new_tokens = Vec::new();
  let mut iter = tokens.into_iter().peekable();

  while let Some(token) = iter.clone().peek() {
    if token.kind == old.kind && token.value == old.value {
      iter.next();
      new_tokens.extend(new.clone().into_iter()
        .map(|tok| lexer::Token {
          kind: tok.kind,
          value: tok.value,
          location: token.location.clone(),
        })
        .collect::<Vec<_>>()
        .clone());
    } else if token.kind == lexer::TokenKind::Stringify
      && &token.value[1..token.value.len() - 1]
        == old.value.as_str()
    {
      iter.next();
      new_tokens.push(lexer::Token {
        kind: lexer::TokenKind::String,
        value: format!("{:?}", render_tokens(new.clone())),
        location: token.clone().location,
      });
    } else {
      new_tokens.push(iter.next().unwrap());
    }
  }

  new_tokens
}

fn get_macro_body(
  iter: &mut Peekable<IntoIter<lexer::Token>>,
) -> lexer::Tokens {
  let mut tokens = vec![];
  while let Some(token) = iter.next() {
    if token.kind == lexer::TokenKind::Define {
      let new_tokens = get_macro_body(iter);
      tokens.push(token);
      tokens.extend(new_tokens);
    } else if token.kind == lexer::TokenKind::EndDefine {
      return tokens
    } else {
      tokens.push(token);
    }
  }
  tokens
}

fn process_value_macro(
  iter: &mut Peekable<IntoIter<lexer::Token>>,
  value_macros: &mut Vec<ValueMacro>,
  name: lexer::Token,
) -> Option<()> {
  let value = get_macro_body(iter);
  value_macros.push(ValueMacro {
    name: name.value,
    tokens: value,
  });
  Some(())
}

fn parse_func_params_rest(
  iter: &mut Peekable<IntoIter<lexer::Token>>,
  args: &mut Vec<String>,
) -> Option<bool> {
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
      if name_token.kind == lexer::TokenKind::Vararg {
        iter.next()?;
        return Some(true);
      } else if name_token.kind != lexer::TokenKind::Name {
        return None;
      }
      args.push(name_token.value);
    }
  }
  Some(false)
}

fn parse_func_params(
  iter: &mut Peekable<IntoIter<lexer::Token>>,
) -> Option<(Vec<String>, bool)> {
  let mut args = vec![];
  let vararg = iter
    .peek()
    .filter(|var| var.kind == lexer::TokenKind::Vararg);
  match vararg {
    None => {
      let name = iter
        .next()
        .filter(|name| name.kind == lexer::TokenKind::Name)?;
      args.push(name.clone().value);
      let vararg = parse_func_params_rest(iter, &mut args)?;
      Some((args, vararg))
    }
    Some(_) => {
      iter.next();
      iter.next().filter(|paren|
        paren.value.as_str() == ")"
      )?;
      Some((args, true))
    },
  }
}

fn process_func_macro(
  iter: &mut Peekable<IntoIter<lexer::Token>>,
  func_macros: &mut Vec<FuncMacro>,
  name: lexer::Token,
) -> Option<()> {
  let (params, vararg) = parse_func_params(iter)?;
  let eq_or_end = iter.next()?;
  match eq_or_end.value.as_str() {
    "=" => {}
    "#end" => {
      func_macros.push(FuncMacro {
        name: name.value,
        params: params,
        vararg: vararg,
        tokens: vec![],
      });
      return Some(());
    }
    _ => return None,
  }
  let value = get_macro_body(iter);
  func_macros.push(FuncMacro {
    name: name.value,
    params: params,
    vararg: vararg,
    tokens: value,
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
    if nesting_level <= 0
      && token.kind == lexer::TokenKind::Delimiter
    {
      return Some(arg_tokens);
    }
    if nesters.contains(&token.value.as_str()) {
      nesting_level += 1;
    }
    if denesters.contains(&token.value.as_str()) {
      nesting_level -= 1;
      if nesting_level < 0 {
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
    || lparen.value.as_str() == "["
    || lparen.value.as_str() == "{"
  )?;
  let mut args = vec![];
  while let Some(token) = iter.clone().peek() {
    if token.value.as_str() == ")"
      || token.value.as_str() == "]"
      || token.value.as_str() == "}"
    {
      iter.next();
      break;
    }
    if token.kind == lexer::TokenKind::Delimiter {
      iter.next()?;
    }
    let arg = parse_func_arg(iter)?;
    args.push(arg);
  }
  Some(args)
}

fn skip_nested_ifdefs(
  iter: &mut Peekable<IntoIter<lexer::Token>>,
  body: &mut Vec<lexer::Token>,
) {
  let mut inner_body = vec![];
  while let Some(inner_token) = iter.next() {
    inner_body.push(inner_token.clone());
    if inner_token.kind == lexer::TokenKind::Ifdef {
      skip_nested_ifdefs(iter, &mut inner_body);
    } else if inner_token.kind == lexer::TokenKind::Endif {
      break;
    }
  }
  body.extend(inner_body);
}

fn apply_bang_pastes(tokens: lexer::Tokens) -> lexer::Tokens {
  let mut iter = tokens.into_iter().peekable();
  let mut new_tokens = vec![];
  while let Some(token) = iter.next() {
    if token.kind == lexer::TokenKind::Name {
      if let Some(next_token) = iter.peek() {
        if next_token.kind == lexer::TokenKind::Bang {
          iter.next();
          new_tokens.push(lexer::Token {
            kind: lexer::TokenKind::Macro,
            value: (token.value + "!").to_string(),
            location: token.location,
          });
        } else {
          new_tokens.push(token);
        }
      } else {
        new_tokens.push(token);
      }
    } else {
      new_tokens.push(token);
    }
  }
  new_tokens
}

fn join_by_commas(tokens: Vec<lexer::Tokens>) -> lexer::Tokens {
  let mut new = vec![];
  for (i, token_list) in tokens
    .into_iter()
    .enumerate()
  {
    if i > 0 {
      new.push(lexer::Token {
        kind: lexer::TokenKind::Delimiter,
        value: ",".to_string(),
        location: lexer::Location {
          file: "".to_string(),
          line: 0,
          column: 0,
        },
      });
    }
    new.extend(token_list);
  }
  new
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
      || token.kind == lexer::TokenKind::Ifndef
    {
      let name = iter
        .next()
        .filter(|name| name.kind == lexer::TokenKind::Macro)
        .ok_or(format!("{:?}: Expected macro name in `#ifdef`", token.location))?;
      let in_values = value_macros
        .into_iter()
        .find(|val_macro| val_macro.name == name.value)
        .is_some();
      let in_funcs = func_macros
        .into_iter()
        .find(|func_macro| func_macro.name == name.value)
        .is_some();
      let mut body = vec![];
      let mut has_else = false;
      while let Some(next_token) = iter.next() {
        if next_token.kind == lexer::TokenKind::Ifdef
          || next_token.kind == lexer::TokenKind::Ifndef
        {
          body.push(next_token);
          skip_nested_ifdefs(&mut iter, &mut body);
          continue;
        } else if next_token.kind == lexer::TokenKind::Else {
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
          if next_token.kind == lexer::TokenKind::Ifdef
            || next_token.kind == lexer::TokenKind::Ifndef
          {
            else_body.push(next_token);
            skip_nested_ifdefs(&mut iter, &mut else_body);
            continue;
          } else if next_token.kind == lexer::TokenKind::Endif {
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
        let result =
          process_tokens(body, value_macros, func_macros)?;
        new_tokens.extend(result);
      } else if has_else {
        let result =
          process_tokens(else_body, value_macros, func_macros)?;
        new_tokens.extend(result);
      }
    } else if token.kind == lexer::TokenKind::Endif {
      continue;
    } else if token.kind == lexer::TokenKind::Define {
      let name = iter
        .next()
        .filter(|name| name.kind == lexer::TokenKind::Macro)
        .ok_or(format!("{:?}: Expected macro name", token.location))?;
      let eq_or_lparen = iter.next().ok_or(format!(
        "{:?}: Expected '=' or '(' on macro declaration",
        name.location,
      ))?;
      match eq_or_lparen.value.as_str() {
        "=" => {
          process_value_macro(&mut iter, value_macros, name.clone())
            .ok_or(format!("{:?}: Failed parsing value macro", name.clone().location))?
        }
        "(" | "[" | "{" => process_func_macro(&mut iter, func_macros, name.clone())
          .ok_or(format!("{:?}: Failed to parse func macro", name.clone().location))?,
        "#end" => value_macros.push(ValueMacro {
          name: name.value,
          tokens: vec![],
        }),
        _ => {
          return Err(format!("{:?}: Expected '=', '(', or '#end'", name.clone().location))
        }
      }
    } else if token.kind == lexer::TokenKind::Macro {
      let value_macro_opt = value_macros
        .clone()
        .into_iter()
        .find(|val_macro| val_macro.name == token.value);
      if let Some(value_macro) = value_macro_opt {
        let tokens = value_macro.tokens;
        let tokens = apply_bang_pastes(tokens);
        let result = process_tokens(
          tokens,
          value_macros,
          func_macros,
        )?;
        new_tokens.extend(result);
        continue;
      }
      let func_macro_opt = func_macros
        .clone()
        .into_iter()
        .find(|func_macro| func_macro.name == token.value);
      if let Some(func_macro) = func_macro_opt {
        let args = parse_func_args(&mut iter).ok_or(format!(
          "{:?}: Failed parsing arguments on macro invocation",
          token.location,
        ))?;
        let params = func_macro
          .params
          .into_iter()
          .map(|s| lexer::Token {
            kind: lexer::TokenKind::Name,
            value: s,
            location: token.clone().location,
          })
          .collect::<Vec<_>>();
        let mut body = func_macro.tokens;
        let mut rest = args.clone();
        for (arg, param) in args.into_iter().zip(&params) {
          body = replace_tokens(body, param.clone(), arg);
          rest.remove(0);
        }
        let mut body = apply_bang_pastes(body);
        let stringified = rest
          .clone()
          .into_iter()
          .flat_map(|arg| vec![
            lexer::Token {
              kind: lexer::TokenKind::String,
              value: format!(
                "{:?}",
                render_tokens(arg),
              ),
              location: token.clone().location,
            },
            lexer::Token {
              kind: lexer::TokenKind::Delimiter,
              value: ",".to_string(),
              location: token.clone().location
            }
          ]).collect::<Vec<_>>();
        let rest = join_by_commas(rest);
        if func_macro.vararg {
          body = replace_tokens(
            body,
            lexer::Token {
              kind: lexer::TokenKind::Vararg,
              value: "#...".to_string(),
              location: token.clone().location,
            },
            rest,
          );
          body = replace_tokens(
            body,
            lexer::Token {
              kind: lexer::TokenKind::StringifyVararg,
              value: "#...#".to_string(),
              location: token.clone().location,
            },
            stringified,
          );
        }
        let result =
          process_tokens(body, value_macros, func_macros)?;
        new_tokens.extend(result);
        continue;
      }
      return Err(
        format!("{:?}: Attempting to call non-existent macro: `{}`", token.location, token.value),
      );
    } else if token.kind == lexer::TokenKind::Undef {
      let name = iter
        .next()
        .filter(|tok| tok.kind == lexer::TokenKind::Macro)
        .ok_or(format!("{:?}: `#undef` must be followed by a macro name", token.location))?;
      value_macros
        .retain(|val_macro| val_macro.name != name.value);
      func_macros
        .retain(|func_macro| func_macro.name != name.value);
    } else if token.kind == lexer::TokenKind::Include {
      let path = iter
        .next()
        .filter(|tok| tok.kind == lexer::TokenKind::String)
        .ok_or(format!("{:?}: Include must be followed by a string literal", token.location))?;
      let result = process_file(
        (&path.value[1..path.value.len() - 1]).to_string(),
      )?;
      let processed =
        process_tokens(result, value_macros, func_macros)?;
      new_tokens.extend(processed);
    } else if token.kind == lexer::TokenKind::Line {
      new_tokens.push(lexer::Token {
        kind: lexer::TokenKind::Number,
        value: format!("{}", token.location.line),
        location: token.location,
      })
    } else {
      new_tokens.push(token.clone());
    }
  }

  Ok(new_tokens)
}

fn concat_string_lits_rest(
  iter: &mut Peekable<IntoIter<lexer::Token>>,
  parts: &mut Vec<String>,
) {
  while let Some(next_token) = iter.clone().peek() {
    if next_token.kind == lexer::TokenKind::String {
      iter.next();
      parts.push(
        (&next_token.value[1..next_token.value.len() - 1])
          .to_string(),
      );
      continue;
    }
    break;
  }
}

fn concat_string_lits(tokens: lexer::Tokens) -> lexer::Tokens {
  let mut iter = tokens.into_iter().peekable();
  let mut new_tokens = vec![];
  while let Some(token) = iter.next() {
    if token.kind == lexer::TokenKind::String {
      let mut parts = vec![(&token.value
        [1..token.value.len() - 1])
        .to_string()];
      concat_string_lits_rest(&mut iter, &mut parts);
      let string = parts.join("");
      new_tokens.push(lexer::Token {
        kind: lexer::TokenKind::String,
        value: ("\"".to_owned() + &string + "\"").to_string(),
        location: token.clone().location,
      })
    } else {
      new_tokens.push(token);
    }
  }
  new_tokens
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
          continue;
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
      let mut parts = vec![token.clone().value];
      apply_pastes_rest(&mut iter, &mut parts);
      let string = parts.join("");
      new_tokens.push(lexer::Token {
        kind: lexer::TokenKind::Name,
        value: string,
        location: token.clone().location,
      });
    } else {
      new_tokens.push(token.clone());
    }
  }
  new_tokens
}

fn render_tokens(tokens: lexer::Tokens) -> String {
  tokens
    .into_iter()
    .map(|token| token.value)
    .collect::<Vec<_>>()
    .join(" ")
}

fn add_header_guard(
  path: String,
  tokens: Vec<lexer::Token>,
) -> Vec<lexer::Token> {
  let location = lexer::Location {
    file: path.clone(),
    line: 0,
    column: 0,
  };
  let mut new_tokens = vec![
    lexer::Token {
      kind: lexer::TokenKind::Ifndef,
      value: "#ifndef".to_string(),
      location: location.clone(),
    },
    lexer::Token {
      kind: lexer::TokenKind::Macro,
      value: path.clone(),
      location: location.clone(),
    },
    lexer::Token {
      kind: lexer::TokenKind::Define,
      value: "#define".to_string(),
      location: location.clone(),
    },
    lexer::Token {
      kind: lexer::TokenKind::Macro,
      value: path,
      location: location.clone(),
    },
    lexer::Token {
      kind: lexer::TokenKind::EndDefine,
      value: "#end".to_string(),
      location: location.clone(),
    },
  ];
  new_tokens.extend(tokens);
  new_tokens.push(lexer::Token {
    kind: lexer::TokenKind::Endif,
    value: "#endif".to_string(),
    location,
  });
  new_tokens
}

fn add_flags(
  flags: Vec<String>,
  tokens: lexer::Tokens,
) -> lexer::Tokens {
  let location = lexer::Location {
    file: "".to_string(),
    line: 0,
    column: 0,
  };
  let flags = flags.into_iter().flat_map(|flag| {
    vec![
      lexer::Token {
        kind: lexer::TokenKind::Define,
        value: "#define".to_string(),
        location: location.clone()
      },
      lexer::Token {
        kind: lexer::TokenKind::Macro,
        value: (flag + "!").to_string(),
        location: location.clone()
      },
      lexer::Token {
        kind: lexer::TokenKind::EndDefine,
        value: "#end".to_string(),
        location: location.clone()
      },
    ]
  });
  flags.into_iter().chain(tokens).collect()
}

fn process_file(path: String) -> Result<lexer::Tokens, String> {
  let mut input_file =
    File::open(path.clone()).map_err(|e| format!("{}", e))?;
  let mut input = String::new();
  input_file
    .read_to_string(&mut input)
    .map_err(|e| format!("{}", e))?;
  let tokens =
    lexer::lex(&path, input.clone()).ok_or("Tokenization failed")?;
  let tokens = add_header_guard(path, tokens);
  Ok(tokens)
}

fn strip_trailing_commas(
  tokens: lexer::Tokens,
) -> lexer::Tokens {
  let mut new_tokens = vec![];
  let mut iter = tokens.into_iter().peekable();
  while let Some(token) = iter.next() {
    if token.value.as_str() == "," {
      if let Some(next_token) = iter.clone().peek() {
        if next_token.value.as_str() == ")"
          || next_token.value.as_str() == "]"
          || next_token.value.as_str() == "}"
        {
          iter.next();
          new_tokens.push(next_token.clone());
        } else {
          new_tokens.push(token);
        }
      } else {
        new_tokens.push(token);
      }
    } else {
      new_tokens.push(token);
    }
  }
  new_tokens
}

enum CliMode {
  Com,
  Run,
}

struct CliOptions {
  input_path: String,
  output_path: String,
  flags: Vec<String>,
  mode: CliMode,
}

fn print_usage() {
  println!(
    "Usage: luaproc <mode> <file> <options> [--flags=*,]"
  );
  println!("    <mode>      run. Runs the file");
  println!("                com. Compiles the file");
  println!("");
  println!("    <file>      Path to the file");
  println!("");
  println!("    --flags     Comma separated list of flags");
  println!(
    "                that are treated as empty definitions"
  );
  exit(1);
}

fn process_cli_args(args: &mut Vec<String>) -> CliOptions {
  if args.len() <= 0 {
    println!("Error: expected mode");
    print_usage();
  }
  let mut input_path = "".to_string();
  let mut output_path = "out.lua".to_string();
  let mut flags = vec![];
  let mode = match args.remove(0).as_str() {
    "com" => CliMode::Com,
    "run" => CliMode::Run,
    mode => {
      println!("Error: Invalid mode: {}", mode);
      print_usage();
      exit(1);
    }
  };
  while args.len() > 0 {
    if args[0].starts_with("--flags=") {
      flags = (&args[0]["--flags=".len()..])
        .split(',')
        .map(|flag| flag.to_string())
        .collect();
      args.remove(0);
    } else if args[0].as_str() == "-o" {
      args.remove(0);
      output_path = args.remove(0);
    } else {
      input_path = (&args[0].clone()).to_string();
      args.remove(0);
    }
  }
  if input_path.len() == 0 {
    println!("Error: Expected input file path");
    print_usage();
  }
  CliOptions {
    input_path,
    output_path,
    flags,
    mode,
  }
}

fn main() {
  let mut args: Vec<String> = env::args().collect();
  args.remove(0);
  let opts = process_cli_args(&mut args);
  let input_path = opts.input_path;
  let output_path = opts.output_path;
  let flags = opts.flags;
  let processed = match process_file(input_path.to_string()) {
    Err(e) => {
      eprintln!("{}", e);
      return;
    }
    Ok(p) => p,
  };
  let processed = add_flags(flags, processed);
  let processed =
    process_tokens(processed, &mut vec![], &mut vec![])
      .expect("Processing failed");
  let processed = apply_pastes(processed);
  let processed = concat_string_lits(processed);
  let processed = strip_trailing_commas(processed);
  let string = render_tokens(processed);
  let mut output_file = File::create(output_path.clone())
    .expect("Could not create file");
  match output_file.write_all(string.as_bytes()) {
    Err(e) => eprintln!("Error: {}", e),
    Ok(_) => {}
  };
  let _ =
    Command::new("stylua").arg(output_path.clone()).output();
  match opts.mode {
    CliMode::Com => {}
    CliMode::Run => {
      let _ = Command::new("luajit")
        .arg(output_path.clone())
        .arg("&&")
        .arg("rm")
        .arg(output_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output();
    }
  }
}
