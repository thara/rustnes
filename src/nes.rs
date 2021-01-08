use std::cell::RefCell;
use std::rc::Rc;

use crate::cpu::{CPUCycle, Trace, CPU};
use crate::interrupt::Interrupt;
use crate::memory_map::{CPUBus, PPUBus};
use crate::ppu::PPU;
use crate::rom::ROM;

pub struct NES {
    cpu: CPU,
    ppu: Rc<RefCell<PPU>>,

    interrupt: Interrupt,

    cycles: u128,
}

impl Default for NES {
    fn default() -> Self {
        let cpu_bus = Box::new([0; 0x10000]);
        let ppu_bus = Box::new([0; 0x10000]);
        Self {
            cpu: CPU::new(cpu_bus),
            ppu: Rc::new(RefCell::new(PPU::new(ppu_bus))),
            interrupt: Interrupt::NO_INTERRUPT,
            cycles: 0,
        }
    }
}

impl NES {
    pub fn frame(&mut self) {
        let current = self.ppu.borrow_mut().frames;

        loop {
            self.step();
            if current != self.ppu.borrow_mut().frames {
                break;
            }
        }
    }

    fn step(&mut self) {
        let cpu_cycles = self.cpu_step();
        self.cycles = self.cycles.wrapping_add(cpu_cycles);

        let mut ppu = self.ppu.borrow_mut();
        for _ in 0..(cpu_cycles * 3) {
            let line = ppu.current_line();

            if let Some(interrupt) = ppu.step() {
                self.interrupt.set(interrupt);
            }

            if line != ppu.current_line() {
                //TODO render
            }
        }
    }

    fn cpu_step(&mut self) -> CPUCycle {
        let before = self.cpu.cycles;

        self.handle_interrupt();
        self.cpu.step();

        let after = self.cpu.cycles;
        Self::diff_cycles(before, after)
    }

    fn diff_cycles(before: CPUCycle, after: CPUCycle) -> CPUCycle {
        if before <= after {
            after.wrapping_sub(before)
        } else {
            u128::MAX - before + after
        }
    }

    pub fn power_on(&mut self) {
        self.cpu.a = 0x00.into();
        self.cpu.x = 0x00.into();
        self.cpu.y = 0x00.into();
        self.cpu.s = 0xFD.into();
        self.cpu.p = 0x34.into();
    }

    pub fn reset(&mut self) {
        self.interrupt.set(Interrupt::RESET);
        self.ppu.borrow_mut().reset();
    }

    pub fn load(&mut self, rom: ROM) {
        let ppu_bus = Box::new(PPUBus::new(rom.mapper.clone()));
        let ppu = Rc::new(RefCell::new(PPU::new(ppu_bus)));
        let cpu_bus = Box::new(CPUBus::new(rom.mapper.clone(), ppu.clone()));
        *self = Self {
            cpu: CPU::new(cpu_bus),
            ppu,
            interrupt: Interrupt::NO_INTERRUPT,
            cycles: 0,
        }
    }

    fn handle_interrupt(&mut self) {
        let interrupt = self.interrupt.get();
        match interrupt {
            Interrupt::RESET => {
                self.cpu.reset();
                self.interrupt.unset(interrupt)
            }
            Interrupt::NMI => {
                self.cpu.non_markable_interrupt();
                self.interrupt.unset(interrupt)
            }
            Interrupt::IRQ => {
                if self.cpu.interrupted() {
                    self.cpu.interrupt_request();
                    self.interrupt.unset(interrupt)
                }
            }
            Interrupt::BRK => {
                if self.cpu.interrupted() {
                    self.cpu.break_interrupt();
                    self.interrupt.unset(interrupt)
                }
            }
            _ => {}
        }
    }
}

// nestest
impl NES {
    pub fn nestest<F: FnMut(&Trace)>(&mut self, mut f: F) {
        self.cpu.cycles = 7;
        self.cpu.pc = 0xC000u16.into();
        // https://wiki.nesdev.com/w/index.php/CPU_power_up_state#cite_ref-1
        self.cpu.p = 0x24.into();

        loop {
            let before = self.cpu.cycles;

            self.handle_interrupt();

            let trace = Trace::trace(&self.cpu);
            f(&trace);

            self.cpu.step();

            let after = self.cpu.cycles;
            let cpu_cycles = Self::diff_cycles(before, after);

            let mut ppu = self.ppu.borrow_mut();
            for _ in 0..(cpu_cycles * 3) {
                if let Some(interrupt) = ppu.step() {
                    self.interrupt.set(interrupt);
                }
            }

            if 26554 < self.cpu.cycles {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{self, BufRead};

    #[test]
    #[cfg_attr(not(feature = "nestest"), ignore)]
    fn nestest() {
        let rom = ROM::load("nestest.nes").unwrap();

        let mut nes = NES::default();
        nes.load(rom);
        nes.power_on();

        let file = File::open("nestest-cpu.log").unwrap();
        let mut lines = io::BufReader::new(file).lines();

        nes.nestest(|trace| {
            let line = lines.next().unwrap().unwrap();
            assert_eq!(format!("{}", trace), line);
        });
    }
}
