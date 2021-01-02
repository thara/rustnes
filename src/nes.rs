use crate::cpu::{CPUCycle, CPU};
use crate::interrupt::Interrupt;

pub struct NES {
    cpu: CPU,

    interrupt: Interrupt,
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
}
