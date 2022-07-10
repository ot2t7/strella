use std::{ffi::OsString, os::unix::prelude::{OsStringExt, OsStrExt}, collections::HashMap};

use departure::{Constant, Function, Instruction, OpCode};

/// Go through an instruction and see whether or not it loads a global, moves
/// it, or writes over it, and then update the list of registers on the stack
/// which still hold a reference to that global. Of course, this analysis
/// doesn't consider conditionals, and if someone wrote something like this,
/// the function would output incorrect results:
/// ```lua
/// local a = require -- assume require is the global to look for
/// if math.random() < .5 then
///     a = tostring
/// end
/// a("hello.lua")
/// ```
/// This analysis also doesn't account for the return value of functions,
/// their parameters, or the function's upvalues.
fn global_loads(
    registers_holding_global: &mut Vec<i32>,
    next_instruction: &Instruction,
    proto: &Function,
    global: &str,
) {
    match next_instruction.op_code {
        OpCode::GetGlobal => {
            let constant = &proto.constants[next_instruction.bx.unwrap() as usize];
            match constant {
                Constant::String(s) => {
                    if s.to_string_lossy() == global {
                        registers_holding_global.push(next_instruction.a);
                    }
                }
                _ => {} // I don't know how a global wouldn't be a string
            }
        }
        OpCode::Move => {
            if registers_holding_global.contains(&next_instruction.b.unwrap()) {
                registers_holding_global.retain(|v| *v != next_instruction.b.unwrap()); // Delete the register holding the global, it's moved
                registers_holding_global.push(next_instruction.a);
            }
        }
        OpCode::Loadk | OpCode::LoadBool => {
            registers_holding_global.retain(|v| *v != next_instruction.a);
        }
        OpCode::LoadNil => {
            for r in next_instruction.a..next_instruction.b.unwrap() + 1 {
                registers_holding_global.retain(|v| *v != r);
            }
        }
        OpCode::GetUpval => {
            registers_holding_global.retain(|v| *v != next_instruction.a);
        }
        OpCode::GetTable | OpCode::NewTable => {
            registers_holding_global.retain(|v| *v != next_instruction.a);
        }
        OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div | OpCode::Mod | OpCode::Pow | OpCode::Concat => {
            registers_holding_global.retain(|v| *v != next_instruction.a);
        }
        OpCode::Unm | OpCode::Not | OpCode::Len => {
            registers_holding_global.retain(|v| *v != next_instruction.a);
        }
        OpCode::Closure => {
            registers_holding_global.retain(|v| *v != next_instruction.a);
        }
        _ => {}
    }
}

/// Using the list of registers that currently hold a reference to the global,
/// figure out if the current next instruction is calling the global, and if
/// it is, return the register holding the argument.
fn analyze_call(
    registers_holding_global: &mut Vec<i32>,
    next_instruction: &Instruction,
    proto: &Function,
) -> Option<Vec<i32>> {
    return match next_instruction.op_code {
        OpCode::Call => {
            if registers_holding_global.contains(&next_instruction.a) {
                todo!()
            } else {
                return None;
            }
        }
        _ => None,
    };
}

fn analyze_string_mutability(
    constant_strings: &mut HashMap<i32, OsString>,
    next_instruction: &Instruction,
    proto: &Function,
) {
    match next_instruction.op_code {
        OpCode::Loadk => {
            let constant = &proto.constants[next_instruction.bx.unwrap() as usize];
            match constant {
                Constant::String(s) => {
                    let str_const = OsString::from_vec(s.as_bytes().to_vec());
                    constant_strings.insert(next_instruction.a, str_const);
                }
                _ => {}
            }
        }
        OpCode::Move => {
            if constant_strings.contains_key(&next_instruction.b.unwrap()) {
                let chars = constant_strings.get(&next_instruction.b.unwrap()).unwrap().as_bytes();
                let copy = OsString::from_vec(chars.to_vec());
                constant_strings.remove(&next_instruction.b.unwrap());
                constant_strings.insert(next_instruction.a, copy);
            }
        }
        OpCode::LoadBool => {
            constant_strings.remove(&next_instruction.a);
        }
        OpCode::LoadNil => {
            for r in next_instruction.a..next_instruction.b.unwrap() + 1 {
                constant_strings.remove(&r);
            }
        }
        OpCode::GetUpval | OpCode::GetGlobal | OpCode::SetGlobal | OpCode::SetUpval => { // Lets prevent people from making this string a global, or an upvalue
            constant_strings.remove(&next_instruction.a);
        }
        OpCode::GetTable | OpCode::NewTable => {
            constant_strings.remove(&next_instruction.a);
        }
        OpCode::Not | OpCode::Len | OpCode::Closure => {
            constant_strings.remove(&next_instruction.a);
        }
        OpCode::Concat => {
            constant_strings.remove(&next_instruction.a);
            let concat_regs = next_instruction.b.unwrap()..next_instruction.c.unwrap() + 1;
            for r in concat_regs {
                constant_strings.remove(&r);
            }
        }
        _ => {}
    }
}

/// Analyzes the provided function to figure out which possible constant strings
/// are `require`d. See `global_loads` to learn the limitations of this
/// analysis.
pub fn get_imports(func: &Function) -> Vec<String> {
    let mut require_registers: Vec<i32> = vec![];
    for i in &func.instructions {
        global_loads(&mut require_registers, i, func, "require");
        std::thread::sleep_ms(1000);
        println!("{:?}", require_registers);
    }

    todo!();
}
