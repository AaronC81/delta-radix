use crate::FlexInt;

impl FlexInt {
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

        (
            result,
            if signed {
                if other.is_negative() {
                    started_negative && !ended_negative
                } else {
                    !started_negative && ended_negative
                }
            } else {
                carry
            }
        )
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

            // Another thing to check - check that the resultant signedness matches the combined
            // signedness of the operands
            // (Two of the same sign = pos, two different signs = neg)
            if !result.is_zero() {
                let result_should_be_negative = self.is_negative() ^ other.is_negative();
                if result.is_negative() != result_should_be_negative {
                    overflow = true;
                }
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

        // Special cases - there are problems dividing the largest possible negative by 1 (or -1), 
        // so handle this explicitly
        let other_is_one = 
            if signed {
                other.abs() == Some(Self::new_one(self.size()))
            } else {
                other == &Self::new_one(self.size())
            };
        if other_is_one {
            if other.is_negative() {
                if let Some(neg) = self.negate() {
                    return (neg, false)
                } else {
                    return (Self::new(self.size()), true)
                }
            } else {
                return (self.clone(), false)
            }
        }

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
            return (FlexInt::new(self.size()), true)
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

    /// Subtracts another integer from this one.
    /// 
    /// Convenience method which calls either `subtract_signed` or `subtract_unsigned` based on the
    /// value of `signed`.
    pub fn subtract(&self, other: &FlexInt, signed: bool) -> (FlexInt, bool) {
        if signed {
            self.subtract_signed(other)
        } else {
            self.subtract_unsigned(other)
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
            // Add one and negate that, which is sure to succeed
            let (other_plus_one, _) = other.add(&FlexInt::new_one(self.size()), true);
            let (result, over_1) = self.add(&other_plus_one.negate().unwrap(), true);
            // ...then subtract another one
            let (result, over_2) = result.add(&FlexInt::from_int(1, self.size()), true);

            (result, over_1 || over_2)
        }
    }
}