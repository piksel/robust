use std::{ops::{Add, AddAssign, Sub, Deref}, fmt::{Display, UpperHex}};

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Addr(pub u16);

impl Addr {

    pub fn from_zero(lsb: u8) -> Self {
        Self(lsb as u16)
    }

    pub fn from_bytes(msb: u8, lsb: u8) -> Self {
        Self(((msb as u16) << 8) | lsb as u16)
    }

    pub fn same_page_as(&self, other: Addr) -> bool {
        self.msb() == other.msb()
    }

    pub fn msb(&self) -> u8 {
        (self.0 >> 8) as u8
    }

    pub fn lsb(&self) -> u8 {
        self.0 as u8
    }
}

impl Into<Addr> for u16 {
    fn into(self) -> Addr {
        Addr(self)
    }
}

impl Into<Addr> for i32 {
    fn into(self) -> Addr {
        Addr(self as u16)
    }
}

impl Into<u16> for Addr {
    fn into(self) -> u16 {
        self.0
    }
}


impl AddAssign<usize> for Addr {
    fn add_assign(&mut self, rhs: usize) {
        self.0 = self.0.wrapping_add(rhs as u16);
    }
}
impl AddAssign<i8> for Addr {
    fn add_assign(&mut self, rhs: i8) {
        self.0 = (self.0 as i32 + (rhs as i32)) as u16;
    }
}

impl <A: Into<Addr>> Add<A> for Addr {
    type Output = Self;

    fn add(self, rhs: A) -> Self::Output {
        Self(self.0.wrapping_add(rhs.into().0))
    }
}

impl Add<i8> for Addr {
    type Output = Self;
    fn add(self, rhs: i8) -> Self {
        Self((self.0 as i32 + (rhs as i32)) as u16)
    }
}


impl <A: Into<Addr>> Sub<A> for Addr {
    type Output = Self;

    fn sub(self, rhs: A) -> Self::Output {
        Self(self.0 - rhs.into().0)
    }
}

impl Display for Addr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("@{:04x}", self.0))
    }
}

impl UpperHex for Addr{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        UpperHex::fmt(&self.0, f)
    }
}

impl Deref for Addr {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq<u16> for Addr {
    fn eq(&self, other: &u16) -> bool {
        self.0 == *other
    }
}

impl PartialOrd<u16> for Addr {
    fn partial_cmp(&self, other: &u16) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}