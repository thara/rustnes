use crate::cpu::{CPUCycle, Trace, CPU};
use crate::interrupt::Interrupt;
use crate::memory_map::CPUBus;
use crate::rom::ROM;

pub struct NES {
    cpu: CPU,

    interrupt: Interrupt,
}

impl Default for NES {
    fn default() -> Self {
        let cpu_bus = Box::new([0; 0x10000]);
        Self {
            cpu: CPU::new(cpu_bus),
            interrupt: Interrupt::NO_INTERRUPT,
        }
    }
}

impl NES {
    pub fn cpu_step(&mut self) -> CPUCycle {
        let before = self.cpu.cycles;

        self.handle_interrupt();
        self.cpu.step();

        let after = self.cpu.cycles;
        if before <= after {
            after.wrapping_add(before)
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
        self.interrupt.set(Interrupt::RESET)
    }

    pub fn load(&mut self, rom: ROM) {
        let cpu_bus = Box::new(CPUBus::new(rom.mapper));
        *self = Self {
            cpu: CPU::new(cpu_bus),
            interrupt: Interrupt::NO_INTERRUPT,
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
        self.cpu.pc = 0xC000.into();
        // https://wiki.nesdev.com/w/index.php/CPU_power_up_state#cite_ref-1
        self.cpu.p = 0x24.into();

        loop {
            self.handle_interrupt();

            let trace = Trace::trace(&self.cpu);
            f(&trace);

            self.cpu.step();

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
