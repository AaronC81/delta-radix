#![no_std]
extern crate alloc;

use alloc::{vec, vec::Vec, string::{ToString, String}};

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

    /// Creates a new unsigned integer of a given size by parsing a string of decimal digits.
    /// 
    /// Only digits are permitted in the string; this will panic if other characters are
    /// encountered.
    /// 
    /// Also returns a boolean indicating whether the digits overflow the given size.
    /// 
    /// ```rust
    /// # use flex_int::FlexInt;
    /// let (i_str, over) = FlexInt::from_decimal_string("1234", 16);
    /// let i_num = FlexInt::from_int(1234, 16);
    /// assert_eq!(i_str, i_num);
    /// assert!(!over);
    /// 
    /// let (i_str, over) = FlexInt::from_decimal_string("260", 8);
    /// let i_num = FlexInt::from_int(260 % 256, 8);
    /// assert_eq!(i_str, i_num);
    /// assert!(over);
    /// ```
    pub fn from_decimal_string(s: &str, size: usize) -> (Self, bool) {
        let mut result = Self::new(size);
        let ten = Self::from_int(10, size);
        let mut overflow = false;

        for c in s.chars() {
            let (r, over) = result.multiply(&ten, false);
            overflow = overflow || over;
            result = r;

            let (r, over) = result.add(&Self::from_int(
                char::to_digit(c, 10).expect("invalid character") as u64,
                size,
            ), false);
            overflow = overflow || over;
            result = r;
        }

        (result, overflow)
    }

    /// Creates a new unsigned integer of a given size by parsing a string of hexadecimal digits.
    /// 
    /// Only hexadecimal are permitted in the string; this will panic if other characters are
    /// encountered.
    /// 
    /// Also returns a boolean indicating whether the digits overflow the given size.
    /// 
    /// ```rust
    /// # use flex_int::FlexInt;
    /// let (i_str, over) = FlexInt::from_hex_string("12A4", 16);
    /// let i_num = FlexInt::from_int(0x12A4, 16);
    /// assert_eq!(i_str, i_num);
    /// assert!(!over);
    /// 
    /// let (i_str, over) = FlexInt::from_hex_string("12A4", 8);
    /// let i_num = FlexInt::from_int(0xA4, 8);
    /// assert_eq!(i_str, i_num);
    /// assert!(over);
    /// ```
    pub fn from_hex_string(s: &str, size: usize) -> (Self, bool) {
        let mut result = Self::new(size);
        let mut overflow = false;

        for c in s.chars() {
            // Shift left by 4 - if any of the bits that this will truncate are 1s, then overflow
            // has occurred
            let (new_result, shifted_bits) = result.pop_shift_left(4);
            result = new_result;
            if shifted_bits.contains(&true) {
                overflow = true;
            }

            // Insert bits of hexadecimal digit
            let bits = match c {
                // LSB -> MSB
                '0'       => [false, false, false, false],
                '1'       => [true,  false, false, false],
                '2'       => [false, true,  false, false],
                '3'       => [true,  true,  false, false],
                '4'       => [false, false, true,  false],
                '5'       => [true,  false, true,  false],
                '6'       => [false, true,  true,  false],
                '7'       => [true,  true,  true,  false],
                '8'       => [false, false, false, true ],
                '9'       => [true,  false, false, true ],
                'A' | 'a' => [false, true,  false, true ],
                'B' | 'b' => [true,  true,  false, true ],
                'C' | 'c' => [false, false, true,  true ],
                'D' | 'd' => [true,  false, true,  true ],
                'E' | 'e' => [false, true,  true,  true ],
                'F' | 'f' => [true,  true,  true,  true ],
                _ => panic!("invalid character"),
            };
            result.bits.splice(0..4, bits);
        }

        (result, overflow)
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

    /// Creates a clone of this number which has been sign-extended to a particular number of bits.
    /// This involves repeating the most-significant bit until the number is the required size.
    /// 
    /// Panics if the new size is less than the current size.
    ///
    /// ```rust
    /// # use flex_int::FlexInt;
    /// let pos = FlexInt::from_int(0b0101, 4);
    /// let pos_ext = pos.sign_extend(8);
    /// assert_eq!(pos_ext.bits(), &[true, false, true, false, false, false, false, false]);
    /// 
    /// let neg = FlexInt::from_int(0b1101, 4);
    /// let neg_ext = neg.sign_extend(8);
    /// assert_eq!(neg_ext.bits(), &[true, false, true, true, true, true, true, true]);
    /// ```
    pub fn sign_extend(&self, new_size: usize) -> Self {
        if new_size < self.bits.len() {
            panic!("cannot sign-extend to a lower size");
        }

        let mut bits = self.bits.clone();
        let sign = *bits.last().unwrap();
        while bits.len() < new_size {
            bits.push(sign);
        }

        Self::from_bits(&bits)
    }

    /// Creates a clone of this number which has been extended with 0s to a particular number of
    /// bits.
    /// 
    /// Panics if the new size is less than the current size.
    /// 
    /// ```rust
    /// # use flex_int::FlexInt;
    /// let pos = FlexInt::from_int(0b0101, 4);
    /// let pos_ext = pos.zero_extend(8);
    /// assert_eq!(pos_ext.bits(), &[true, false, true, false, false, false, false, false]);
    /// 
    /// let neg = FlexInt::from_int(0b1101, 4);
    /// let neg_ext = neg.zero_extend(8);
    /// assert_eq!(neg_ext.bits(), &[true, false, true, true, false, false, false, false]);
    /// ```
    pub fn zero_extend(&self, new_size: usize) -> Self {
        if new_size < self.bits.len() {
            panic!("cannot sign-extend to a lower size");
        }

        let mut bits = self.bits.clone();
        while bits.len() < new_size {
            bits.push(false);
        }

        Self::from_bits(&bits)
    }

    /// Extends a number by calling either [`sign_extend`] or [`zero_extend`], based on whether
    /// `signed` is true or false respectively.
    /// 
    /// Panics if the new size is less than the current size.
    pub fn extend(&self, new_size: usize, signed: bool) -> Self {
        if signed {
            self.sign_extend(new_size)
        } else {
            self.zero_extend(new_size)
        }
    }

    /// Removes the most-significant bits from a number to reduce it to a given size.
    /// 
    /// Returns the shrinked number, and the count of (zero, one) bits removed.
    /// 
    /// Panics if the new size is greater than the current size.
    /// 
    /// ```rust
    /// # use flex_int::FlexInt;
    /// let pos = FlexInt::from_int(0b11100101, 8);
    /// let (pos_ext, zeroes, ones) = pos.shrink(4);
    /// assert_eq!(pos_ext.bits(), &[true, false, true, false]);
    /// assert_eq!(zeroes, 1);
    /// assert_eq!(ones, 3);
    /// ```
    pub fn shrink(&self, new_size: usize) -> (Self, usize, usize) {
        if new_size > self.bits.len() {
            panic!("cannot sign-extend to a greater size");
        }

        let mut bits = self.bits.clone();
        let mut zero_count = 0;
        let mut one_count = 0;

        while bits.len() > new_size {
            if bits.pop().unwrap() {
                one_count += 1;
            } else {
                zero_count += 1;
            }
        }

        (Self::from_bits(&bits), zero_count, one_count)
    }

    /// Returns a clone of this integer with all of its bits flipped.
    pub fn invert(&self) -> FlexInt {
        Self::from_bits(&self.bits.iter().map(|b| !b).collect::<Vec<_>>())
    }

    /// Returns a clone of this integer which has been numerically negated, assuming that it is
    /// being treated as signed.
    /// 
    /// For a two's complement representation, this is done by flipping its bits and then adding
    /// one.
    /// 
    /// If the number is the largest possible negative number, i.e. it has just the most-significant
    /// bit set (0b1000...), then it is not possible to store the inverted number in the same number
    /// of bits. In this case, `None` is returned instead.
    /// 
    /// ```rust
    /// # use flex_int::FlexInt;
    /// // Valid
    /// let a = FlexInt::from_int(0b0110, 4);
    /// assert_eq!(a.negate(), Some(FlexInt::from_int(0b1010, 4)));
    /// 
    /// // Invalid
    /// let a = FlexInt::from_int(0b1000, 4);
    /// assert_eq!(a.negate(), None);
    /// ```
    pub fn negate(&self) -> Option<FlexInt> {
        if self.is_largest_possible_negative() {
            return None
        }

        let (num, over) = self.invert().add(&Self::new_one(self.size()), false);
        if over {
            panic!("overflow not expected during negation")
        }
        Some(num)
    }

    /// Returns a clone of this number which has been numerically negated iff the original number is
    /// negative, assuming that this is being treated as signed.
    /// 
    /// If the number is the largest possible negative number, i.e. it has just the most-significant
    /// bit set (0b1000...), then it is not possible to store the inverted number in the same number
    /// of bits. In this case, `None` is returned instead.
    pub fn abs(&self) -> Option<FlexInt> {
        if self.is_negative() {
            self.negate()
        } else {
            Some(self.clone())
        }
    }

    /// Adds one integer to another, and returns the result, plus a boolean indicating whether
    /// overflow occurred.
    /// 
    /// Panics unless the two integers are the same size.
    /// 
    /// ```rust
    /// # use flex_int::FlexInt;
    /// // Non-overflowing, unsigned
    /// let a = FlexInt::from_int(0b0110, 4);
    /// let b = FlexInt::from_int(0b0011, 4);
    /// assert_eq!(a.add(&b, false), (FlexInt::from_int(0b1001, 4), false));
    /// 
    /// // Overflowing, unsigned
    /// let a = FlexInt::from_int(0b1110, 4);
    /// let b = FlexInt::from_int(0b0011, 4);
    /// assert_eq!(a.add(&b, false), (FlexInt::from_int(0b0001, 4), true));
    /// 
    /// // Non-overflowing, signed
    /// let a = FlexInt::from_int(0b1110, 4);
    /// let b = FlexInt::from_int(0b0011, 4);
    /// assert_eq!(a.add(&b, true), (FlexInt::from_int(0b0001, 4), false));
    /// 
    /// // Overflowing, signed
    /// let a = FlexInt::from_int(0b0110, 4);
    /// let b = FlexInt::from_int(0b0011, 4);
    /// assert_eq!(a.add(&b, true), (FlexInt::from_int(0b1001, 4), true));
    /// ```
    pub fn add(&self, other: &FlexInt, signed: bool) -> (FlexInt, bool) {
        self.validate_size(other);

        let mut result = FlexInt::new(self.size());
        let mut carry = false;
        for i in 0..self.size() {
            let mut set_count = 0u8;
            if self.bit(i) { set_count += 1 };
            if other.bit(i) { set_count += 1 };
            if carry { set_count += 1 }

            let (res, cry) = match set_count {
                0 => (false, false),
                1 => (true, false),
                2 => (false, true),
                3 => (true, true),
                _ => unreachable!()
            };
            *result.bit_mut(i) = res;
            carry = cry;
        }

        let started_negative = self.is_negative();
        let ended_negative = result.is_negative();

        (result, if signed { !started_negative && ended_negative } else { carry })
    }

    /// Multiplies one integer to another, and returns the result, plus a boolean indicating whether
    /// overflow occurred.
    /// 
    /// Multiplication must know whether the numbers being used should be treated as signed, as the
    /// procedure involves extending the numbers, so it must be known whether zero-extension or
    /// sign-extension should be used.
    /// 
    /// Panics unless the two integers are the same size.
    /// 
    /// ```rust
    /// # use flex_int::FlexInt;
    /// // Non-overflowing, unsigned
    /// let a = FlexInt::from_int(11, 8);
    /// let b = FlexInt::from_int(8, 8);
    /// assert_eq!(a.multiply(&b, false), (FlexInt::from_int(11 * 8, 8), false));
    /// 
    /// // Overflowing, unsigned
    /// let a = FlexInt::from_int(50, 8);
    /// let b = FlexInt::from_int(6, 8);
    /// assert_eq!(a.multiply(&b, false), (FlexInt::from_int((50 * 6) % 256, 8), true));
    /// 
    /// // Non-overflowing, signed
    /// let a = FlexInt::from_int(11, 8);
    /// let b = FlexInt::from_int(8, 8).negate().unwrap();
    /// assert_eq!(a.multiply(&b, true), (FlexInt::from_int(11 * 8, 8).negate().unwrap(), false));
    /// 
    /// // Overflowing, signed
    /// let a = FlexInt::from_int(50, 8);
    /// let b = FlexInt::from_int(5, 8).negate().unwrap();
    /// assert_eq!(a.multiply(&b, true), (FlexInt::from_int(6, 8), true));
    /// ```
    pub fn multiply(&self, other: &FlexInt, signed: bool) -> (FlexInt, bool) {
        self.validate_size(other);

        // Extend both numbers to twice their size
        let a_ext = self.extend(self.size() * 2, signed);
        let b_ext = other.extend(self.size() * 2, signed);

        // Perform repeated addition
        let mut overflow = false;
        let mut result_ext = Self::new(self.size() * 2);
        for (i, bit) in b_ext.bits.into_iter().enumerate() {
            if bit {
                let (res, over) = result_ext.add(&a_ext.unchecked_shift_left(i), false);
                result_ext = res;
                overflow = overflow || (over && !signed);
            }
        }

        // Cut back down to size
        let (result, cut_zeroes, cut_ones) = result_ext.shrink(self.size());
        if signed {
            // In a signed number, overflow has only occurred if a mixture of zeroes and ones were
            // cut. If just ones were cut, then we've shrunk a negative number, and just zeroes a
            // positive number
            if cut_zeroes > 0 && cut_ones > 0 {
                overflow = true;
            }

            // If ones were cut but the number is no longer negative, this is also invalid
            // e.g.
            //      \/ cut point
            //   0b1110000 -> 0b10000    = valid, same signed number
            //
            //      \/ cut point
            //   0b1100000 -> 0b00000    = invalid, different number
            if cut_ones > 0 && !result.is_negative() {
                overflow = true;
            }
        } else {
            // In an unsigned number, overflow has occurred if any ones were cut
            if cut_ones > 0 {
                overflow = true;
            }
        }

        (result, overflow)
    }

    /// Divides this integer by another, and returns the result, plus a boolean indicating whether
    /// overflow occurred.
    /// 
    /// Division must know whether the numbers being used should be treated as signed.
    /// 
    /// Panics unless the two integers are the same size.
    /// 
    /// ```rust
    /// # use flex_int::FlexInt;
    /// let a = FlexInt::from_int(12, 8);
    /// let b = FlexInt::from_int(3, 8);
    /// assert_eq!(a.divide(&b, false), (FlexInt::from_int(4, 8), false));
    /// ```
    pub fn divide(&self, other: &FlexInt, signed: bool) -> (FlexInt, bool) {
        self.validate_size(other);

        let a;
        let b;
        let negate_result;
        if signed {
            // Two's complement division is probably really hard, so sign-extend the numbers by one
            // bit, negate the negative ones to be positive, and keep track of whether we need to
            // negate again at the end.
            // The reason we sign-extend is so we don't overflow if negating the lowest possible
            // negative
            a = self.sign_extend(self.size() + 1).abs().expect("unexpected overflow while preparing division");
            b = other.sign_extend(self.size() + 1).abs().expect("unexpected overflow while preparing division");
            negate_result = self.is_negative() ^ other.is_negative();
        } else {
            a = self.clone();
            b = other.clone();
            negate_result = false;
        }

        if other.is_zero() {
            // TODO handle better
            panic!("divide by zero")
        }

        let mut quotient = FlexInt::new(a.size());
        let mut remainder = FlexInt::new(a.size());
        for (i, bit) in a.bits().iter().enumerate().rev() {
            remainder = remainder.unchecked_shift_left(1);
            *remainder.bit_mut(0) = *bit;

            if remainder.is_greater_than_unsigned(&b) || remainder.equals(&b) {
                let (rem, over) = remainder.subtract_unsigned(&b);
                if over {
                    panic!(
                        "unexpected overflow during division when performing {} - {}",
                        remainder.to_unsigned_decimal_string(),
                        b.to_unsigned_decimal_string(),
                    );
                }
                remainder = rem;
                *quotient.bit_mut(i) = true;
            }
        }

        if signed {
            // Get the sign bit and then chop it off
            // (Remember we sign-extended by one earlier)
            let sign = quotient.is_negative();
            (quotient, _, _) = quotient.shrink(quotient.size() - 1);

            // Overflow is whether we've changed the sign
            let overflow = sign != quotient.is_negative();
            
            // We also might need to negate the result - if this fails, report overflow too
            if negate_result {
                if let Some(r) = quotient.negate() {
                    (r, overflow)
                } else {
                    (quotient, true)
                }
            } else {
                (quotient, overflow)
            }
        } else {
            (quotient, false)
        }
    }

    /// Subtracts another unsigned integer from this one. Also returns a boolean indicating if
    /// whether the number became negative, which would not be valid for an unsigned number.
    /// 
    /// Panics unless the two integers are the same size.
    /// 
    /// ```rust
    /// # use flex_int::FlexInt;
    /// let a = FlexInt::from_int(12, 8);
    /// let b = FlexInt::from_int(3, 8);
    /// assert_eq!(a.subtract_unsigned(&b), (FlexInt::from_int(9, 8), false));
    /// ```
    pub fn subtract_unsigned(&self, other: &FlexInt) -> (FlexInt, bool) {
        // Intermediate functions return (difference, borrow)
        fn half_sub(a: bool, b: bool) -> (bool, bool) { 
            (a ^ b, !a && b)
        }

        fn full_sub(a: bool, b: bool, borrow: bool) -> (bool, bool) {
            let (diff, borrow_intermediate_1) = half_sub(a, b);
            let (diff, borrow_intermediate_2) = half_sub(diff, borrow);
            (diff, borrow_intermediate_1 || borrow_intermediate_2)
        }

        self.validate_size(other);

        let mut result = Self::new(self.size());
        let (diff, mut borrow) = half_sub(self.bit(0), other.bit(0));
        *result.bit_mut(0) = diff;

        for (i, (a, b)) in self.bits.iter().zip(other.bits.iter()).enumerate().skip(1) {
            let (diff, b) = full_sub(*a, *b, borrow);
            borrow = b;

            *result.bit_mut(i) = diff;
        }

        (result, borrow)
    }

    /// Subtracts another signed integer from this one. Also returns a boolean indicating if 
    /// overflow occurred.
    /// 
    /// Panics unless the two integers are the same size.
    /// 
    /// ```rust
    /// # use flex_int::FlexInt;
    /// let a = FlexInt::from_int(12, 8);
    /// let b = FlexInt::from_int(3, 8).negate().unwrap();
    /// assert_eq!(a.subtract_signed(&b), (FlexInt::from_int(15, 8), false));
    /// ```
    pub fn subtract_signed(&self, other: &FlexInt) -> (FlexInt, bool) {
        self.validate_size(other);

        // To perform signed subtraction, we'll just negate the other operand and add them!
        // The negation can fail iff the other operand is already the lowest possible negative
        // number (e.g. you can't negate -128 to represent 128 in a signed byte)
        if let Some(negated) = other.negate() {
            self.add(&negated, true)
        } else {
            // TODO: want to test this more carefully

            // Add one and negate that, which is sure to succeed
            let (other_plus_one, _) = self.add(other, true);
            let (result, over_1) = self.add(&other_plus_one, true);
            // ...then subtract another one
            let (result, over_2) = result.add(&FlexInt::from_int(1, self.size()), true);

            (result, over_1 || over_2)
        }
        // TODO how to deal?
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

    /// Converts this number into a string of decimal digits, treating it as unsigned.
    /// 
    /// ```rust
    /// # use flex_int::FlexInt;
    /// let i = FlexInt::from_int(1234, 32);
    /// assert_eq!(i.to_unsigned_decimal_string(), "1234");
    /// 
    /// let zero = FlexInt::new(16);
    /// assert_eq!(zero.to_unsigned_decimal_string(), "0");
    /// ```
    pub fn to_unsigned_decimal_string(&self) -> String {
        // Algorithm translated from: https://stackoverflow.com/a/5247217/2626000
        
        // TODO: allocate smarter! len(bits) * ln(2) / ln(10)
        let mut digits = vec![0u8; self.size()];

        fn add(dst: &mut [u8], src: &[u8]) {
            let mut carry = 0;
            let mut oi = 0;
            for i in 0..src.len() {
                let dividend = src[i] + dst[i] + carry;
                carry = dividend / 10;
                dst[i] = dividend % 10;
                oi += 1;
            }
            while carry > 0 {
                oi += 1;
                let dividend = dst[oi] + carry;
                carry = dividend / 10;
                dst[oi] = dividend % 10;
            }
        }

        for bit in self.bits().iter().rev() {
            let result_clone = digits.clone();
            add(&mut digits, &result_clone);

            if *bit {
                add(&mut digits, &[1]);
            }
        }

        let mut result = "".to_string();
        let mut encountered_nonzero_digit = false;
        for digit in digits.iter().rev() {
            if !encountered_nonzero_digit && *digit != 0 {
                encountered_nonzero_digit = true;
            }

            if encountered_nonzero_digit {
                result.push(char::from_digit(*digit as u32, 10).unwrap());
            }
        }

        if result.is_empty() {
            result = "0".to_string()
        }
        result
    }

    /// Validates that the size of this integer matches the size of another, and panics if it does
    /// not.
    fn validate_size(&self, other: &FlexInt) {
        if self.size() != other.size() {
            panic!("cannot perform arithmetic on differently-sized FlexInts")
        }
    }

    /// Determines whether this number is storing the largest possible negative value for its number
    /// of bits - that is, the most-significant bit is set, and no others are.
    fn is_largest_possible_negative(&self) -> bool {
        if self.bit(self.size() - 1) {
            for i in 0..(self.size() - 1) {
                if self.bit(i) {
                    return false
                }
            }
            return true
        } else {
            return false
        }
    }

    fn pop_shift_left(&self, amount: usize) -> (Self, Vec<bool>) {
        let mut bits = self.bits.clone();
        let mut popped = vec![];
        for _ in 0..amount {
            bits.insert(0, false);
            popped.push(bits.pop().unwrap());
        }
        (Self::from_bits(&bits), popped)
    }

    fn unchecked_shift_left(&self, amount: usize) -> Self {
        let (n, _) = self.pop_shift_left(amount);
        n
    }
}
