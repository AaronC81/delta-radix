use alloc::{vec, vec::Vec};

use crate::FlexInt;

impl FlexInt {
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

        // If we don't handle this case explicitly, the add after the inversion will overflow
        //   0 0 0 0  --invert->  1 1 1 1  --add->  oh no
        if self.is_zero() {
            return Some(self.clone())
        }

        let (num, over) = self.invert().add(&Self::new_one(self.size()), false);
        if over {
            panic!("overflow not expected during negation of {:?}", self)
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

    pub(crate) fn pop_shift_left(&self, amount: usize) -> (Self, Vec<bool>) {
        let mut bits = self.bits.clone();
        let mut popped = vec![];
        for _ in 0..amount {
            bits.insert(0, false);
            popped.push(bits.pop().unwrap());
        }
        (Self::from_bits(&bits), popped)
    }

    pub(crate) fn unchecked_shift_left(&self, amount: usize) -> Self {
        let (n, _) = self.pop_shift_left(amount);
        n
    }
}