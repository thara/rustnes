mod cpu;
mod types;

pub struct NES {
    cpu: cpu::CPU,
}

impl NES {
    pub fn cpu_step(&mut self) {
        self.cpu.step();
    }
}
