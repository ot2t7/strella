use std::ffi::OsStr;
use std::fmt::Display;
use std::fs::read;
use std::path::PathBuf;
use std::process::exit;

use clap::Parser;
use lualexer::{FastLexer, TokenType};

#[derive(Parser)]
struct BuildRequest {
    #[clap(parse(from_os_str))]
    input_file: PathBuf
}

/// Like an `unwrap` or `expect` function, but it has a custom print message.
fn maybe_fail<T, E>(input: Result<T, E>) -> T
where E: Display
{
    return match input {
        Ok(v) => v,
        Err(e) => {eprintln!("strella: {}", e); exit(1)}
    }
}

/// Like an `unwrap` or `expect` function, but it has a custom print message
/// and also is ran on an Option.
fn must_some<T>(input: Option<T>, msg: &str) -> T
{
    return match input {
        Some(v) => v,
        None => {eprintln!("strella: {}", msg); exit(1)}
    }
}

/// Same error messages as `maybe_fail`, but will always fail.
fn fail<E>(input: E) -> !
where E: Display
{
    eprintln!("strella: {}", input); 
    exit(1);
}

/// Find all strings the script attempts to import
fn analyze_imports(source: &Vec<u8>, current_location: &PathBuf) -> Vec<String> {
    let src = String::from_utf8_lossy(source);
    let lexer = FastLexer::new();
    let mut tokens;
    match lexer.parse(&src) {
        Ok(v) => {
            tokens = v;
        }
        Err(e) => fail(format!("lexer error: {}", e.1))
    }

    // Remove all comments
    let mut to_delete = vec![];
    for (i, t) in tokens.iter().enumerate() {
        if t.get_type() == &TokenType::Comment {
            to_delete.push(i);
        }
    }
    for i in to_delete {
        tokens.remove(i);
    }

    // Analyze tokens
    let mut strings_to_require: Vec<String> = vec![];
    for (index, token) in tokens.iter().enumerate() {
        if token.get_type() == &TokenType::Identifier && token.get_content() == "require" {
            let next = must_some(tokens.get(index + 1), "ran out of tokens");
            match next.get_type() {
                &TokenType::String => { // require "something.lua"
                    let mut path = next.get_content();
                    path = &path[1..path.len() - 1]; // Remove the quotes
                    strings_to_require.push(path.to_string());
                }
                &TokenType::Symbol => {
                    if next.get_content() == "(" { // require( 
                        let mut require_string = "";
                        let mut require_string_registered = false;
                        let mut broke = false;
                        let mut parenthesis = 1;
                        let mut curr_token = index + 2;
                        while parenthesis > 0 && broke == false {
                            let seeking_token = must_some(tokens.get(curr_token), "ran out of tokens");
                            match seeking_token.get_type() {
                                &TokenType::String => { // require("something.lua")
                                    let mut path = seeking_token.get_content();
                                    if require_string_registered != true {
                                        path = &path[1..path.len() - 1]; // Remove the quotes

                                        require_string = path;
                                        require_string_registered = true;
                                    } else {
                                        broke = true;
                                    }
                                }
                                &TokenType::Symbol => { // require(("something.lua"))
                                    if seeking_token.get_content() == "(" {
                                        parenthesis += 1;
                                    } else if seeking_token.get_content() == ")" {
                                        parenthesis -= 1;
                                    } else {
                                        broke = true;
                                    }
                                }
                                _ => {
                                    broke = true;
                                }
                            }
                            
                            curr_token += 1;
                        }
                        if broke {
                            println!("{}: Possibly skipped an import because require is not properly called with one string constant.", current_location.to_string_lossy()); 
                        } else if require_string_registered == true {
                            strings_to_require.push(require_string.to_string());
                        }
                    } else {
                        println!("{}: Possibly skipped an import because require is not properly called with one string constant.", current_location.to_string_lossy());
                    }
                }
                _ => {
                    println!("{}: Possibly skipped an import because require is not properly called with one string constant.", current_location.to_string_lossy());
                }
            }
        }
    }

    // Parse imports for invalid ones
    for (i, import) in strings_to_require.iter().enumerate() {
        let path = PathBuf::from(import);
    }

    return strings_to_require;
}

fn main() {
    let args = BuildRequest::parse();
    let input_src = maybe_fail(read(&args.input_file));
    if args.input_file.extension() != Some(OsStr::new("lua")) { maybe_fail(Err("Input file isn't a .lua file"))};

    analyze_imports(&input_src, &args.input_file);
}