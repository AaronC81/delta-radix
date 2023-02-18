use alloc::{string::{String, ToString}, vec, vec::Vec};

use crate::FlexInt;

impl FlexInt {
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

    /// Converts this number into a string of hexadecimal digits, treating it as unsigned.
    /// 
    /// ```rust
    /// # use flex_int::FlexInt;
    /// let i = FlexInt::from_int(0x12A4, 32);
    /// assert_eq!(i.to_unsigned_hex_string(), "12A4");
    /// 
    /// let zero = FlexInt::new(16);
    /// assert_eq!(zero.to_unsigned_hex_string(), "0");
    /// ```
    pub fn to_unsigned_hex_string(&self) -> String {
        // Algorithm makes assumptions there will be some bits, so handle the case where there
        // aren't early
        if self.is_zero() {
            return "0".to_string();
        }

        let mut result = "".to_string();

        // Do some twiddling to chop off the "leading" zeroes
        // Remember our bit representation goes from LSB to MSB, so in our representation they're
        // actually trailing - handle this by reversing first
        let bits = self.bits.iter()
            .rev()
            .copied()
            .skip_while(|x| !*x)
            .collect::<Vec<_>>()
            .iter()
            .rev()
            .copied()
            .collect::<Vec<_>>();

        // Iterate through the bits of this number, in chunks of 4, from LSB to MSB
        // (Pad with 0s if we don't have a full 4)
        for chunk in bits.chunks(4) {
            let mut chunk = chunk.to_vec();
            while chunk.len() < 4 {
                chunk.push(false);
            }

            let char = match &chunk[..] {
                [false, false, false, false] => '0',
                [true,  false, false, false] => '1',
                [false, true,  false, false] => '2',
                [true,  true,  false, false] => '3',
                [false, false, true,  false] => '4',
                [true,  false, true,  false] => '5',
                [false, true,  true,  false] => '6',
                [true,  true,  true,  false] => '7',
                [false, false, false, true ] => '8',
                [true,  false, false, true ] => '9',
                [false, true,  false, true ] => 'A',
                [true,  true,  false, true ] => 'B',
                [false, false, true,  true ] => 'C',
                [true,  false, true,  true ] => 'D',
                [false, true,  true,  true ] => 'E',
                [true,  true,  true,  true ] => 'F',

                _ => unreachable!(),
            };
            result.insert(0, char);
        }

        result
    }

    /// Converts this number into a string of decimal digits, treating it as signed.
    /// 
    /// ```rust
    /// # use flex_int::FlexInt;
    /// let (i, _) = FlexInt::from_signed_decimal_string("1234", 16).unwrap();
    /// assert_eq!(i.to_signed_decimal_string(), "1234");
    /// 
    /// let (i, _) = FlexInt::from_signed_decimal_string("-1234", 16).unwrap();
    /// assert_eq!(i.to_signed_decimal_string(), "-1234");
    /// 
    /// let (i, over) = FlexInt::from_signed_decimal_string("254", 8).unwrap();
    /// assert_eq!(i.to_signed_decimal_string(), "-2");
    /// assert!(over);
    /// ```
    pub fn to_signed_decimal_string(&self) -> String {
        self.to_signed_string(Self::to_unsigned_decimal_string)
    }

    /// Converts this number into a string of hexadecimal digits, treating it as signed.
    /// 
    /// ```rust
    /// # use flex_int::FlexInt;
    /// let (i, _) = FlexInt::from_signed_hex_string("12A4", 32).unwrap();
    /// assert_eq!(i.to_signed_hex_string(), "12A4");
    /// 
    /// let (i, _) = FlexInt::from_signed_hex_string("-12A4", 32).unwrap();
    /// assert_eq!(i.to_signed_hex_string(), "-12A4");
    /// ```
    pub fn to_signed_hex_string(&self) -> String {
        self.to_signed_string(Self::to_unsigned_hex_string)
    }

    /// A convenience method which performs a signed number-to-string conversion by using an
    /// existing implementation of an unsigned conversion.
    fn to_signed_string(&self, unsigned_string_fn: impl FnOnce(&Self) -> String) -> String {
        // Make absolute and convert to unsigned string, then just add the sign if needed
        let mut str = unsigned_string_fn(&self.sign_extend(self.size() + 1).abs().unwrap());
        if self.is_negative() {
            str.insert(0, '-');
        }
        str
    }
}