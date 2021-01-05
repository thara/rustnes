mod addressing_modes;
mod instructions;
mod status;
mod trace;

use crate::types::{Byte, Memory, Word};

use instructions::{decode, execute};
use status::CPUStatus;
pub use trace::Trace;

pub type CPUCycle = u128;

pub struct CPU {
    pub(super) a: Byte,
    pub(super) x: Byte,
    pub(super) y: Byte,
    pub(super) s: Byte,
    pub(super) p: CPUStatus,
    pub(super) pc: Word,

    pub cycles: CPUCycle,

    bus: Box<dyn Memory>,
}

impl CPU {
    pub fn new(cpu_bus: Box<dyn Memory>) -> Self {
        Self {
            a: 0x00.into(),
            x: 0x00.into(),
            y: 0x00.into(),
            s: 0x00.into(),
            p: CPUStatus::from(0),
            pc: 0x00.into(),
            cycles: 0,
            bus: cpu_bus,
        }
    }

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

    pub(super) fn push_stack_word(&mut self, word: impl Into<Word>) {
        let value = word.into();
        self.push_stack((value >> 8).byte());
        self.push_stack((value & 0xFF).byte());
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

// handling interrupt
impl CPU {
    pub fn interrupted(&self) -> bool {
        self.p.is_set(CPUStatus::I)
    }

    pub fn reset(&mut self) {
        self.cycles += 5;
        self.pc = self.read_word(0xFFFC);
        self.p.set(CPUStatus::I);
        self.s -= 3
    }

    // NMI
    pub fn non_markable_interrupt(&mut self) {
        self.cycles += 2;
        self.push_stack_word(self.pc);
        // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
        // http://visual6502.org/wiki/index.php?title=6502_BRK_and_B_bit
        self.push_stack(self.p | CPUStatus::INTERRUPTED_B);
        self.p.set(CPUStatus::I);
        self.pc = self.read_word(0xFFFA)
    }

    // IRQ
    pub fn interrupt_request(&mut self) {
        self.cycles += 2;
        self.push_stack_word(self.pc);
        // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
        // http://visual6502.org/wiki/index.php?title=6502_BRK_and_B_bit
        self.push_stack(self.p | CPUStatus::INTERRUPTED_B);
        self.p.set(CPUStatus::I);
        self.pc = self.read_word(0xFFFE)
    }

    // BRK
    pub fn break_interrupt(&mut self) {
        self.cycles += 2;
        self.pc += 1;
        self.push_stack_word(self.pc);
        // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
        // http://visual6502.org/wiki/index.php?title=6502_BRK_and_B_bit
        self.push_stack(self.p | CPUStatus::INTERRUPTED_B);
        self.p.set(CPUStatus::I);
        self.pc = self.read_word(0xFFFE)
    }
}

fn page_crossed_u16<A: Into<u16>, B: Into<u16>>(value: A, from: B) -> bool {
    let a = value.into();
    let b = from.into();
    (b.wrapping_add(a) & 0xFF00) != (b & 0xFF00)
}

pub fn page_crossed<A: Into<i64>, B: Into<i64>>(value: A, from: B) -> bool {
    let a = value.into();
    let b = from.into();
    (b.wrapping_add(a) & 0xFF00) != (b & 0xFF00)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_cpu() -> CPU {
        let test_mem: Box<dyn Memory> = Box::new([0; 0x10000]);
        CPU::new(test_mem)
    }

    #[test]
    fn fetch() {
        let mut cpu = new_cpu();
        cpu.write(0x9051.into(), 0x90.into());
        cpu.write(0x9052.into(), 0x3F.into());
        cpu.write(0x9053.into(), 0x81.into());
        cpu.write(0x9054.into(), 0x90.into());

        cpu.pc = 0x9052.into();

        let instruction = cpu.fetch();
        assert_eq!(instruction, 0x3F.into());

        let instruction = cpu.fetch();
        assert_eq!(instruction, 0x81.into());
    }

    #[test]
    fn reset() {
        let mut cpu = new_cpu();

        cpu.a = 0xFA.into();
        cpu.x = 0x1F.into();
        cpu.y = 0x59.into();
        cpu.s = 0x37.into();
        cpu.p = CPUStatus::N | CPUStatus::V;
        cpu.pc = 0b01010110_10001101.into();

        cpu.write(0xFFFB.into(), 1.into());
        cpu.write(0xFFFC.into(), 32.into());
        cpu.write(0xFFFD.into(), 127.into());
        cpu.write(0xFFFE.into(), 64.into());

        cpu.reset();

        assert_eq!(cpu.a, 0xFA.into());
        assert_eq!(cpu.x, 0x1F.into());
        assert_eq!(cpu.y, 0x59.into());
        assert_eq!(cpu.s, 0x34.into());
        assert_eq!(cpu.p, CPUStatus::N | CPUStatus::V | CPUStatus::I);
        assert_eq!(cpu.pc, 0b01111111_00100000.into());
    }

    #[test]
    fn stack() {
        let mut cpu = new_cpu();
        cpu.s = 0xFF.into();

        cpu.push_stack(0x83);
        cpu.push_stack(0x14);

        assert_eq!(cpu.pull_stack(), 0x14.into());
        assert_eq!(cpu.pull_stack(), 0x83.into());
    }

    #[test]
    fn stack_word() {
        let mut cpu = new_cpu();
        cpu.s = 0xFF.into();

        cpu.push_stack_word(0x98AF);
        cpu.push_stack_word(0x003A);

        assert_eq!(cpu.pull_stack_word(), 0x003A.into());
        assert_eq!(cpu.pull_stack_word(), 0x98AF.into());
    }
}
