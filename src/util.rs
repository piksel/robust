

// Ported from rust nightly
// Source: https://github.com/clarfonthey/rust/blob/cc15047d505c2cb6bba7475b18450f9785a78d7e/library/core/src/num/uint_macros.rs#L1403
pub const fn carrying_add(lhs: u8, rhs: u8, carry: bool) -> (u8, bool) {
    let (a, b) = lhs.overflowing_add(rhs);
    let (c, d) = a.overflowing_add(carry as u8);
    (c, b | d)
}