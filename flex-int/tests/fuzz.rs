use std::fmt::Display;

use flex_int::FlexInt;
use num_traits::ops::overflowing::{OverflowingAdd, OverflowingSub};
use rand::{prelude::Distribution, distributions::Standard};

trait TestCaseInt
where
    Self: Sized + OverflowingAdd + OverflowingSub + Display,
    Standard: Distribution<Self>,
{
    fn bits() -> usize;
    fn is_signed() -> bool;
}

impl TestCaseInt for u32 {
    fn bits() -> usize { 32 }
    fn is_signed() -> bool { false }
}

fn fuzz_once<I: TestCaseInt>() where Standard: Distribution<I> {
    let a = rand::random::<I>();
    let b = rand::random::<I>();
    // TODO: more operations
    let (expected_result, expected_overflow) = a.overflowing_add(&b);

    let (a_flex, a_err) = FlexInt::from_decimal_string(&a.to_string(), I::bits());
    assert!(!a_err, "failed to convert {} to {} bits (signedness {})", a, I::bits(), I::is_signed());
    let (b_flex, b_err) = FlexInt::from_decimal_string(&b.to_string(), I::bits());
    assert!(!b_err, "failed to convert {} to {} bits (signedness {})", b, I::bits(), I::is_signed());

    let (flex_result, flex_overflow) = a_flex.add(&b_flex, I::is_signed());

    let desc = format!(
        "expected: {} + {} = {} (over {}), got: {} + {} = {} (over {})",
        a, b, expected_result, expected_overflow,
        // TODO: correct signednesses
        a_flex.to_unsigned_decimal_string(), b_flex.to_unsigned_decimal_string(),
        flex_result.to_unsigned_decimal_string(), flex_overflow, 
    );
    assert!(flex_result.to_unsigned_decimal_string() == expected_result.to_string(), "{}", &desc);
    assert!(expected_overflow == flex_overflow, "{}", &desc);
}

#[test]
fn fuzz() {
    for _ in 0..10000 {
        fuzz_once::<u32>()
    }
}