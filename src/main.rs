mod instr;
mod parse;
mod analyze;

use instr::Function;
use instr::OpCode;
use instr::Constant;
use parse::deserialize;
use parse::ParserError;

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

fn debug_func(func: &Function, level: u32) {
    let indent = " ".repeat(level as usize);
    print!("{}constants: ", indent);
    for c in &func.constants {
        match c {
            Constant::String(s) => {
                print!("{}{}, ", indent, s.to_string_lossy())
            }
            _ => {
                print!("{}{:?}, ", indent, c);
            }
        }
    }
    println!();
    for i in &func.instructions {
        match i.instruction_kind {
            instr::InstructionKind::ABC => {
                println!("{}{:>10?}{:>4}{:>4}{:>4}", indent, i.op_code, i.a, i.b.unwrap(), i.c.unwrap());
            },
            instr::InstructionKind::ABx => {
                println!("{}{:10?}{:4}{:4}", indent, i.op_code, i.a, i.bx.unwrap());
            },
            instr::InstructionKind::AsBx => {
                println!("{}{:10?}{:4}{:4}", indent, i.op_code, i.a, i.sbx.unwrap());
            },
        }
    }
    for f in &func.function_protos {
        debug_func(f, level + 1);
    }
}

/// Find all strings the script attempts to import
fn analyze_imports(source: &Vec<u8>) -> Vec<Box<OsStr>> {
    let src = String::from_utf8_lossy(&source);
    let state = Lua::new(); // The state will hold all lua variables we can't represent
    let func: Function;
    match deserialize(&src.to_string(), &state) {
        Ok(v) => func = v,
        Err(e) => {
            match e {
                ParserError::LuaError(le) => fail(le),
                _ => fail("Internal parsing error")
            }
        }
    }

    debug_func(&func, 0);

    fn analyze_func(f: &Function) {
        let mut require_registers: Vec<i32> = vec![];
        for i in &f.instructions {
            match i.op_code {
                OpCode::GetGlobal => {
                    let constant = &f.constants[i.bx.unwrap() as usize];
                    match constant {
                        Constant::String(s) => {
                            if s.to_string_lossy() == "require" {
                                require_registers.push(i.a);
                            }
                        },
                        _ => {}
                    }
                },
                OpCode::Call => {
                    println!("call.. {}", i.a);
                    if require_registers.contains(&i.a) {
                        println!("Require called...");
                    }
                },
                OpCode::Move => {
                    if require_registers.contains(&i.b.unwrap()) {
                        require_registers.retain(|v| *v != i.b.unwrap());
                        require_registers.push(i.a);
                    }
                }
                _ => {}
            }
        }

        for proto in &f.function_protos {
            analyze_func(proto);
        }
    }

    analyze_func(&func);

    todo!()
}

fn main() {
    let args = BuildRequest::parse();
    let input_src = maybe_fail(read(&args.input_file));
    if args.input_file.extension() != Some(OsStr::new("lua")) { maybe_fail(Err("Input file isn't a lua file"))};

    analyze_imports(&input_src);
}