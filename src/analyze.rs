use crate::instr::{Instruction, Function, OpCode, Constant};

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
        },
        OpCode::Move => {
            if registers_holding_global.contains(&next_instruction.b.unwrap()) {
                registers_holding_global.retain(|v| *v != next_instruction.b.unwrap());
                registers_holding_global.push(next_instruction.a);
            }
        }
        _ => {}
    }
}