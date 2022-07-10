mod analyze;

use analyze::get_imports;
use departure::Function;
use departure::Constant;
use departure::deserialize;
use departure::ParserError;

use std::ffi::OsStr;
use std::fmt::Display;
use std::fs::read;
use std::path::PathBuf;
use std::process::exit;

use clap::Parser;
use mlua::Lua;

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

/// Same error messages as `maybe_fail`, but will always fail.
fn fail<E>(input: E) -> !
where E: Display
{
    eprintln!("strella: {}", input); 
    exit(1);
}

/// Find all strings the script attempts to import
fn analyze_imports(source: &Vec<u8>) -> Vec<Box<OsStr>> {
    let src = String::from_utf8_lossy(&source);
    let func: Function;
    match deserialize(&src.to_string()) {
        Ok(v) => func = v,
        Err(e) => {
            match e {
                ParserError::LuaError(le) => fail(le),
                _ => fail("Internal parsing error")
            }
        }
    }

    get_imports(&func);

    todo!()
}

fn main() {
    let args = BuildRequest::parse();
    let input_src = maybe_fail(read(&args.input_file));
    if args.input_file.extension() != Some(OsStr::new("lua")) { maybe_fail(Err("Input file isn't a lua file"))};

    analyze_imports(&input_src);
}