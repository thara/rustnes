use crate::cpu::{CPUCycle, CPU};
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
            _ => self.cpu.step(),
        }

        let after = self.cpu.cycles;
        if before <= after {
            after.wrapping_add(before)
        } else {
            u128::MAX - before + after
        }
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
}
