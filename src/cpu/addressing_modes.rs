use crate::types::Word;

use super::{page_crossed_u16, CPU};

pub type Operand = Word;

// http://wiki.nesdev.com/w/index.php/CPU_addressing_modes
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
            Self::Implicit => Word::from(0x00u16),
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
                if *penalty {
                    if page_crossed_u16(cpu.x, data) {
                        cpu.cycles += 1;
                    }
                } else {
                    cpu.cycles += 1;
                }
                operand
            }
            Self::AbsoluteY { penalty } => {
                let data = cpu.read_word(cpu.pc);
                let operand = data + Word::from(cpu.y);
                cpu.pc += 2;
                if *penalty {
                    if page_crossed_u16(cpu.y, data) {
                        cpu.cycles += 1;
                    }
                } else {
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
                if page_crossed_u16(y, operand - y) {
                    cpu.cycles += 1;
                }
                operand
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::CPU;
    use crate::types::Memory;

    fn new_cpu() -> CPU {
        let test_mem: Box<dyn Memory> = Box::new([0; 0x10000]);
        let mut cpu = CPU::new(test_mem);
        cpu.x = 0x05.into();
        cpu.y = 0x80.into();
        cpu.pc = 0x8234.into();
        cpu.write(0x8234.into(), 0x90.into());
        cpu.write(0x8235.into(), 0x94.into());
        cpu.write(0x9490.into(), 0x33.into());
        cpu.write(0x9491.into(), 0x81.into());
        cpu.write(0x8234.into(), 0x90.into());
        cpu.write(0x8235.into(), 0x94.into());
        cpu.write(0x9490.into(), 0x33.into());
        cpu.write(0x9491.into(), 0x81.into());
        cpu
    }

    #[test]
    fn implicit() {
        let mut cpu = new_cpu();

        let before = cpu.pc;
        let operand = AddressingMode::Implicit.get_operand(&mut cpu);
        assert_eq!(operand, 0x00u16.into());
        assert_eq!(cpu.pc - before, 0u16.into());
    }

    #[test]
    fn accumulator() {
        let mut cpu = new_cpu();
        cpu.a = 0xFA.into();

        let before = cpu.pc;
        let operand = AddressingMode::Accumulator.get_operand(&mut cpu);
        assert_eq!(operand, 0xFAu16.into());
        assert_eq!(cpu.pc - before, 0u16.into());
    }

    #[test]
    fn immediate() {
        let mut cpu = new_cpu();

        let before = cpu.pc;
        let operand = AddressingMode::Immediate.get_operand(&mut cpu);
        assert_eq!(operand, 0x8234u16.into());
        assert_eq!(cpu.pc - before, 1u16.into());
    }

    #[test]
    fn zero_page() {
        let mut cpu = new_cpu();

        let before = cpu.pc;
        let operand = AddressingMode::ZeroPage.get_operand(&mut cpu);
        assert_eq!(operand, 0x0090u16.into());
        assert_eq!(cpu.pc - before, 1u16.into());
    }

    #[test]
    fn zero_page_x() {
        let mut cpu = new_cpu();

        let before = cpu.pc;
        let operand = AddressingMode::ZeroPageX.get_operand(&mut cpu);
        assert_eq!(operand, 0x0095u16.into()); // 0x90 + 0x05 & 0xFF
        assert_eq!(cpu.pc - before, 1u16.into());
    }

    #[test]
    fn zero_page_y() {
        let mut cpu = new_cpu();

        let before = cpu.pc;
        let operand = AddressingMode::ZeroPageY.get_operand(&mut cpu);
        assert_eq!(operand, 0x0010u16.into()); // (0x90 + 0x80) & 0xFF
        assert_eq!(cpu.pc - before, 1u16.into());
    }

    #[test]
    fn absolute() {
        let mut cpu = new_cpu();

        let before = cpu.pc;
        let operand = AddressingMode::Absolute.get_operand(&mut cpu);
        assert_eq!(operand, 0x9490u16.into());
        assert_eq!(cpu.pc - before, 2u16.into());
    }

    #[test]
    fn absolute_x() {
        let mut cpu = new_cpu();

        let before = cpu.pc;
        let operand = AddressingMode::AbsoluteX { penalty: false }.get_operand(&mut cpu);
        assert_eq!(operand, 0x9495u16.into()); // 0x9490 + 0x05
        assert_eq!(cpu.pc - before, 2u16.into());
    }

    #[test]
    fn absolute_y() {
        let mut cpu = new_cpu();

        let before = cpu.pc;
        let operand = AddressingMode::AbsoluteY { penalty: false }.get_operand(&mut cpu);
        assert_eq!(operand, 0x9510u16.into()); // 0x9490 + 0x80
        assert_eq!(cpu.pc - before, 2u16.into());
    }

    #[test]
    fn relative() {
        let mut cpu = new_cpu();
        cpu.pc = 0x0050u16.into();
        cpu.write(0x0050u16.into(), 0x78.into());

        let before = cpu.pc;
        let operand = AddressingMode::Relative.get_operand(&mut cpu);
        assert_eq!(operand, 0x78u16.into());
        assert_eq!(cpu.pc - before, 1u16.into());
    }

    #[test]
    fn indirect() {
        let mut cpu = new_cpu();

        let before = cpu.pc;
        let operand = AddressingMode::Indirect.get_operand(&mut cpu);
        assert_eq!(operand, 0x8133u16.into()); // 0x33 + (0x81 << 8)
        assert_eq!(cpu.pc - before, 2u16.into());
    }

    #[test]
    fn indexed_indirect() {
        let mut cpu = new_cpu();
        cpu.write(0x0095u16.into(), 0xFF.into());
        cpu.write(0x0096u16.into(), 0xF0.into());

        let before = cpu.pc;
        let operand = AddressingMode::IndexedIndirect.get_operand(&mut cpu);
        assert_eq!(operand, 0xF0FFu16.into()); // 0xFF + (0xF0 << 8)
        assert_eq!(cpu.pc - before, 1u16.into());
    }

    #[test]
    fn indirect_indexed() {
        let mut cpu = new_cpu();
        cpu.write(0x0090u16.into(), 0x43.into());
        cpu.write(0x0091u16.into(), 0xC0.into());

        let before = cpu.pc;
        let operand = AddressingMode::IndirectIndexed.get_operand(&mut cpu);
        assert_eq!(operand, 0xC0C3u16.into()); // 0xC043 + Y
        assert_eq!(cpu.pc - before, 1u16.into());
    }
}
