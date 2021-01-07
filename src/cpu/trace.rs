use std::fmt;

use crate::types::{Byte, Memory, Word};

use super::addressing_modes::AddressingMode;
use super::instructions::{decode, Mnemonic, Opcode};
use super::{CPUCycle, CPU};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trace {
    pc: Word,
    operation: Byte,
    operand_1: Byte,
    operand_2: Byte,
    a: Byte,
    x: Byte,
    y: Byte,
    sp: Byte,
    p: Byte,
    cycle: CPUCycle,

    opcode: Opcode,
    assembly_code: String,
}

impl Trace {
    pub fn trace(cpu: &CPU) -> Self {
        let instruction = cpu.bus.read(cpu.pc);
        let opcode = decode(instruction);
        let assembly_code = to_assembly_code(instruction, opcode, &cpu);
        Self {
            pc: cpu.pc,
            operation: cpu.bus.read(cpu.pc),
            operand_1: cpu.bus.read(cpu.pc + 1),
            operand_2: cpu.bus.read(cpu.pc + 2),
            a: cpu.a,
            x: cpu.x,
            y: cpu.y,
            sp: cpu.s,
            p: cpu.p.into(),
            cycle: cpu.cycles,
            opcode,
            assembly_code,
        }
    }
}

impl fmt::Display for Trace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let len = self.opcode.addressing_mode.instruction_length();
        let machine_code = match len {
            3 => format!(
                "{:02X} {:02X} {:02X}",
                self.operation, self.operand_1, self.operand_2
            ),
            2 => format!("{:02X} {:02X}   ", self.operation, self.operand_1),
            _ => format!("{:02X}      ", self.operation),
        };
        write!(
            f,
            "{:04X}  {} {}A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}",
            self.pc,
            machine_code,
            self.assembly_code,
            self.a,
            self.x,
            self.y,
            self.p,
            self.sp,
            self.cycle
        )
    }
}

impl CPU {
    fn operand_1(&self) -> Byte {
        self.bus.read(self.pc + 1)
    }

    fn operand_2(&self) -> Byte {
        self.bus.read(self.pc + 2)
    }

    fn operand_16(&self) -> Word {
        <Byte as Into<Word>>::into(self.operand_1())
            | <Byte as Into<Word>>::into(self.operand_2()) << 8
    }
}

fn to_assembly_code(operation: Byte, opcode: Opcode, cpu: &CPU) -> String {
    let name = opcode.mnemonic.to_string();
    let prefix = if UNDOCUMENTED_OPCODES.contains(&operation.u8()) {
        "*"
    } else {
        " "
    };

    let operand = match (opcode.mnemonic, opcode.addressing_mode) {
        (Mnemonic::JMP, AddressingMode::Absolute) | (Mnemonic::JSR, AddressingMode::Absolute) => {
            format!("${:4X}", decode_address(opcode.addressing_mode, &cpu))
        }
        (Mnemonic::LSR, AddressingMode::Accumulator)
        | (Mnemonic::ASL, AddressingMode::Accumulator)
        | (Mnemonic::ROR, AddressingMode::Accumulator)
        | (Mnemonic::ROL, AddressingMode::Accumulator) => "A".to_string(),

        (_, addressing_mode) => match addressing_mode {
            AddressingMode::Implicit | AddressingMode::Accumulator => " ".to_string(),
            AddressingMode::Immediate => format!("#${:02X}", cpu.operand_1()),
            AddressingMode::ZeroPage => format!(
                "${:02X} = {:02X}",
                cpu.operand_1(),
                cpu.bus.read(decode_address(addressing_mode, &cpu))
            ),
            AddressingMode::ZeroPageX => format!(
                "${:02X},X @ {:02X} = {:02X}",
                cpu.operand_1(),
                cpu.operand_1() + cpu.x,
                cpu.bus.read(decode_address(addressing_mode, &cpu))
            ),
            AddressingMode::ZeroPageY => format!(
                "${:02X},Y @ {:02X} = {:02X}",
                cpu.operand_1(),
                cpu.operand_1() + cpu.y,
                cpu.bus.read(decode_address(addressing_mode, &cpu))
            ),
            AddressingMode::Absolute => format!(
                "${:04X} = {:02X}",
                cpu.operand_16(),
                cpu.bus.read(decode_address(addressing_mode, &cpu))
            ),
            AddressingMode::AbsoluteX { .. } => format!(
                "${:04X},X @ {:04X} = {:02X}",
                cpu.operand_16(),
                cpu.operand_16() + cpu.x,
                cpu.bus.read(decode_address(addressing_mode, &cpu))
            ),
            AddressingMode::AbsoluteY { .. } => format!(
                "${:04X},Y @ {:04X} = {:02X}",
                cpu.operand_16(),
                cpu.operand_16() + cpu.y,
                cpu.bus.read(decode_address(addressing_mode, &cpu))
            ),
            AddressingMode::Relative => {
                let pc = <Word as Into<i16>>::into(cpu.pc);
                let offset = <Byte as Into<i8>>::into(cpu.operand_1());
                format!("${:04X}", pc.wrapping_add(2).wrapping_add(offset as i16))
            }
            AddressingMode::Indirect => format!(
                "(${:04X}) = {:04X}",
                cpu.operand_16(),
                cpu.bus.read_on_indirect(cpu.operand_16())
            ),
            AddressingMode::IndexedIndirect => {
                let operand_x = cpu.operand_1() + cpu.x;
                let addr = cpu.bus.read_on_indirect(operand_x.into());
                format!(
                    "(${:02X},X) @ {:02X} = {:04X} = {:02X}",
                    cpu.operand_1(),
                    operand_x,
                    addr,
                    cpu.bus.read(addr)
                )
            }
            AddressingMode::IndirectIndexed => {
                let addr = cpu.bus.read_on_indirect(cpu.operand_1().into());
                format!(
                    "(${:02X}),Y = {:04X} @ {:04X} = {:02X}",
                    cpu.operand_1(),
                    addr,
                    addr + cpu.y,
                    cpu.bus.read(addr + cpu.y)
                )
            }
        },
    };
    format!("{}{} {:<28}", prefix, name, operand)
}

fn decode_address(addressing_mode: AddressingMode, cpu: &CPU) -> Word {
    match addressing_mode {
        AddressingMode::Implicit => 0x00u16.into(),
        AddressingMode::Immediate => cpu.pc,
        AddressingMode::ZeroPage => cpu.operand_1().into(),
        AddressingMode::ZeroPageX => <Byte as Into<Word>>::into(cpu.operand_1() + cpu.x) & 0xFF,
        AddressingMode::ZeroPageY => <Byte as Into<Word>>::into(cpu.operand_1() + cpu.y) & 0xFF,
        AddressingMode::Absolute => cpu.operand_16(),
        AddressingMode::AbsoluteX { .. } => cpu.operand_16() + cpu.x,
        AddressingMode::AbsoluteY { .. } => cpu.operand_16() + cpu.y,
        AddressingMode::Relative => cpu.pc,
        AddressingMode::Indirect => cpu.bus.read_on_indirect(cpu.operand_16()),
        AddressingMode::IndexedIndirect => {
            cpu.bus.read_on_indirect((cpu.operand_16() + cpu.x) & 0xFF)
        }
        AddressingMode::IndirectIndexed => cpu.bus.read_on_indirect(cpu.operand_16()) + cpu.y,
        _ => 0x00u16.into(),
    }
}

impl fmt::Display for Mnemonic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

const UNDOCUMENTED_OPCODES: [u8; 80] = [
    0xEB, 0x04, 0x44, 0x64, 0x0C, 0x14, 0x34, 0x54, 0x74, 0xD4, 0xF4, 0x1A, 0x3A, 0x5A, 0x7A, 0xDA,
    0xFA, 0x1C, 0x3C, 0x5C, 0x7C, 0xDC, 0xFC, 0x80, 0x82, 0x89, 0xC2, 0xE2, 0xA3, 0xA7, 0xAF, 0xB3,
    0xB7, 0xBF, 0x83, 0x87, 0x8F, 0x97, 0xC3, 0xC7, 0xCF, 0xD3, 0xD7, 0xDB, 0xDF, 0xE3, 0xE7, 0xEF,
    0xF3, 0xF7, 0xFB, 0xFF, 0x03, 0x07, 0x0F, 0x13, 0x17, 0x1B, 0x1F, 0x23, 0x27, 0x2F, 0x33, 0x37,
    0x3B, 0x3F, 0x43, 0x47, 0x4F, 0x53, 0x57, 0x5B, 0x5F, 0x63, 0x67, 0x6F, 0x73, 0x77, 0x7B, 0x7F,
];

impl AddressingMode {
    fn instruction_length(&self) -> u8 {
        match self {
            Self::Immediate
            | Self::ZeroPage
            | Self::ZeroPageX
            | Self::ZeroPageY
            | Self::Relative
            | Self::IndirectIndexed
            | Self::IndexedIndirect => 2,
            Self::Indirect | Self::Absolute | Self::AbsoluteX { .. } | Self::AbsoluteY { .. } => 3,
            _ => 1,
        }
    }
}

impl fmt::UpperHex for Byte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = <Self as Into<u8>>::into(*self);
        fmt::UpperHex::fmt(&v, f)
    }
}

impl fmt::UpperHex for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = <Self as Into<u16>>::into(*self);
        fmt::UpperHex::fmt(&v, f)
    }
}

impl dyn Memory {
    pub(super) fn read_on_indirect(&self, operand: Word) -> Word {
        let low = Word::from(self.read(operand));
        // Reproduce 6502 bug; http://nesdev.com/6502bugs.txt
        let addr = operand & 0xFF00 | ((operand + 1) & 0x00FF);
        let high = Word::from(self.read(addr)) << 8;
        low | high
    }
}
