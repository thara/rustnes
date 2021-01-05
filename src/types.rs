use std::cmp::Ordering;
use std::ops;

#[derive(Copy, Clone)]
pub enum Mirroring {
    Vertical(),
    Horizontal(),
}

pub trait Memory {
    fn read(&self, addr: Word) -> Byte;
    fn write(&mut self, addr: Word, value: Byte);
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Byte(u8);

impl Byte {
    pub fn u8(&self) -> u8 {
        self.0
    }

    pub fn nth(&self, n: u8) -> u8 {
        let Self(v) = self;
        (v >> n) & 1
    }
}

impl From<u8> for Byte {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl Into<u8> for Byte {
    fn into(self) -> u8 {
        self.0
    }
}

impl Into<u16> for Byte {
    fn into(self) -> u16 {
        self.0 as u16
    }
}

impl Into<i8> for Byte {
    fn into(self) -> i8 {
        self.0 as i8
    }
}

impl Into<i16> for Byte {
    fn into(self) -> i16 {
        self.0 as i16
    }
}

impl Into<i64> for Byte {
    fn into(self) -> i64 {
        self.0 as i64
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

impl ops::Sub<u8> for Byte {
    type Output = Self;

    fn sub(self, rhs: u8) -> Byte {
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

impl PartialOrd for Byte {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.cmp(&other.0))
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

impl ops::Shl<u8> for Byte {
    type Output = Self;

    fn shl(self, rhs: u8) -> Self::Output {
        let Self(v) = self;
        Self(v << rhs)
    }
}

impl ops::ShlAssign<u8> for Byte {
    fn shl_assign(&mut self, rhs: u8) {
        let Self(v) = self;
        *self = Self(*v << rhs)
    }
}

impl ops::Shr<u8> for Byte {
    type Output = Self;

    fn shr(self, rhs: u8) -> Self::Output {
        let Self(v) = self;
        Self(v >> rhs)
    }
}

impl ops::ShrAssign<u8> for Byte {
    fn shr_assign(&mut self, rhs: u8) {
        let Self(v) = self;
        *self = Self(*v >> rhs)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Word(u16);

impl From<u8> for Word {
    fn from(value: u8) -> Self {
        Self(value as u16)
    }
}

impl From<u16> for Word {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<Byte> for Word {
    fn from(Byte(value): Byte) -> Self {
        Self(value as u16)
    }
}

impl Into<u16> for Word {
    fn into(self) -> u16 {
        self.0
    }
}

impl Into<i16> for Word {
    fn into(self) -> i16 {
        self.0 as i16
    }
}

impl Into<i32> for Word {
    fn into(self) -> i32 {
        self.0 as i32
    }
}

impl Into<i64> for Word {
    fn into(self) -> i64 {
        self.0 as i64
    }
}

impl Word {
    pub fn byte(&self) -> Byte {
        let Self(v) = self;
        Byte(*v as u8)
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

impl ops::Add<Byte> for Word {
    type Output = Self;

    fn add(self, Byte(rhs): Byte) -> Self {
        let Self(v) = self;
        Self(v.wrapping_add(rhs.into()))
    }
}

impl ops::AddAssign for Word {
    fn add_assign(&mut self, Self(other): Self) {
        let Self(v) = self;
        *self = Self(v.wrapping_add(other))
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

    fn sub(self, Self(rhs): Self) -> Self::Output {
        let Self(v) = self;
        Self(v.wrapping_sub(rhs))
    }
}

impl ops::Sub<u16> for Word {
    type Output = Self;

    fn sub(self, rhs: u16) -> Self::Output {
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

impl ops::BitOr<u16> for Word {
    type Output = Self;

    fn bitor(self, rhs: u16) -> Self::Output {
        let Self(v) = self;
        Self(v | rhs)
    }
}

impl ops::BitXor<u16> for Word {
    type Output = Self;

    fn bitxor(self, rhs: u16) -> Self::Output {
        let Self(v) = self;
        Self(v ^ rhs)
    }
}
