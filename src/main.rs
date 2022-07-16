use std::{ffi::OsStr, path::Path};
use std::fmt::Display;
use std::fs::read;
use std::path::PathBuf;
use std::process::exit;
use std::env::current_dir;

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
// TODO: If two scripts import each other, or theres any kind
// of import loop, the builder will crash.
// TODO: Absolute paths for importing does not work.
fn analyze_imports(source: &Vec<u8>, current_location: &PathBuf) -> Vec<PathBuf> {
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
    tokens = tokens
        .into_iter()
        .filter(|t| t.get_type() != &TokenType::Comment)
        .collect();

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
                            let path = current_location.file_name().unwrap_or(OsStr::new("")).to_string_lossy();
                            println!("{}: Possibly skipped an import because require is not properly called with one string constant.", path); 
                        } else if require_string_registered == true {
                            strings_to_require.push(require_string.to_string());
                        }
                    } else {
                        let path = current_location.file_name().unwrap_or(OsStr::new("")).to_string_lossy();
                        println!("{}: Possibly skipped an import because require is not properly called with one string constant.", path);
                    }
                }
                _ => {
                    let path = current_location.file_name().unwrap_or(OsStr::new("")).to_string_lossy();
                    println!("{}: Possibly skipped an import because require is not properly called with one string constant.", path);
                }
            }
        }
    }

    // Make all import paths into absolute paths
    let parent = current_location.parent().unwrap_or(Path::new(""));
    let mut paths_to_require: Vec<PathBuf> = strings_to_require
        .into_iter()
        .map(|i| parent.join(&i))
        .collect();

    // Parse imports for invalid ones
    paths_to_require.retain(|path| {
        match read(&path) {
            Ok(_) => {
                if path.extension() != Some(OsStr::new("lua")) {
                    let curr_path = current_location.file_name().unwrap_or(OsStr::new("")).to_string_lossy();
                    println!("{}: Skipping import `{}` (file isn't .lua file)", curr_path, path.to_string_lossy());
                    return false;
                }
            }
            Err(e) => {
                let curr_path = current_location.file_name().unwrap_or(OsStr::new("")).to_string_lossy();
                println!("{}: Skipping import `{}` ({})", curr_path, path.to_string_lossy(), e);
                return false
            }
        }
        return true;
    });

    // Recursively analyze imports
    for i in &paths_to_require.clone() {
        // Safety: These files are already parsed and must be
        // valid.
        let contents = read(&i).unwrap();
        let mut imports = analyze_imports(&contents, i);
        paths_to_require.append(&mut imports);
    }

    return paths_to_require;
}

fn main() {
    let args = BuildRequest::parse();
    let path = maybe_fail(current_dir()).join(args.input_file);
    let input_src = maybe_fail(read(&path));
    if path.extension() != Some(OsStr::new("lua")) { maybe_fail(Err("Input file isn't a .lua file"))};

    let imports = analyze_imports(&input_src, &path);
    println!("{:?}", imports);
}