use std::ops;

pub trait Memory {
    fn read(&self, addr: Word) -> Byte;
    fn write(&mut self, addr: Word, value: Byte);
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Byte(u8);

impl Byte {
    pub fn u8(&self) -> u8 {
        self.0
    }

    pub fn bit(&self, n: u8) -> u8 {
        let Self(v) = self;
        (v >> n) & 1
    }

    pub fn word(&self) -> Word {
        let Self(v) = self;
        Word(*v as u16)
    }

    pub fn is_set(&self, flag: u8) -> bool {
        let Self(v) = self;
        v & flag == flag
    }
}

impl ops::Add for Byte {
    type Output = Self;

    fn add(self, Self(rhs): Byte) -> Byte {
        let Self(v) = self;
        Self(v.wrapping_add(rhs))
    }
}

impl ops::Add<u8> for Byte {
    type Output = Self;

    fn add(self, rhs: u8) -> Byte {
        let Self(v) = self;
        Self(v.wrapping_add(rhs))
    }
}

impl ops::AddAssign<u8> for Byte {
    fn add_assign(&mut self, other: u8) {
        let Self(v) = self;
        *self = Self(v.wrapping_add(other))
    }
}

impl ops::Sub for Byte {
    type Output = Self;

    fn sub(self, Self(rhs): Byte) -> Byte {
        let Self(v) = self;
        Self(v.wrapping_sub(rhs))
    }
}

impl ops::SubAssign<u8> for Byte {
    fn sub_assign(&mut self, other: u8) {
        let Self(v) = self;
        *self = Self(v.wrapping_sub(other))
    }
}

impl ops::BitAnd for Byte {
    type Output = Self;

    fn bitand(self, Self(rhs): Self) -> Self::Output {
        let Self(v) = self;
        Self(v & rhs)
    }
}

impl ops::BitAnd<u8> for Byte {
    type Output = Self;

    fn bitand(self, rhs: u8) -> Self::Output {
        let Self(v) = self;
        Self(v & rhs)
    }
}

impl ops::BitAndAssign for Byte {
    fn bitand_assign(&mut self, Self(rhs): Self) {
        let Self(v) = self;
        *self = Self(*v & rhs)
    }
}

impl ops::BitAndAssign<u8> for Byte {
    fn bitand_assign(&mut self, rhs: u8) {
        let Self(v) = self;
        *self = Self(*v & rhs)
    }
}

impl ops::BitOr for Byte {
    type Output = Self;

    fn bitor(self, Self(rhs): Self) -> Self::Output {
        let Self(v) = self;
        Self(v | rhs)
    }
}

impl ops::BitOr<u8> for Byte {
    type Output = Self;

    fn bitor(self, rhs: u8) -> Self::Output {
        let Self(v) = self;
        Self(v | rhs)
    }
}

impl ops::BitOrAssign for Byte {
    fn bitor_assign(&mut self, Self(rhs): Self) {
        let Self(v) = self;
        *self = Self(*v | rhs)
    }
}

impl ops::BitOrAssign<u8> for Byte {
    fn bitor_assign(&mut self, rhs: u8) {
        let Self(v) = self;
        *self = Self(*v | rhs)
    }
}

impl ops::BitXor for Byte {
    type Output = Self;

    fn bitxor(self, Self(rhs): Self) -> Self::Output {
        let Self(v) = self;
        Self(v ^ rhs)
    }
}

impl ops::BitXor<u8> for Byte {
    type Output = Self;

    fn bitxor(self, rhs: u8) -> Self::Output {
        let Self(v) = self;
        Self(v ^ rhs)
    }
}

impl ops::BitXorAssign for Byte {
    fn bitxor_assign(&mut self, Self(rhs): Self) {
        let Self(v) = self;
        *self = Self(*v ^ rhs)
    }
}

impl ops::Not for Byte {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Word(u16);

pub fn word(value: u16) -> Word {
    Word(value)
}

impl Word {
    pub fn u16(&self) -> u16 {
        self.0
    }

    pub fn byte(&self) -> Byte {
        let Self(v) = self;
        Byte(*v as u8)
    }

    pub fn page_crossed(&self, from: Word) -> bool {
        ((from + self.0) & 0xFF00) != (from & 0xFF00)
    }
}

impl ops::Add for Word {
    type Output = Self;

    fn add(self, Self(rhs): Self) -> Word {
        let Self(v) = self;
        Self(v.wrapping_add(rhs))
    }
}

impl ops::Add<u16> for Word {
    type Output = Self;

    fn add(self, rhs: u16) -> Word {
        let Self(v) = self;
        Self(v.wrapping_add(rhs))
    }
}

impl ops::AddAssign<u16> for Word {
    fn add_assign(&mut self, other: u16) {
        let Self(v) = self;
        *self = Self(v.wrapping_add(other))
    }
}

impl ops::Sub for Word {
    type Output = Self;

    fn sub(self, Self(rhs): Self) -> Word {
        let Self(v) = self;
        Self(v.wrapping_sub(rhs))
    }
}

impl ops::Shr<u16> for Word {
    type Output = Self;

    fn shr(self, rhs: u16) -> Self::Output {
        let Self(v) = self;
        Self(v >> rhs)
    }
}

impl ops::Shl<u16> for Word {
    type Output = Self;

    fn shl(self, rhs: u16) -> Self::Output {
        let Self(v) = self;
        Self(v << rhs)
    }
}

impl ops::BitAnd<u16> for Word {
    type Output = Self;

    fn bitand(self, rhs: u16) -> Self::Output {
        let Self(v) = self;
        Self(v & rhs)
    }
}

impl ops::BitOr for Word {
    type Output = Self;

    fn bitor(self, Self(rhs): Word) -> Self::Output {
        let Self(v) = self;
        Self(v | rhs)
    }
}
