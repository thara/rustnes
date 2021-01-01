use crate::types::{word, Byte, Word};

use super::cpu::{page_crossed, CPU};

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
            Self::Implicit => word(0x00),
            Self::Accumulator => cpu.a.word(),
            Self::Immediate => {
                let operand = cpu.pc;
                cpu.pc += 1;
                operand
            }
            Self::ZeroPage => {
                let operand = cpu.read(cpu.pc).word() & 0xFF;
                cpu.pc += 1;
                operand
            }
            Self::ZeroPageX => {
                let operand = (cpu.read(cpu.pc).word() + cpu.x.word()) & 0xFF;
                cpu.pc += 1;
                cpu.cycles += 1;
                operand
            }
            Self::ZeroPageY => {
                let operand = (cpu.read(cpu.pc).word() + cpu.y.word()) & 0xFF;
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
                let operand = (data + cpu.x.word()) & 0xFFFF;
                cpu.pc += 2;
                cpu.cycles += 1;
                if *penalty && page_crossed(operand, data) {
                    cpu.cycles += 1;
                }
                operand
            }
            Self::AbsoluteY { penalty } => {
                let data = cpu.read_word(cpu.pc);
                let operand = (data + cpu.y.word()) & 0xFFFF;
                cpu.pc += 2;
                cpu.cycles += 1;
                if *penalty && page_crossed(operand, data) {
                    cpu.cycles += 1;
                }
                operand
            }
            Self::Relative => {
                let operand = cpu.read(cpu.pc).word();
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
                let operand = cpu.read_on_indirect((data + cpu.x).word() & 0xFF);
                cpu.pc += 1;
                cpu.cycles += 1;
                operand
            }
            Self::IndirectIndexed => {
                let y = cpu.y.word();
                let data = cpu.read(cpu.pc).word();
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
