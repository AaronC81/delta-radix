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
    /// # use delta_radix_os::calc::num::FlexInt;
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
    /// # use delta_radix_os::calc::num::FlexInt;
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

    /// Creates a clone of this number which has been sign-extended to a particular number of bits.
    /// This involves repeating the most-significant bit until the number is the required size.
    /// 
    /// Panics if the new size is less than the current size.
    ///
    /// ```rust
    /// # use delta_radix_os::calc::num::FlexInt;
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
    /// # use delta_radix_os::calc::num::FlexInt;
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

        let (num, over) = self.invert().add(&Self::new_one(self.size()));
        if over {
            panic!("overflow not expected during negation")
        }
        Some(num)
    }

    /// Adds one integer to another, and returns the result, plus a boolean indicating whether
    /// overflow or underflow occurred.
    /// 
    /// Panics unless the two integers are the same size.
    /// 
    /// ```rust
    /// # use delta_radix_os::calc::num::FlexInt;
    /// // Non-overflowing
    /// let a = FlexInt::from_int(0b0110, 4);
    /// let b = FlexInt::from_int(0b0011, 4);
    /// assert_eq!(a.add(&b), (FlexInt::from_int(0b1001, 4), false));
    /// 
    /// // Overflowing
    /// let a = FlexInt::from_int(0b1110, 4);
    /// let b = FlexInt::from_int(0b0011, 4);
    /// assert_eq!(a.add(&b), (FlexInt::from_int(0b0001, 4), true));
    /// ```
    pub fn add(&self, other: &FlexInt) -> (FlexInt, bool) {
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

        (result, carry)
    }

    /// Whether this number is zero.
    pub fn is_zero(&self) -> bool {
        self.bits.iter().all(|b| !*b)
    }

    /// Converts this number into a string of decimal digits, treating it as unsigned.
    /// 
    /// ```rust
    /// # use delta_radix_os::calc::num::FlexInt;
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
}
