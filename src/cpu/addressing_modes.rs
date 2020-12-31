use crate::types::Word;

use super::{page_crossed, CPU};

pub type Operand = Word;

// http://wiki.nesdev.com/w/index.php/CPU_addressing_modes
pub enum AddressingMode {
    Implicit,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX { penalty: bool },
    AbsoluteY { penalty: bool },
    Relative,
    Indirect,
    IndexedIndirect,
    IndirectIndexed,
}

impl AddressingMode {
    pub fn get_operand(&self, cpu: &mut CPU) -> Operand {
        match self {
            Self::Implicit => Word::from(0x00),
            Self::Accumulator => cpu.a.into(),
            Self::Immediate => {
                let operand = cpu.pc;
                cpu.pc += 1;
                operand
            }
            Self::ZeroPage => {
                let operand = Word::from(cpu.read(cpu.pc)) & 0xFF;
                cpu.pc += 1;
                operand
            }
            Self::ZeroPageX => {
                let operand = (Word::from(cpu.read(cpu.pc)) + Word::from(cpu.x)) & 0xFF;
                cpu.pc += 1;
                cpu.cycles += 1;
                operand
            }
            Self::ZeroPageY => {
                let operand = (Word::from(cpu.read(cpu.pc)) + Word::from(cpu.y)) & 0xFF;
                cpu.pc += 1;
                cpu.cycles += 1;
                operand
            }
            Self::Absolute => {
                let operand = cpu.read_word(cpu.pc);
                cpu.pc += 2;
                operand
            }
            Self::AbsoluteX { penalty } => {
                let data = cpu.read_word(cpu.pc);
                let operand = data + Word::from(cpu.x);
                cpu.pc += 2;
                cpu.cycles += 1;
                if *penalty && page_crossed(operand, data) {
                    cpu.cycles += 1;
                }
                operand
            }
            Self::AbsoluteY { penalty } => {
                let data = cpu.read_word(cpu.pc);
                let operand = data + Word::from(cpu.y);
                cpu.pc += 2;
                cpu.cycles += 1;
                if *penalty && page_crossed(operand, data) {
                    cpu.cycles += 1;
                }
                operand
            }
            Self::Relative => {
                let operand: Word = cpu.read(cpu.pc).into();
                cpu.pc += 1;
                operand
            }
            Self::Indirect => {
                let data = cpu.read_word(cpu.pc);
                let operand = cpu.read_on_indirect(data);
                cpu.pc += 2;
                operand
            }
            Self::IndexedIndirect => {
                let data = cpu.read(cpu.pc);
                let operand = cpu.read_on_indirect(Word::from(data + cpu.x) & 0xFF);
                cpu.pc += 1;
                cpu.cycles += 1;
                operand
            }
            Self::IndirectIndexed => {
                let y: Word = cpu.y.into();
                let data: Word = cpu.read(cpu.pc).into();
                let operand = cpu.read_on_indirect(data) + y;
                cpu.pc += 1;
                if page_crossed(y, operand - y) {
                    cpu.cycles += 1;
                }
                operand
            }
        }
    }
}
