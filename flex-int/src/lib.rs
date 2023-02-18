#![no_std]
extern crate alloc;

mod from_string;
mod to_string;
mod op;
mod binary_op;

use alloc::{vec, vec::Vec};

/// An arbitrary-precision integer, stored as a sequence of bits.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct FlexInt {
    /// The bits composing this integer.
    /// 
    /// The least-significant bit appears first in this list.
    bits: Vec<bool>,
}

impl FlexInt {
    /// Creates a new zeroed integer built of a particular number of bits.
    /// 
    /// ```rust
    /// # use flex_int::FlexInt;
    /// let i = FlexInt::new(4);
    /// assert_eq!(i.bits(), &[false, false, false, false]);
    /// ```
    pub fn new(size: usize) -> Self {
        Self { bits: vec![false; size] }
    }

    /// Creates an integer of a particular number of bits, where only the least-significant bit is
    /// set.
    pub fn new_one(size: usize) -> Self {
        let mut result = Self::new(size);
        *result.bit_mut(0) = true;
        result
    }

    /// Creates a new integer from a slice of bits, with the least-significant first.
    pub fn from_bits(bits: &[bool]) -> Self {
        Self { bits: bits.to_vec() }
    }

    /// Creates an integer by taking the `size` least-significant bits of the given `value`.
    /// 
    /// ```rust
    /// # use flex_int::FlexInt;
    /// let i = FlexInt::from_int(0b1101, 4);
    /// assert_eq!(i.bits(), &[true, false, true, true]);
    /// ```
    pub fn from_int(value: u64, size: usize) -> Self {
        let mut bits = vec![];
        let mut mask = 0b1;
        for _ in 0..size {
            bits.push(value & mask > 0);
            mask <<= 1;
        }
        Self::from_bits(&bits)
    }

    /// Gets the bits of this number, least-significant first.
    pub fn bits(&self) -> &[bool] {
        &self.bits
    }

    /// Gets a mutable reference to the bits of this number, least-significant first.
    pub fn bits_mut(&mut self) -> &mut [bool] {
        &mut self.bits
    }

    /// Gets an individual bit of this number, given the index of a bit (where 0 is the
    /// least-significant)
    /// 
    /// Panics if the bit does not exist in the number.
    pub fn bit(&self, index: usize) -> bool {
        self.bits[index]
    }

    /// Gets a mutable reference to an individual bit of this number, given the index of a bit 
    /// (where 0 is the least-significant)
    /// 
    /// Panics if the bit does not exist in the number.
    pub fn bit_mut(&mut self, index: usize) -> &mut bool {
        &mut self.bits[index]
    }

    /// Gets the number of bits which compose this integer.
    /// 
    /// This also includes bits which are unnecessary, e.g. `0001` will have a size of 4 bits.
    pub fn size(&self) -> usize {
        self.bits.len()
    }

    /// Determines whether this number is storing the largest possible negative value for its number
    /// of bits - that is, the most-significant bit is set, and no others are.
    pub(crate) fn is_largest_possible_negative(&self) -> bool {
        if self.bit(self.size() - 1) {
            for i in 0..(self.size() - 1) {
                if self.bit(i) {
                    return false
                }
            }
            true
        } else {
            false
        }
    }

    /// Whether this number is zero.
    pub fn is_zero(&self) -> bool {
        self.bits.iter().all(|b| !*b)
    }

    /// Whether this number is negative, assuming it is being treated as signed.
    pub fn is_negative(&self) -> bool {
        // Most-significant bit is sign
        self.bit(self.size() - 1)
    }

    /// Whether this number is strictly greater than other, assuming that both numbers are unsigned.
    /// 
    /// Panics unless the two integers are the same size.
    /// 
    /// ```rust
    /// # use flex_int::FlexInt;
    /// let a = FlexInt::from_int(12, 8);
    /// let b = FlexInt::from_int(3, 8);
    /// assert_eq!(a.is_greater_than_unsigned(&b), true);
    /// assert_eq!(b.is_greater_than_unsigned(&a), false);
    /// ```
    pub fn is_greater_than_unsigned(&self, other: &FlexInt) -> bool {
        self.validate_size(other);

        // Iterate over bits from most- to least-significant
        for (self_bit, other_bit) in self.bits().iter().zip(other.bits().iter()).rev() {
            match (*self_bit, *other_bit) {
                (true, false) => return true,
                (false, true) => return false,
                _ => (),
            }
        }

        // They're equal!
        false
    }

    /// Whether this number equals another.
    /// 
    /// Panics unless the two integers are the same size.
    /// 
    /// ```rust
    /// # use flex_int::FlexInt;
    /// let a = FlexInt::from_int(12, 8);
    /// let b = FlexInt::from_int(12, 8);
    /// assert_eq!(a.equals(&b), true);
    /// assert_eq!(b.equals(&a), true);
    /// 
    /// let c = FlexInt::from_int(11, 8);
    /// assert_eq!(a.equals(&c), false);
    /// ```
    pub fn equals(&self, other: &FlexInt) -> bool {
        self.validate_size(other);
        self.bits == other.bits
    }

    /// Validates that the size of this integer matches the size of another, and panics if it does
    /// not.
    fn validate_size(&self, other: &FlexInt) {
        if self.size() != other.size() {
            panic!("cannot perform arithmetic on differently-sized FlexInts")
        }
    }
}
