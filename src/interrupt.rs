#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Interrupt(u8);

impl Interrupt {
    pub const RESET: Self = Self(1 << 3);
    pub const NMI: Self = Self(1 << 2);
    pub const IRQ: Self = Self(1 << 1);
    pub const BRK: Self = Self(1 << 0);

    pub const NO_INTERRUPT: Self = Self(0);

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
        self.0 & s.0 == s.0
    }

    pub fn set(&mut self, s: Self) {
        self.0 |= s.0
    }

    pub fn unset(&mut self, s: Self) {
        self.0 &= !s.0
    }

    // pub fn is_interrupted(&self) -> bool {
    //     self.0 != 0
    // }
}
