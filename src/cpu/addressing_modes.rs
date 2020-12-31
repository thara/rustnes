use crate::types::{word, Byte, Word};

use super::cpu::CPU;

pub type Operand = Word;

pub trait AddressingMode {
    fn get_operand(cpu: &mut CPU) -> Operand;
}

pub struct Implicit;
impl AddressingMode for Implicit {
    fn get_operand(_cpu: &mut CPU) -> Operand {
        word(0x00)
    }
}

pub struct Accumulator;
impl AddressingMode for Accumulator {
    fn get_operand(cpu: &mut CPU) -> Operand {
        cpu.a.word()
    }
}

pub struct Immediate;
impl AddressingMode for Immediate {
    fn get_operand(cpu: &mut CPU) -> Operand {
        let operand = cpu.pc;
        cpu.pc += 1;
        operand
    }
}

pub struct ZeroPage;
impl AddressingMode for ZeroPage {
    fn get_operand(cpu: &mut CPU) -> Operand {
        let operand = cpu.read(cpu.pc).word() & 0xFF;
        cpu.pc += 1;
        operand
    }
}

pub struct ZeroPageX;
impl AddressingMode for ZeroPageX {
    fn get_operand(cpu: &mut CPU) -> Operand {
        cpu.cycles += 1;

        let operand = (cpu.read(cpu.pc).word() + cpu.x.word()) & 0xFF;
        cpu.pc += 1;
        operand
    }
}

pub struct ZeroPageY;
impl AddressingMode for ZeroPageY {
    fn get_operand(cpu: &mut CPU) -> Operand {
        cpu.cycles += 1;

        let operand = (cpu.read(cpu.pc).word() + cpu.y.word()) & 0xFF;
        cpu.pc += 1;
        operand
    }
}

pub struct Absolute;
impl AddressingMode for Absolute {
    fn get_operand(cpu: &mut CPU) -> Operand {
        let operand = cpu.read_word(cpu.pc);
        cpu.pc += 2;
        operand
    }
}

pub struct AbsoluteX {}
impl AddressingMode for AbsoluteX {
    fn get_operand(cpu: &mut CPU) -> Operand {
        let data = cpu.read_word(cpu.pc);
        let operand = (data + cpu.x.word()) & 0xFFFF;
        cpu.pc += 2;
        cpu.cycles += 1;
        operand
    }
}

pub struct AbsoluteXWithPenalty {}
impl AddressingMode for AbsoluteXWithPenalty {
    fn get_operand(cpu: &mut CPU) -> Operand {
        let data = cpu.read_word(cpu.pc);
        let operand = (data + cpu.x.word()) & 0xFFFF;
        cpu.pc += 2;
        if operand.page_crossed(data) {
            cpu.cycles += 1;
        }
        operand
    }
}

pub struct AbsoluteY {}
impl AddressingMode for AbsoluteY {
    fn get_operand(cpu: &mut CPU) -> Operand {
        let data = cpu.read_word(cpu.pc);
        let operand = (data + cpu.y.word()) & 0xFFFF;
        cpu.pc += 2;
        cpu.cycles += 1;
        operand
    }
}

pub struct AbsoluteYWithPenalty {}
impl AddressingMode for AbsoluteYWithPenalty {
    fn get_operand(cpu: &mut CPU) -> Operand {
        let data = cpu.read_word(cpu.pc);
        let operand = (data + cpu.y.word()) & 0xFFFF;
        cpu.pc += 2;
        if operand.page_crossed(data) {
            cpu.cycles += 1;
        }
        operand
    }
}

pub struct Relative {}
impl AddressingMode for Relative {
    fn get_operand(cpu: &mut CPU) -> Operand {
        let operand = cpu.read(cpu.pc).word();
        cpu.pc += 1;
        operand
    }
}

pub struct Indirect {}
impl AddressingMode for Indirect {
    fn get_operand(cpu: &mut CPU) -> Operand {
        let data = cpu.read_word(cpu.pc);
        let operand = cpu.read_on_indirect(data);
        cpu.pc += 2;
        operand
    }
}

pub struct IndexedIndirect {}
impl AddressingMode for IndexedIndirect {
    fn get_operand(cpu: &mut CPU) -> Operand {
        let data = cpu.read(cpu.pc);
        let operand = cpu.read_on_indirect((data + cpu.x).word() & 0xFF);
        cpu.pc += 1;
        cpu.cycles += 1;
        operand
    }
}

pub struct IndirectIndexed {}
impl AddressingMode for IndirectIndexed {
    fn get_operand(cpu: &mut CPU) -> Operand {
        let y = cpu.y.word();
        let data = cpu.read(cpu.pc).word();
        let operand = cpu.read_on_indirect(data) + y;
        cpu.pc += 1;
        if y.page_crossed(operand - y) {
            cpu.cycles += 1;
        }
        operand
    }
}
