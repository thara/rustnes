use crate::types::{Byte, Memory, Word};

use super::instructions::{decode, execute};
use super::status::CPUStatus;

type CPUCycle = u64;

pub struct CPU {
    pub(super) a: Byte,
    pub(super) x: Byte,
    pub(super) y: Byte,
    pub(super) s: Byte,
    pub(super) p: CPUStatus,
    pub(super) pc: Word,

    pub(super) cycles: CPUCycle,

    bus: Box<dyn Memory>,
}

impl CPU {
    pub fn step(&mut self) {
        let instruction = self.fetch();
        let opcode = decode(instruction);
        execute(self, opcode);
    }

    fn fetch(&mut self) -> Byte {
        let opcode = self.read(self.pc);
        self.pc += 1;
        opcode
    }

    pub(super) fn read(&mut self, addr: impl Into<Word>) -> Byte {
        let addr: Word = addr.into();
        self.cycles += 1;
        self.bus.read(addr)
    }

    pub(super) fn read_word(&mut self, addr: impl Into<Word>) -> Word {
        let addr: Word = addr.into();
        Word::from(self.read(addr)) | (Word::from(self.read(addr + 1)) << 8)
    }

    pub(super) fn read_on_indirect(&mut self, operand: Word) -> Word {
        let low = Word::from(self.read(operand));
        // Reproduce 6502 bug; http://nesdev.com/6502bugs.txt
        let addr = operand & 0xFF00 | ((operand + 1) & 0x00FF);
        let high = Word::from(self.read(addr)) << 8;
        low | high
    }

    pub(super) fn write(&mut self, addr: Word, value: Byte) {
        self.cycles += 1;
        self.bus.write(addr, value)
    }
}

// stack operation
impl CPU {
    pub(super) fn push_stack(&mut self, value: impl Into<Byte>) {
        let value = value.into();
        self.write(Word::from(self.s) + 0x100, value);
        self.s -= 1;
    }

    pub(super) fn push_stack_word(&mut self, word: Word) {
        self.push_stack((word >> 8).byte());
        self.push_stack((word & 0xFF).byte());
    }

    pub(super) fn pull_stack(&mut self) -> Byte {
        self.s += 1;
        self.read(Word::from(self.s) + 0x100)
    }

    pub(super) fn pull_stack_word(&mut self) -> Word {
        let l: Word = self.pull_stack().into();
        let h: Word = self.pull_stack().into();
        h << 8 | l
    }
}

// utils
impl CPU {
    pub(super) fn set_zn(&mut self, value: Byte) {
        self.p.update(CPUStatus::Z, value.u8() == 0);
        self.p.update(CPUStatus::N, value.bit(7) == 1);
    }
}

pub fn page_crossed<A: Into<i64>, B: Into<i64>>(value: A, from: B) -> bool {
    let a = value.into();
    let b = from.into();
    ((b + a) & 0xFF00) != (b & 0xFF00)
}
