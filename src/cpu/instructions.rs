use std::marker::PhantomData;

use crate::types::{word, Byte, Word};

use super::addressing_modes::{AddressingMode, Immediate, Operand};
use super::cpu::{cpu_status_operated_b, Opcode, CPU};

pub trait Instruction {
    fn execute(cpu: &mut CPU, operand: Operand);
}

pub fn decode(opcode: Opcode) -> (impl Instruction, impl AddressingMode) {
    match opcode {
        0xA9 => (LDA {}, Immediate {}),
        _ => (LDA {}, Immediate {}),
    }
}

// Implements for Load/Store Operations

// loadAccumulator
struct LDA;
impl Instruction for LDA {
    fn execute(cpu: &mut CPU, operand: Operand) {
        cpu.a = cpu.read(operand)
    }
}

// loadXRegister
struct LDX;
impl Instruction for LDX {
    fn execute(cpu: &mut CPU, operand: Operand) {
        cpu.x = cpu.read(operand)
    }
}

// loadYRegister
struct LDY;
impl Instruction for LDY {
    fn execute(cpu: &mut CPU, operand: Operand) {
        cpu.y = cpu.read(operand)
    }
}

// storeAccumulator
struct STA;
impl Instruction for STA {
    fn execute(cpu: &mut CPU, operand: Operand) {
        cpu.write(operand, cpu.a)
    }
}

// storeXRegister
struct STX;
impl Instruction for STX {
    fn execute(cpu: &mut CPU, operand: Operand) {
        cpu.write(operand, cpu.x)
    }
}

// storeYRegister
struct STY;
impl Instruction for STY {
    fn execute(cpu: &mut CPU, operand: Operand) {
        cpu.write(operand, cpu.y)
    }
}

// transferAccumulatorToX
struct TAX;
impl Instruction for TAX {
    fn execute(cpu: &mut CPU, _operand: Operand) {
        cpu.x = cpu.a;
        cpu.cycles += 1;
    }
}

// transferStackPointerToX
struct TSX;
impl Instruction for TSX {
    fn execute(cpu: &mut CPU, _operand: Operand) {
        cpu.x = cpu.s;
        cpu.cycles += 1;
    }
}

// transferAccumulatorToY
struct TAY;
impl Instruction for TAY {
    fn execute(cpu: &mut CPU, _operand: Operand) {
        cpu.y = cpu.s;
        cpu.cycles += 1;
    }
}

// transferXtoAccumulator
struct TXA;
impl Instruction for TXA {
    fn execute(cpu: &mut CPU, _operand: Operand) {
        cpu.a = cpu.x;
        cpu.cycles += 1;
    }
}

// transferXtoStackPointer
struct TXS;
impl Instruction for TXS {
    fn execute(cpu: &mut CPU, _operand: Operand) {
        cpu.s = cpu.x;
        cpu.cycles += 1;
    }
}

// transferYtoAccumulator
struct TYA;
impl Instruction for TYA {
    fn execute(cpu: &mut CPU, _operand: Operand) {
        cpu.a = cpu.y;
        cpu.cycles += 1;
    }
}

// pushAccumulator
struct PHA;
impl Instruction for PHA {
    fn execute(cpu: &mut CPU, _operand: Operand) {
        cpu.push_stack(cpu.a);
        cpu.cycles += 1;
    }
}

// pushProcessorStatus
struct PHP;
impl Instruction for PHP {
    fn execute(cpu: &mut CPU, _operand: Operand) {
        // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
        // http://visual6502.org/wiki/index.php?title=6502_BRK_and_B_bit
        cpu.push_stack(cpu.p | cpu_status_operated_b);
        cpu.cycles += 1;
    }
}

// pullAccumulator
struct PLA;
impl Instruction for PLA {
    fn execute(cpu: &mut CPU, _operand: Operand) {
        cpu.a = cpu.pull_stack();
        cpu.cycles += 2;
    }
}

// pullProcessorStatus
struct PLP;
impl Instruction for PLP {
    fn execute(cpu: &mut CPU, _operand: Operand) {
        // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
        // http://visual6502.org/wiki/index.php?title=6502_BRK_and_B_bit
        cpu.p = cpu.pull_stack() & 0x10 | 0x20;
        cpu.cycles += 2;
    }
}

// bitwiseANDwithAccumulator
struct AND;
impl Instruction for AND {
    fn execute(cpu: &mut CPU, operand: Operand) {
        cpu.a &= cpu.read(operand)
    }
}

// bitwiseExclusiveOR
struct EOR;
impl Instruction for EOR {
    fn execute(cpu: &mut CPU, operand: Operand) {
        cpu.a ^= cpu.read(operand)
    }
}

// bitwiseORwithAccumulator
struct ORA;
impl Instruction for ORA {
    fn execute(cpu: &mut CPU, operand: Operand) {
        cpu.a |= cpu.read(operand)
    }
}

// testBits
struct BIT;
impl Instruction for BIT {
    fn execute(cpu: &mut CPU, operand: Operand) {
        let value = cpu.read(operand);
        let data = cpu.a & value;
        cpu.p &= 0xC2; // Z, V, N
    }
}

// addWithCarry
struct ADC;
impl Instruction for ADC {
    fn execute(cpu: &mut CPU, operand: Operand) {
        let a = cpu.a;
        let val = cpu.read(operand);
        let mut result = a + val;

        if cpu.p.is_set(0b10) {
            result += 1;
        }

        cpu.p &= !0xC3; // C, Z, V, N

        // http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
        let a7 = a & 0x80;
        let v7 = val & 0x80;
        let c6 = a7 ^ v7 ^ (result & 0x80);
        let c7 = (a7 & v7) | (a7 & c6) | (v7 & c6);

        if c7.u8() == 1 {
            cpu.p |= 0x01; // C
        }
        if (c6 ^ c7).u8() == 1 {
            cpu.p |= 0x40; // V
        }

        cpu.a = result
    }
}

// subtractWithCarry
struct SBC;
impl Instruction for SBC {
    fn execute(cpu: &mut CPU, operand: Operand) {
        let a = cpu.a;
        let val = !cpu.read(operand);
        let mut result = a + val;

        if cpu.p.is_set(0b10) {
            result += 1;
        }

        cpu.p &= !0xC3; // C, Z, V, N

        // http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
        let a7 = a & 0x80;
        let v7 = val & 0x80;
        let c6 = a7 ^ v7 ^ (result & 0x80);
        let c7 = (a7 & v7) | (a7 & c6) | (v7 & c6);

        if c7.u8() == 1 {
            cpu.p |= 0x01; // C
        }
        if (c6 ^ c7).u8() == 1 {
            cpu.p |= 0x40; // V
        }

        cpu.a = result
    }
}

// compareAccumulator
struct CMP;
impl Instruction for CMP {
    fn execute(cpu: &mut CPU, operand: Operand) {
        let cmp = cpu.a.word() - cpu.read(operand).word();

        cpu.p &= !0x83; // C, Z, N
        cpu.set_zn(cmp.byte());
        if 0 <= cmp.u16() {
            cpu.p |= 0x01
        } else {
            cpu.p &= !0x01
        }
    }
}
