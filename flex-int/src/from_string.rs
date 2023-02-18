use alloc::string::ToString;

use crate::FlexInt;

impl FlexInt {
        /// Creates a new unsigned integer of a given size by parsing a string of decimal digits.
    /// 
    /// Only digits are permitted in the string; returns `None` if any other character is
    /// encountered.
    /// 
    /// Also returns a boolean indicating whether the digits overflow the given size.
    /// 
    /// ```rust
    /// # use flex_int::FlexInt;
    /// let (i_str, over) = FlexInt::from_unsigned_decimal_string("1234", 16).unwrap();
    /// let i_num = FlexInt::from_int(1234, 16);
    /// assert_eq!(i_str, i_num);
    /// assert!(!over);
    /// 
    /// let (i_str, over) = FlexInt::from_unsigned_decimal_string("260", 8).unwrap();
    /// let i_num = FlexInt::from_int(260 % 256, 8);
    /// assert_eq!(i_str, i_num);
    /// assert!(over);
    /// ```
    pub fn from_unsigned_decimal_string(s: &str, size: usize) -> Option<(Self, bool)> {
        let mut result = Self::new(size);
        let ten = Self::from_int(10, size);
        let mut overflow = false;

        for c in s.chars() {
            let (r, over) = result.multiply(&ten, false);
            overflow = overflow || over;
            result = r;

            let Some(d) = char::to_digit(c, 10) else {
                return None
            };

            let (r, over) = result.add(&Self::from_int(d as u64, size), false);
            overflow = overflow || over;
            result = r;
        }

        Some((result, overflow))
    }

    /// Creates a new unsigned integer of a given size by parsing a string of decimal digits.
    /// 
    /// The first character may optionally be a sign, then only digits are permitted in the string.
    /// This will return None if other characters are encountered.
    /// 
    /// Also returns a boolean indicating whether the digits overflow the given size.
    /// 
    /// ```rust
    /// # use flex_int::FlexInt;
    /// // Positive conversion
    /// let (i_str, over) = FlexInt::from_signed_decimal_string("1234", 16).unwrap();
    /// let i_num = FlexInt::from_int(1234, 16);
    /// assert_eq!(i_str, i_num);
    /// assert!(!over);
    ///
    /// // Negative conversion
    /// let (i_str, over) = FlexInt::from_signed_decimal_string("-1234", 16).unwrap();
    /// let i_num = FlexInt::from_int(1234, 16).negate().unwrap();
    /// assert_eq!(i_str, i_num);
    /// assert!(!over);
    /// 
    /// // Largest possible negative conversion
    /// let (i_str, over) = FlexInt::from_signed_decimal_string("-128", 8).unwrap();
    /// let i_num = FlexInt::from_bits(&[false, false, false, false, false, false, false, true]);
    /// assert_eq!(i_str, i_num);
    /// assert!(!over);
    /// 
    /// // Overflowing conversion
    /// let (i_str, over) = FlexInt::from_signed_decimal_string("-129", 8).unwrap();
    /// assert!(over);
    /// ```
    pub fn from_signed_decimal_string(s: &str, size: usize) -> Option<(Self, bool)> {
        Self::from_signed_string(s, size, Self::from_unsigned_decimal_string)
    }

    /// Creates a new unsigned integer of a given size by parsing a string of hexadecimal digits.
    /// 
    /// Only hexadecimal are permitted in the string; this will return None if other characters are
    /// encountered.
    /// 
    /// Also returns a boolean indicating whether the digits overflow the given size.
    /// 
    /// ```rust
    /// # use flex_int::FlexInt;
    /// let (i_str, over) = FlexInt::from_unsigned_hex_string("12A4", 16).unwrap();
    /// let i_num = FlexInt::from_int(0x12A4, 16);
    /// assert_eq!(i_str, i_num);
    /// assert!(!over);
    /// 
    /// let (i_str, over) = FlexInt::from_unsigned_hex_string("12A4", 8).unwrap();
    /// let i_num = FlexInt::from_int(0xA4, 8);
    /// assert_eq!(i_str, i_num);
    /// assert!(over);
    /// ```
    pub fn from_unsigned_hex_string(s: &str, size: usize) -> Option<(Self, bool)> {
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
                _ => return None,
            };
            result.bits.splice(0..4, bits);
        }

        Some((result, overflow))
    }

    /// Creates a new signed integer of a given size by parsing a string of hexadecimal digits.
    /// 
    /// The first character may optionally be a sign, then only hexadecimal digits are permitted in
    /// the string. This will return None if other characters are encountered.
    /// 
    /// Also returns a boolean indicating whether the digits overflow the given size.
    /// 
    /// ```rust
    /// # use flex_int::FlexInt;
    /// let (i_str, over) = FlexInt::from_signed_hex_string("-12A4", 16).unwrap();
    /// let i_num = FlexInt::from_int(0x12A4, 16).negate().unwrap();
    /// assert_eq!(i_str, i_num);
    /// assert!(!over);
    /// ```
    pub fn from_signed_hex_string(s: &str, size: usize) -> Option<(Self, bool)> {
        Self::from_signed_string(s, size, Self::from_unsigned_hex_string)
    }

    /// A convenience methods which performs a signed string-to-number conversion by using an
    /// existing implementation of an unsigned conversion.
    fn from_signed_string(s: &str, size: usize, unsigned_string_fn: impl FnOnce(&str, usize) -> Option<(Self, bool)>) -> Option<(Self, bool)> {
        let mut s = s.to_string();
        
        // Handle sign
        let mut is_negative = false;
        let first_char = s.chars().next();
        match first_char {
            Some('+') => {
                s.remove(0);
            }
            Some('-') => {
                is_negative = true;
                s.remove(0);
            }
            _ => (),
        }

        // Parse as an unsigned number
        let (num, mut over) = unsigned_string_fn(&s, size)?;

        // If the most-significant bit is set, there's already been overflow - unless this number
        // is going to be negated to the largest possible negative number
        // (Remember we can represent one more negative number than positive number)
        if num.is_negative() && !(num.is_largest_possible_negative() && is_negative) {
            over = true;
        }

        // Try to negate if the number is supposed to be negative, overflow if this fails
        if is_negative {
            if let Some(negated) = num.negate() {
                Some((negated, over))
            } else {
                // Negation might fail if we had the largest possible negative before - override
                // this
                let over = !num.is_largest_possible_negative();
                Some((num, over))
            }
        } else {
            Some((num, over))
        }
    }
}