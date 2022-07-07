use crate::instr::{Instruction, Function, OpCode, Constant};

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
fn global_loads(registers_holding_global: &mut Vec<i32>, next_instruction: &Instruction, proto: &Function, global: &str) {
    match next_instruction.op_code {
        OpCode::GetGlobal => {
            let constant = &proto.constants[next_instruction.bx.unwrap() as usize];
            match constant {
                Constant::String(s) => {
                    if s.to_string_lossy() == global {
                        registers_holding_global.push(next_instruction.a);
                    }
                },
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
        OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div | OpCode::Mod | OpCode::Pow => {
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

fn get_imports() {
    
}