use crate::types::{word, Byte, Word};

use super::addressing_modes::{AddressingMode, Immediate, Operand};
use super::cpu::{Opcode, CPU};

trait Instruction {
    fn execute(cpu: &mut CPU, operand: Operand);
}

pub struct Operation {
    get_operand: Box<dyn FnMut(&mut CPU) -> Operand>,
    exec: Box<dyn FnMut(&mut CPU, Operand)>,
}

impl Operation {
    fn execute(&mut self, cpu: &mut CPU) {
        let operand = (self.get_operand)(cpu);
        (self.exec)(cpu, operand)
    }
}

fn new<I: Instruction, A: AddressingMode>() -> Operation {
    Operation {
        get_operand: Box::new(|cpu| A::get_operand(cpu)),
        exec: Box::new(|cpu, operand| I::execute(cpu, operand)),
    }
}

pub fn decode(opcode: Opcode) -> Operation {
    new::<LDA, Immediate>()
}

struct LDA;
impl Instruction for LDA {
    fn execute(cpu: &mut CPU, operand: Operand) {
        cpu.a = cpu.read(operand)
    }
}
