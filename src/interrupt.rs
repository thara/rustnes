#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Interrupt {
    raw_value: u8,
}

impl Interrupt {
    pub const RESET: Self = Self { raw_value: 1 << 3 };
    pub const NMI: Self = Self { raw_value: 1 << 2 };
    pub const IRQ: Self = Self { raw_value: 1 << 1 };
    pub const BRK: Self = Self { raw_value: 1 << 0 };

    const NO_INTERRUPT: Self = Self { raw_value: 0 };

    pub fn get(&self) -> Self {
        if self.is_set(Self::RESET) {
            Self::RESET
        } else if self.is_set(Self::NMI) {
            Self::NMI
        } else if self.is_set(Self::IRQ) {
            Self::IRQ
        } else if self.is_set(Self::BRK) {
            Self::BRK
        } else {
            Self::NO_INTERRUPT
        }
    }

    pub fn is_set(&self, s: Self) -> bool {
        self.raw_value & s.raw_value == s.raw_value
    }

    pub fn set(&mut self, s: Self) {
        self.raw_value |= s.raw_value
    }

    pub fn unset(&mut self, s: Self) {
        self.raw_value &= !s.raw_value
    }

    // pub fn is_interrupted(&self) -> bool {
    //     self.raw_value != 0
    // }
}
