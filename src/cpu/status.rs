use std::ops;

use crate::types::Byte;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CPUStatus {
    raw_value: u8,
}

impl CPUStatus {
    // Negative
    pub const N: Self = Self { raw_value: 1 << 7 };
    // Overflow
    pub const V: Self = Self { raw_value: 1 << 6 };
    pub const R: Self = Self { raw_value: 1 << 5 };
    pub const B: Self = Self { raw_value: 1 << 4 };
    // Decimal mode
    pub const D: Self = Self { raw_value: 1 << 3 };
    // IRQ prevention
    pub const I: Self = Self { raw_value: 1 << 2 };
    // Zero
    pub const Z: Self = Self { raw_value: 1 << 1 };
    // Carry
    pub const C: Self = Self { raw_value: 1 << 0 };

    // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
    pub const OPERATED_B: Self = Self {
        raw_value: 0b110000,
    };
    pub const INTERRUPTED_B: Self = Self {
        raw_value: 0b100000,
    };

    pub fn is_set(&self, s: Self) -> bool {
        self.raw_value & s.raw_value == s.raw_value
    }

    pub fn set(&mut self, s: Self) {
        self.raw_value |= s.raw_value
    }

    pub fn unset(&mut self, s: Self) {
        self.raw_value &= !s.raw_value
    }

    pub fn update(&mut self, s: Self, cond: bool) {
        if cond {
            self.set(s)
        } else {
            self.unset(s)
        }
    }
}

impl From<u8> for CPUStatus {
    fn from(value: u8) -> Self {
        Self { raw_value: value }
    }
}

impl From<Byte> for CPUStatus {
    fn from(value: Byte) -> Self {
        Self {
            raw_value: value.u8(),
        }
    }
}

impl From<CPUStatus> for Byte {
    fn from(value: CPUStatus) -> Self {
        Self::from(value.raw_value)
    }
}

impl ops::BitAnd for CPUStatus {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            raw_value: self.raw_value & rhs.raw_value,
        }
    }
}

impl ops::BitOr for CPUStatus {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            raw_value: self.raw_value | rhs.raw_value,
        }
    }
}

impl ops::BitOr<u8> for CPUStatus {
    type Output = Self;

    fn bitor(self, rhs: u8) -> Self::Output {
        Self {
            raw_value: self.raw_value | rhs,
        }
    }
}

impl ops::Not for CPUStatus {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self {
            raw_value: !self.raw_value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ops() {
        let a = CPUStatus::from(0b10000010);
        let b = !a;
        assert_eq!(b.raw_value, 0b01111101);

        let mut a = CPUStatus::from(0b10000010);
        let b = a | CPUStatus::D;
        assert_eq!(b.raw_value, 0b10001010);
    }

    #[test]
    fn is_set() {
        let a = CPUStatus::from(0b01010101);

        assert!(a.is_set(CPUStatus::from(0b01010101)));
        assert!(a.is_set(CPUStatus::from(0b00000001)));

        assert!(!a.is_set(CPUStatus::from(0b00000010)));
        assert!(!a.is_set(CPUStatus::from(0b10101010)));
        assert!(!a.is_set(CPUStatus::from(0b00000011)));
    }

    #[test]
    fn set() {
        let mut a = CPUStatus::from(0b00100000);

        a.set(CPUStatus::C);
        assert_eq!(a.raw_value, 0b00100001);

        a.set(CPUStatus::N);
        assert_eq!(a.raw_value, 0b10100001);
    }
}
