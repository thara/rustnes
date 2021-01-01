use crate::types::{Byte, Memory, Word};

use super::instructions;

type CPUCycle = u64;

// https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
pub(super) const CPU_STATUS_OPERATED_B: u8 = 0b110000;
pub(super) const CPU_STATUS_INTERRUPTED_B: u8 = 0b100000;

pub struct CPU {
    pub(super) a: Byte,
    pub(super) x: Byte,
    pub(super) y: Byte,
    pub(super) s: Byte,
    pub(super) p: Byte,
    pub(super) pc: Word,

    pub(super) cycles: CPUCycle,

    bus: Box<dyn Memory>,
}

impl CPU {
    fn step(&mut self) {
        let opcode = self.fetch_opcode();
        // let instruction = decode(opcode);
        // instruction.execute(&self);
    }

    fn fetch_opcode(&mut self) -> Byte {
        let opcode = self.read(self.pc);
        self.pc += 1;
        opcode
    }

    pub(super) fn read(&mut self, addr: Word) -> Byte {
        self.cycles += 1;
        self.bus.read(addr)
    }

    pub(super) fn read_word(&mut self, addr: Word) -> Word {
        self.read(addr).word() | (self.read(addr + 1).word() << 8)
    }

    pub(super) fn read_on_indirect(&mut self, operand: Word) -> Word {
        let low = self.read(operand).word();
        // Reproduce 6502 bug; http://nesdev.com/6502bugs.txt
        let addr = operand & 0xFF00 | ((operand + 1) & 0x00FF);
        let high = self.read(addr).word() << 8;
        low | high
    }

    pub(super) fn write(&mut self, addr: Word, value: Byte) {
        self.cycles += 1;
        self.bus.write(addr, value)
    }
}

// stack operation
impl CPU {
    pub(super) fn push_stack(&mut self, value: Byte) {
        self.write(self.s.word() + 0x100, value);
        self.s -= 1;
    }

    pub(super) fn push_stack_word(&mut self, word: Word) {
        self.push_stack((word >> 8).byte());
        self.push_stack((word & 0xFF).byte());
    }

    pub(super) fn pull_stack(&mut self) -> Byte {
        self.s += 1;
        self.read(self.s.word() + 0x100)
    }

    pub(super) fn pull_stack_word(&mut self) -> Word {
        let l = self.pull_stack();
        let h = self.pull_stack();
        h.word() << 8 | l.word()
    }
}

// utils
impl CPU {
    pub(super) fn set_zn(&mut self, value: Byte) {
        // Z
        if value.u8() == 0 {
            self.p |= 0x02;
        } else {
            self.p &= !0x02;
        }

        // N
        if value.bit(7) == 1 {
            self.p |= 0x80;
        } else {
            self.p &= !0x80;
        }
    }
}

pub fn page_crossed<A: Into<i64>, B: Into<i64>>(value: A, from: B) -> bool {
    let a = value.into();
    let b = from.into();
    ((b + a) & 0xFF00) != (b & 0xFF00)
}
