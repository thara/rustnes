use crate::types::{Byte, Word};

use super::addressing_modes::{AddressingMode, Operand};
use super::cpu::{page_crossed, CPU, CPU_STATUS_INTERRUPTED_B, CPU_STATUS_OPERATED_B};
use super::status::CPUStatus;

// http://obelisk.me.uk/6502/reference.html
pub enum Mnemonic {
    // Load/Store Operations
    LDA,
    LDX,
    LDY,
    STA,
    STX,
    STY,
    // Register Operations
    TAX,
    TSX,
    TAY,
    TXA,
    TXS,
    TYA,
    // Stack instructions
    PHA,
    PHP,
    PLA,
    PLP,
    // Logical instructions
    AND,
    EOR,
    ORA,
    BIT,
    // Arithmetic instructions
    ADC,
    SBC,
    CMP,
    CPX,
    CPY,
    // Increment/Decrement instructions
    INC,
    INX,
    INY,
    DEC,
    DEX,
    DEY,
    // Shift instructions
    ASL,
    LSR,
    ROL,
    ROR,
    // Jump instructions
    JMP,
    JSR,
    RTS,
    RTI,
    // Branch instructions
    BCC,
    BCS,
    BEQ,
    BMI,
    BNE,
    BPL,
    BVC,
    BVS,
    // Flag control instructions
    CLC,
    CLD,
    CLI,
    CLV,
    SEC,
    SED,
    SEI,
    // Misc
    BRK,
    NOP,
    // Unofficial
    LAX,
    SAX,
    DCP,
    ISB,
    SLO,
    RLA,
    SRE,
    RRA,
}

pub struct Opcode {
    mnemonic: Mnemonic,
    addressing_mode: AddressingMode,
}

pub fn decode(opcode: Byte) -> Opcode {
    let (m, am) = match opcode.u8() {
        0xA9 => (Mnemonic::LDA, AddressingMode::Immediate),
        0xA5 => (Mnemonic::LDA, AddressingMode::ZeroPage),
        0xB5 => (Mnemonic::LDA, AddressingMode::ZeroPageX),
        0xAD => (Mnemonic::LDA, AddressingMode::Absolute),
        0xBD => (Mnemonic::LDA, AddressingMode::AbsoluteX { penalty: true }),
        0xB9 => (Mnemonic::LDA, AddressingMode::AbsoluteY { penalty: true }),
        0xA1 => (Mnemonic::LDA, AddressingMode::IndexedIndirect),
        0xB1 => (Mnemonic::LDA, AddressingMode::IndirectIndexed),
        0xA2 => (Mnemonic::LDX, AddressingMode::Immediate),
        0xA6 => (Mnemonic::LDX, AddressingMode::ZeroPage),
        0xB6 => (Mnemonic::LDX, AddressingMode::ZeroPageY),
        0xAE => (Mnemonic::LDX, AddressingMode::Absolute),
        0xBE => (Mnemonic::LDX, AddressingMode::AbsoluteY { penalty: true }),
        0xA0 => (Mnemonic::LDY, AddressingMode::Immediate),
        0xA4 => (Mnemonic::LDY, AddressingMode::ZeroPage),
        0xB4 => (Mnemonic::LDY, AddressingMode::ZeroPageX),
        0xAC => (Mnemonic::LDY, AddressingMode::Absolute),
        0xBC => (Mnemonic::LDY, AddressingMode::AbsoluteX { penalty: true }),
        0x85 => (Mnemonic::STA, AddressingMode::ZeroPage),
        0x95 => (Mnemonic::STA, AddressingMode::ZeroPageX),
        0x8D => (Mnemonic::STA, AddressingMode::Absolute),
        0x9D => (Mnemonic::STA, AddressingMode::AbsoluteX { penalty: false }),
        0x99 => (Mnemonic::STA, AddressingMode::AbsoluteY { penalty: false }),
        0x81 => (Mnemonic::STA, AddressingMode::IndexedIndirect),
        0x91 => (Mnemonic::STA, AddressingMode::IndirectIndexed),
        0x86 => (Mnemonic::STX, AddressingMode::ZeroPage),
        0x96 => (Mnemonic::STX, AddressingMode::ZeroPageY),
        0x8E => (Mnemonic::STX, AddressingMode::Absolute),
        0x84 => (Mnemonic::STY, AddressingMode::ZeroPage),
        0x94 => (Mnemonic::STY, AddressingMode::ZeroPageX),
        0x8C => (Mnemonic::STY, AddressingMode::Absolute),
        0xAA => (Mnemonic::TAX, AddressingMode::Implicit),
        0xBA => (Mnemonic::TSX, AddressingMode::Implicit),
        0xA8 => (Mnemonic::TAY, AddressingMode::Implicit),
        0x8A => (Mnemonic::TXA, AddressingMode::Implicit),
        0x9A => (Mnemonic::TXS, AddressingMode::Implicit),
        0x98 => (Mnemonic::TYA, AddressingMode::Implicit),

        0x48 => (Mnemonic::PHA, AddressingMode::Implicit),
        0x08 => (Mnemonic::PHP, AddressingMode::Implicit),
        0x68 => (Mnemonic::PLA, AddressingMode::Implicit),
        0x28 => (Mnemonic::PLP, AddressingMode::Implicit),

        0x29 => (Mnemonic::AND, AddressingMode::Immediate),
        0x25 => (Mnemonic::AND, AddressingMode::ZeroPage),
        0x35 => (Mnemonic::AND, AddressingMode::ZeroPageX),
        0x2D => (Mnemonic::AND, AddressingMode::Absolute),
        0x3D => (Mnemonic::AND, AddressingMode::AbsoluteX { penalty: true }),
        0x39 => (Mnemonic::AND, AddressingMode::AbsoluteY { penalty: true }),
        0x21 => (Mnemonic::AND, AddressingMode::IndexedIndirect),
        0x31 => (Mnemonic::AND, AddressingMode::IndirectIndexed),
        0x49 => (Mnemonic::EOR, AddressingMode::Immediate),
        0x45 => (Mnemonic::EOR, AddressingMode::ZeroPage),
        0x55 => (Mnemonic::EOR, AddressingMode::ZeroPageX),
        0x4D => (Mnemonic::EOR, AddressingMode::Absolute),
        0x5D => (Mnemonic::EOR, AddressingMode::AbsoluteX { penalty: true }),
        0x59 => (Mnemonic::EOR, AddressingMode::AbsoluteY { penalty: true }),
        0x41 => (Mnemonic::EOR, AddressingMode::IndexedIndirect),
        0x51 => (Mnemonic::EOR, AddressingMode::IndirectIndexed),
        0x09 => (Mnemonic::ORA, AddressingMode::Immediate),
        0x05 => (Mnemonic::ORA, AddressingMode::ZeroPage),
        0x15 => (Mnemonic::ORA, AddressingMode::ZeroPageX),
        0x0D => (Mnemonic::ORA, AddressingMode::Absolute),
        0x1D => (Mnemonic::ORA, AddressingMode::AbsoluteX { penalty: true }),
        0x19 => (Mnemonic::ORA, AddressingMode::AbsoluteY { penalty: true }),
        0x01 => (Mnemonic::ORA, AddressingMode::IndexedIndirect),
        0x11 => (Mnemonic::ORA, AddressingMode::IndirectIndexed),
        0x24 => (Mnemonic::BIT, AddressingMode::ZeroPage),
        0x2C => (Mnemonic::BIT, AddressingMode::Absolute),

        0x69 => (Mnemonic::ADC, AddressingMode::Immediate),
        0x65 => (Mnemonic::ADC, AddressingMode::ZeroPage),
        0x75 => (Mnemonic::ADC, AddressingMode::ZeroPageX),
        0x6D => (Mnemonic::ADC, AddressingMode::Absolute),
        0x7D => (Mnemonic::ADC, AddressingMode::AbsoluteX { penalty: true }),
        0x79 => (Mnemonic::ADC, AddressingMode::AbsoluteY { penalty: true }),
        0x61 => (Mnemonic::ADC, AddressingMode::IndexedIndirect),
        0x71 => (Mnemonic::ADC, AddressingMode::IndirectIndexed),
        0xE9 => (Mnemonic::SBC, AddressingMode::Immediate),
        0xE5 => (Mnemonic::SBC, AddressingMode::ZeroPage),
        0xF5 => (Mnemonic::SBC, AddressingMode::ZeroPageX),
        0xED => (Mnemonic::SBC, AddressingMode::Absolute),
        0xFD => (Mnemonic::SBC, AddressingMode::AbsoluteX { penalty: true }),
        0xF9 => (Mnemonic::SBC, AddressingMode::AbsoluteY { penalty: true }),
        0xE1 => (Mnemonic::SBC, AddressingMode::IndexedIndirect),
        0xF1 => (Mnemonic::SBC, AddressingMode::IndirectIndexed),
        0xC9 => (Mnemonic::CMP, AddressingMode::Immediate),
        0xC5 => (Mnemonic::CMP, AddressingMode::ZeroPage),
        0xD5 => (Mnemonic::CMP, AddressingMode::ZeroPageX),
        0xCD => (Mnemonic::CMP, AddressingMode::Absolute),
        0xDD => (Mnemonic::CMP, AddressingMode::AbsoluteX { penalty: true }),
        0xD9 => (Mnemonic::CMP, AddressingMode::AbsoluteY { penalty: true }),
        0xC1 => (Mnemonic::CMP, AddressingMode::IndexedIndirect),
        0xD1 => (Mnemonic::CMP, AddressingMode::IndirectIndexed),
        0xE0 => (Mnemonic::CPX, AddressingMode::Immediate),
        0xE4 => (Mnemonic::CPX, AddressingMode::ZeroPage),
        0xEC => (Mnemonic::CPX, AddressingMode::Absolute),
        0xC0 => (Mnemonic::CPY, AddressingMode::Immediate),
        0xC4 => (Mnemonic::CPY, AddressingMode::ZeroPage),
        0xCC => (Mnemonic::CPY, AddressingMode::Absolute),

        0xE6 => (Mnemonic::INC, AddressingMode::ZeroPage),
        0xF6 => (Mnemonic::INC, AddressingMode::ZeroPageX),
        0xEE => (Mnemonic::INC, AddressingMode::Absolute),
        0xFE => (Mnemonic::INC, AddressingMode::AbsoluteX { penalty: false }),
        0xE8 => (Mnemonic::INX, AddressingMode::Implicit),
        0xC8 => (Mnemonic::INY, AddressingMode::Implicit),
        0xC6 => (Mnemonic::DEC, AddressingMode::ZeroPage),
        0xD6 => (Mnemonic::DEC, AddressingMode::ZeroPageX),
        0xCE => (Mnemonic::DEC, AddressingMode::Absolute),
        0xDE => (Mnemonic::DEC, AddressingMode::AbsoluteX { penalty: false }),
        0xCA => (Mnemonic::DEX, AddressingMode::Implicit),
        0x88 => (Mnemonic::DEY, AddressingMode::Implicit),

        0x0A => (Mnemonic::ASL, AddressingMode::Accumulator),
        0x06 => (Mnemonic::ASL, AddressingMode::ZeroPage),
        0x16 => (Mnemonic::ASL, AddressingMode::ZeroPageX),
        0x0E => (Mnemonic::ASL, AddressingMode::Absolute),
        0x1E => (Mnemonic::ASL, AddressingMode::AbsoluteX { penalty: false }),
        0x4A => (Mnemonic::LSR, AddressingMode::Accumulator),
        0x46 => (Mnemonic::LSR, AddressingMode::ZeroPage),
        0x56 => (Mnemonic::LSR, AddressingMode::ZeroPageX),
        0x4E => (Mnemonic::LSR, AddressingMode::Absolute),
        0x5E => (Mnemonic::LSR, AddressingMode::AbsoluteX { penalty: false }),
        0x2A => (Mnemonic::ROL, AddressingMode::Accumulator),
        0x26 => (Mnemonic::ROL, AddressingMode::ZeroPage),
        0x36 => (Mnemonic::ROL, AddressingMode::ZeroPageX),
        0x2E => (Mnemonic::ROL, AddressingMode::Absolute),
        0x3E => (Mnemonic::ROL, AddressingMode::AbsoluteX { penalty: false }),
        0x6A => (Mnemonic::ROR, AddressingMode::Accumulator),
        0x66 => (Mnemonic::ROR, AddressingMode::ZeroPage),
        0x76 => (Mnemonic::ROR, AddressingMode::ZeroPageX),
        0x6E => (Mnemonic::ROR, AddressingMode::Absolute),
        0x7E => (Mnemonic::ROR, AddressingMode::AbsoluteX { penalty: false }),

        0x4C => (Mnemonic::JMP, AddressingMode::Absolute),
        0x6C => (Mnemonic::JMP, AddressingMode::Indirect),
        0x20 => (Mnemonic::JSR, AddressingMode::Absolute),
        0x60 => (Mnemonic::RTS, AddressingMode::Implicit),
        0x40 => (Mnemonic::RTI, AddressingMode::Implicit),

        0x90 => (Mnemonic::BCC, AddressingMode::Relative),
        0xB0 => (Mnemonic::BCS, AddressingMode::Relative),
        0xF0 => (Mnemonic::BEQ, AddressingMode::Relative),
        0x30 => (Mnemonic::BMI, AddressingMode::Relative),
        0xD0 => (Mnemonic::BNE, AddressingMode::Relative),
        0x10 => (Mnemonic::BPL, AddressingMode::Relative),
        0x50 => (Mnemonic::BVC, AddressingMode::Relative),
        0x70 => (Mnemonic::BVS, AddressingMode::Relative),

        0x18 => (Mnemonic::CLC, AddressingMode::Implicit),
        0xD8 => (Mnemonic::CLD, AddressingMode::Implicit),
        0x58 => (Mnemonic::CLI, AddressingMode::Implicit),
        0xB8 => (Mnemonic::CLV, AddressingMode::Implicit),

        0x38 => (Mnemonic::SEC, AddressingMode::Implicit),
        0xF8 => (Mnemonic::SED, AddressingMode::Implicit),
        0x78 => (Mnemonic::SEI, AddressingMode::Implicit),

        0x00 => (Mnemonic::BRK, AddressingMode::Implicit),

        // Undocumented
        0xEB => (Mnemonic::SBC, AddressingMode::Immediate),

        0x04 | 0x44 | 0x64 => (Mnemonic::NOP, AddressingMode::ZeroPage),
        0x0C => (Mnemonic::NOP, AddressingMode::Absolute),
        0x14 | 0x34 | 0x54 | 0x74 | 0xD4 | 0xF4 => (Mnemonic::NOP, AddressingMode::ZeroPageX),
        0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xEA | 0xFA => (Mnemonic::NOP, AddressingMode::Implicit),
        0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => {
            (Mnemonic::NOP, AddressingMode::AbsoluteX { penalty: true })
        }
        0x80 | 0x82 | 0x89 | 0xC2 | 0xE2 => (Mnemonic::NOP, AddressingMode::Immediate),

        0xA3 => (Mnemonic::LAX, AddressingMode::IndexedIndirect),
        0xA7 => (Mnemonic::LAX, AddressingMode::ZeroPage),
        0xAF => (Mnemonic::LAX, AddressingMode::Absolute),
        0xB3 => (Mnemonic::LAX, AddressingMode::IndirectIndexed),
        0xB7 => (Mnemonic::LAX, AddressingMode::ZeroPageY),
        0xBF => (Mnemonic::LAX, AddressingMode::AbsoluteY { penalty: true }),

        0x83 => (Mnemonic::SAX, AddressingMode::IndexedIndirect),
        0x87 => (Mnemonic::SAX, AddressingMode::ZeroPage),
        0x8F => (Mnemonic::SAX, AddressingMode::Absolute),
        0x97 => (Mnemonic::SAX, AddressingMode::ZeroPageY),

        0xC3 => (Mnemonic::DCP, AddressingMode::IndexedIndirect),
        0xC7 => (Mnemonic::DCP, AddressingMode::ZeroPage),
        0xCF => (Mnemonic::DCP, AddressingMode::Absolute),
        0xD3 => (Mnemonic::DCP, AddressingMode::IndirectIndexed),
        0xD7 => (Mnemonic::DCP, AddressingMode::ZeroPageX),
        0xDB => (Mnemonic::DCP, AddressingMode::AbsoluteY { penalty: false }),
        0xDF => (Mnemonic::DCP, AddressingMode::AbsoluteX { penalty: false }),

        0xE3 => (Mnemonic::ISB, AddressingMode::IndexedIndirect),
        0xE7 => (Mnemonic::ISB, AddressingMode::ZeroPage),
        0xEF => (Mnemonic::ISB, AddressingMode::Absolute),
        0xF3 => (Mnemonic::ISB, AddressingMode::IndirectIndexed),
        0xF7 => (Mnemonic::ISB, AddressingMode::ZeroPageX),
        0xFB => (Mnemonic::ISB, AddressingMode::AbsoluteY { penalty: false }),
        0xFF => (Mnemonic::ISB, AddressingMode::AbsoluteX { penalty: false }),

        0x03 => (Mnemonic::SLO, AddressingMode::IndexedIndirect),
        0x07 => (Mnemonic::SLO, AddressingMode::ZeroPage),
        0x0F => (Mnemonic::SLO, AddressingMode::Absolute),
        0x13 => (Mnemonic::SLO, AddressingMode::IndirectIndexed),
        0x17 => (Mnemonic::SLO, AddressingMode::ZeroPageX),
        0x1B => (Mnemonic::SLO, AddressingMode::AbsoluteY { penalty: false }),
        0x1F => (Mnemonic::SLO, AddressingMode::AbsoluteX { penalty: false }),

        0x23 => (Mnemonic::RLA, AddressingMode::IndexedIndirect),
        0x27 => (Mnemonic::RLA, AddressingMode::ZeroPage),
        0x2F => (Mnemonic::RLA, AddressingMode::Absolute),
        0x33 => (Mnemonic::RLA, AddressingMode::IndirectIndexed),
        0x37 => (Mnemonic::RLA, AddressingMode::ZeroPageX),
        0x3B => (Mnemonic::RLA, AddressingMode::AbsoluteY { penalty: false }),
        0x3F => (Mnemonic::RLA, AddressingMode::AbsoluteX { penalty: false }),

        0x43 => (Mnemonic::SRE, AddressingMode::IndexedIndirect),
        0x47 => (Mnemonic::SRE, AddressingMode::ZeroPage),
        0x4F => (Mnemonic::SRE, AddressingMode::Absolute),
        0x53 => (Mnemonic::SRE, AddressingMode::IndirectIndexed),
        0x57 => (Mnemonic::SRE, AddressingMode::ZeroPageX),
        0x5B => (Mnemonic::SRE, AddressingMode::AbsoluteY { penalty: false }),
        0x5F => (Mnemonic::SRE, AddressingMode::AbsoluteX { penalty: false }),

        0x63 => (Mnemonic::RRA, AddressingMode::IndexedIndirect),
        0x67 => (Mnemonic::RRA, AddressingMode::ZeroPage),
        0x6F => (Mnemonic::RRA, AddressingMode::Absolute),
        0x73 => (Mnemonic::RRA, AddressingMode::IndirectIndexed),
        0x77 => (Mnemonic::RRA, AddressingMode::ZeroPageX),
        0x7B => (Mnemonic::RRA, AddressingMode::AbsoluteY { penalty: false }),
        0x7F => (Mnemonic::RRA, AddressingMode::AbsoluteX { penalty: false }),

        default => (Mnemonic::NOP, AddressingMode::Implicit),
    };
    Opcode {
        mnemonic: m,
        addressing_mode: am,
    }
}

// LoaD Accumulator
fn lda(cpu: &mut CPU, operand: Operand) {
    cpu.a = cpu.read(operand)
}

// LoaD X register
fn ldx(cpu: &mut CPU, operand: Operand) {
    cpu.x = cpu.read(operand)
}

// LoaD Y register
fn ldy(cpu: &mut CPU, operand: Operand) {
    cpu.y = cpu.read(operand)
}

// STore Accumulator
fn sta(cpu: &mut CPU, operand: Operand) {
    cpu.write(operand, cpu.a)
}

// STore X register
fn stx(cpu: &mut CPU, operand: Operand) {
    cpu.write(operand, cpu.x)
}

// STore Y register
fn sty(cpu: &mut CPU, operand: Operand) {
    cpu.write(operand, cpu.y)
}

// Transfer Accumulator to X
fn tax(cpu: &mut CPU, _operand: Operand) {
    cpu.x = cpu.a;
    cpu.cycles += 1;
}

// Transfer Stack pointer to X
fn tsx(cpu: &mut CPU, _operand: Operand) {
    cpu.x = cpu.s;
    cpu.cycles += 1;
}

// Transfer Accumulator to Y
fn tay(cpu: &mut CPU, _operand: Operand) {
    cpu.y = cpu.s;
    cpu.cycles += 1;
}

// Transfer X to Accumulator
fn txa(cpu: &mut CPU, _operand: Operand) {
    cpu.a = cpu.x;
    cpu.cycles += 1;
}

// Transfer X to Stack pointer
fn txs(cpu: &mut CPU, _operand: Operand) {
    cpu.s = cpu.x;
    cpu.cycles += 1;
}

// Transfer Y to Accumulator
fn tya(cpu: &mut CPU, _operand: Operand) {
    cpu.a = cpu.y;
    cpu.cycles += 1;
}

// PusH Accumulator
fn pha(cpu: &mut CPU, _operand: Operand) {
    cpu.push_stack(cpu.a);
    cpu.cycles += 1;
}

// PusH Processor status
fn php(cpu: &mut CPU, _operand: Operand) {
    // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
    // http://visual6502.org/wiki/index.php?title=6502_BRK_and_B_bit
    cpu.push_stack(cpu.p | CPUStatus::OPERATED_B);
    cpu.cycles += 1;
}

// PulL Accumulator
fn pla(cpu: &mut CPU, _operand: Operand) {
    cpu.a = cpu.pull_stack();
    cpu.cycles += 2;
}

// PulL Processor status
fn plp(cpu: &mut CPU, _operand: Operand) {
    // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
    // http://visual6502.org/wiki/index.php?title=6502_BRK_and_B_bit
    cpu.p = CPUStatus::from(cpu.pull_stack()) & !CPUStatus::B | CPUStatus::R;
    cpu.cycles += 2
}

// bitwise AND with accumulator
fn and(cpu: &mut CPU, operand: Operand) {
    let value = cpu.read(operand);
    cpu.a &= value
}

// bitwise Exclusive OR
fn eor(cpu: &mut CPU, operand: Operand) {
    let value = cpu.read(operand);
    cpu.a ^= value
}

// bitwise OR with Accumulator
fn ora(cpu: &mut CPU, operand: Operand) {
    let value = cpu.read(operand);
    cpu.a |= value
}

// test BITs
fn bit(cpu: &mut CPU, operand: Operand) {
    let value = cpu.read(operand);
    let data = cpu.a & value;
    cpu.p.update(CPUStatus::Z, data.u8() == 0);
    cpu.p.update(CPUStatus::V, data.is_set(6));
    cpu.p.update(CPUStatus::N, data.is_set(7));
}

// ADd with Carry
fn adc(cpu: &mut CPU, operand: Operand) {
    let a = cpu.a;
    let val = cpu.read(operand);
    let mut result = a + val;

    if cpu.p.is_set(CPUStatus::C) {
        result += 1;
    }

    cpu.p
        .unset(CPUStatus::C | CPUStatus::Z | CPUStatus::V | CPUStatus::N);

    // http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
    let a7 = a & 0x80;
    let v7 = val & 0x80;
    let c6 = a7 ^ v7 ^ (result & 0x80);
    let c7 = (a7 & v7) | (a7 & c6) | (v7 & c6);

    if c7.u8() == 1 {
        cpu.p.set(CPUStatus::C)
    }
    if (c6 ^ c7).u8() == 1 {
        cpu.p.set(CPUStatus::V)
    }

    cpu.a = result
}

// SuBtract with carry
fn sbc(cpu: &mut CPU, operand: Operand) {
    let a = cpu.a;
    let val = !cpu.read(operand);
    let mut result = a + val;

    if cpu.p.is_set(CPUStatus::C) {
        result += 1;
    }

    cpu.p
        .unset(CPUStatus::C | CPUStatus::Z | CPUStatus::V | CPUStatus::N);

    // http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
    let a7 = a & 0x80;
    let v7 = val & 0x80;
    let c6 = a7 ^ v7 ^ (result & 0x80);
    let c7 = (a7 & v7) | (a7 & c6) | (v7 & c6);

    if c7.u8() == 1 {
        cpu.p.set(CPUStatus::C)
    }
    if (c6 ^ c7).u8() == 1 {
        cpu.p.set(CPUStatus::V)
    }

    cpu.a = result
}

// CoMPare accumulator
fn cmp(cpu: &mut CPU, operand: Operand) {
    let cmp = Word::from(cpu.a) - Word::from(cpu.read(operand));

    cpu.p.unset(CPUStatus::C | CPUStatus::Z | CPUStatus::N);
    cpu.set_zn(cmp.byte());
    cpu.p.update(CPUStatus::C, 0 <= cmp.into())
}

// ComPare X register
fn cpx(cpu: &mut CPU, operand: Operand) {
    let value = cpu.read(operand);
    let cmp = cpu.x - value;

    cpu.p.unset(CPUStatus::C | CPUStatus::Z | CPUStatus::N);
    cpu.set_zn(cmp);
    cpu.p.update(CPUStatus::C, value <= cpu.x)
}

// ComPare Y register
fn cpy(cpu: &mut CPU, operand: Operand) {
    let value = cpu.read(operand);
    let cmp = cpu.y - value;

    cpu.p.unset(CPUStatus::C | CPUStatus::Z | CPUStatus::N);
    cpu.set_zn(cmp);
    cpu.p.update(CPUStatus::C, value <= cpu.y)
}

// INCrement memory
fn inc(cpu: &mut CPU, operand: Operand) {
    let result = cpu.read(operand) + 1;

    cpu.set_zn(result);
    cpu.write(operand, result);
    cpu.cycles += 1
}

// INcrement X register
fn inx(cpu: &mut CPU, operand: Operand) {
    cpu.x += 1;
    cpu.cycles += 1
}

// INcrement Y register
fn iny(cpu: &mut CPU, operand: Operand) {
    cpu.y += 1;
    cpu.cycles += 1
}

// DECrement memory
fn dec(cpu: &mut CPU, operand: Operand) {
    let result = cpu.read(operand) - 1;

    cpu.set_zn(result);
    cpu.write(operand, result);
    cpu.cycles += 1
}

// DEcrement X register
fn dex(cpu: &mut CPU, operand: Operand) {
    cpu.x -= 1;
    cpu.cycles += 1
}

// DEcrement Y register
fn dey(cpu: &mut CPU, operand: Operand) {
    cpu.y -= 1;
    cpu.cycles += 1
}

// Arithmetic Shift Left
fn asl(cpu: &mut CPU, operand: Operand) {
    let mut data = cpu.read(operand);

    cpu.p.unset(CPUStatus::C | CPUStatus::Z | CPUStatus::N);
    if data.is_set(7) {
        cpu.p.set(CPUStatus::C);
    }

    data <<= 1;
    cpu.set_zn(data);
    cpu.write(operand, data);
    cpu.cycles += 1;
}

fn asl_for_accumelator(cpu: &mut CPU, operand: Operand) {
    cpu.p.unset(CPUStatus::C | CPUStatus::Z | CPUStatus::N);
    if cpu.a.is_set(7) {
        cpu.p.set(CPUStatus::C);
    }
    cpu.a <<= 1;
    cpu.cycles += 1;
}

// Logical Shift Right
fn lsr(cpu: &mut CPU, operand: Operand) {
    let mut data = cpu.read(operand);

    cpu.p.unset(CPUStatus::C | CPUStatus::Z | CPUStatus::N);
    if data.is_set(0) {
        cpu.p.set(CPUStatus::C);
    }

    data >>= 1;
    cpu.set_zn(data);
    cpu.write(operand, data);
    cpu.cycles += 1;
}

fn lsr_for_accumelator(cpu: &mut CPU, operand: Operand) {
    cpu.p.unset(CPUStatus::C | CPUStatus::Z | CPUStatus::N);
    if cpu.a.is_set(0) {
        cpu.p.set(CPUStatus::C);
    }
    cpu.a >>= 1;
    cpu.cycles += 1;
}

// ROtate Left
fn rol(cpu: &mut CPU, operand: Operand) {
    let mut data = cpu.read(operand);
    let c = data & 0x80;

    data <<= 1;
    // C
    if cpu.p.is_set(CPUStatus::C) {
        data |= 0x01;
    }
    cpu.p.unset(CPUStatus::C | CPUStatus::Z | CPUStatus::N);
    if c.u8() == 0x80 {
        cpu.p.set(CPUStatus::C);
    }
    cpu.set_zn(data);
    cpu.write(operand, data);
    cpu.cycles += 1;
}

fn rol_for_accumelator(cpu: &mut CPU, operand: Operand) {
    let c = cpu.a & 0x80;

    let mut a = cpu.a << 1;
    cpu.p.unset(CPUStatus::C | CPUStatus::Z | CPUStatus::N);
    if c.u8() == 0x80 {
        cpu.p.set(CPUStatus::C);
    }
    cpu.a = a;
    cpu.cycles += 1;
}

// ROtate Right
fn ror(cpu: &mut CPU, operand: Operand) {
    let mut data = cpu.read(operand);
    let c = data & 0x80;

    data >>= 1;
    // C
    if cpu.p.is_set(CPUStatus::C) {
        data |= 0x80;
    }
    cpu.p.unset(CPUStatus::C | CPUStatus::Z | CPUStatus::N);
    if c.u8() == 1 {
        cpu.p.set(CPUStatus::C);
    }
    cpu.set_zn(data);
    cpu.write(operand, data);
    cpu.cycles += 1;
}

fn ror_for_accumelator(cpu: &mut CPU, operand: Operand) {
    let c = cpu.a & 0x80;

    let mut a = cpu.a >> 1;
    if cpu.p.is_set(CPUStatus::C) {
        a |= 0x80;
    }
    cpu.p.unset(CPUStatus::C | CPUStatus::Z | CPUStatus::N);
    if c.u8() == 0x80 {
        cpu.p.set(CPUStatus::C)
    }
    cpu.a = a;
    cpu.cycles += 1;
}

// JuMP
fn jmp(cpu: &mut CPU, operand: Operand) {
    cpu.pc = operand
}

// Jump to SubRoutine
fn jsr(cpu: &mut CPU, operand: Operand) {
    cpu.push_stack_word(cpu.pc - 1);
    cpu.cycles += 1;
    cpu.pc = operand
}

// ReTurn from Subroutine
fn rts(cpu: &mut CPU, operand: Operand) {
    cpu.cycles += 3;
    cpu.pc = cpu.pull_stack_word() + 1
}

// ReTurn from Interrupt
fn rti(cpu: &mut CPU, operand: Operand) {
    // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
    // http://visual6502.org/wiki/index.php?title=6502_BRK_and_B_bit
    cpu.cycles += 2;
    cpu.p = CPUStatus::from(cpu.pull_stack()) & !CPUStatus::B | CPUStatus::R;
    cpu.pc = cpu.pull_stack_word()
}

// Branch if Carry Clear
fn bcc(cpu: &mut CPU, operand: Operand) {
    if !cpu.p.is_set(CPUStatus::C) {
        branch(cpu, operand)
    }
}

// Branch if Carry Set
fn bcs(cpu: &mut CPU, operand: Operand) {
    if cpu.p.is_set(CPUStatus::C) {
        branch(cpu, operand)
    }
}

// Branch if EQual
fn beq(cpu: &mut CPU, operand: Operand) {
    if cpu.p.is_set(CPUStatus::Z) {
        branch(cpu, operand)
    }
}

// Branch if MInus
fn bmi(cpu: &mut CPU, operand: Operand) {
    if cpu.p.is_set(CPUStatus::N) {
        branch(cpu, operand)
    }
}

// Branch if NotEqual
fn bne(cpu: &mut CPU, operand: Operand) {
    if !cpu.p.is_set(CPUStatus::Z) {
        branch(cpu, operand)
    }
}

// Branch if PLus
fn bpl(cpu: &mut CPU, operand: Operand) {
    if cpu.p.is_set(CPUStatus::N) {
        branch(cpu, operand)
    }
}

// Branch if oVerflow Clear
fn bvc(cpu: &mut CPU, operand: Operand) {
    if !cpu.p.is_set(CPUStatus::V) {
        branch(cpu, operand)
    }
}

// Branch if oVerflow Set
fn bvs(cpu: &mut CPU, operand: Operand) {
    if cpu.p.is_set(CPUStatus::V) {
        branch(cpu, operand)
    }
}

// CLear Carry
fn clc(cpu: &mut CPU, operand: Operand) {
    cpu.p.unset(CPUStatus::C);
    cpu.cycles += 1
}

// CLear Decimal
fn cld(cpu: &mut CPU, operand: Operand) {
    cpu.p.unset(CPUStatus::D);
    cpu.cycles += 1
}

// Clear Interrupt
fn cli(cpu: &mut CPU, operand: Operand) {
    cpu.p.unset(CPUStatus::I);
    cpu.cycles += 1
}

// CLear oVerflow
fn clv(cpu: &mut CPU, operand: Operand) {
    cpu.p.unset(CPUStatus::V);
    cpu.cycles += 1
}

// SEt Carry flag
fn sec(cpu: &mut CPU, operand: Operand) {
    cpu.p.set(CPUStatus::C);
    cpu.cycles += 1
}

// SEt Decimal flag
fn sed(cpu: &mut CPU, operand: Operand) {
    cpu.p.set(CPUStatus::D);
    cpu.cycles += 1
}

// SEt Interrupt disable
fn sei(cpu: &mut CPU, operand: Operand) {
    cpu.p.set(CPUStatus::I);
    cpu.cycles += 1
}

// BReaK(force interrupt)
fn brk(cpu: &mut CPU, operand: Operand) {
    cpu.push_stack_word(cpu.pc);
    // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
    // http://visual6502.org/wiki/index.php?title=6502_BRK_and_B_bit
    cpu.push_stack(cpu.p | CPUStatus::INTERRUPTED_B);
    cpu.cycles += 1;
    cpu.pc = cpu.read_word(0xFFFE)
}

// No OPeration
fn nop(cpu: &mut CPU, operand: Operand) {
    cpu.cycles += 1;
}

fn branch(cpu: &mut CPU, operand: Operand) {
    cpu.cycles += 1;
    if page_crossed(operand, cpu.pc) {
        cpu.cycles += 1;
    }
    cpu.pc += operand
}
