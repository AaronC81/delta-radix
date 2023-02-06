use std::fmt::Display;

use flex_int::FlexInt;
use num_traits::ops::overflowing::{OverflowingAdd, OverflowingSub};
use rand::{prelude::Distribution, distributions::Standard, seq::SliceRandom};

trait TestCaseInt
where
    Self: Sized + OverflowingAdd + OverflowingSub + Display,
{
    fn bits() -> usize;
    fn is_signed() -> bool;
}

impl TestCaseInt for u32 {
    fn bits() -> usize { 32 }
    fn is_signed() -> bool { false }
}

impl TestCaseInt for u8 {
    fn bits() -> usize { 8 }
    fn is_signed() -> bool { false }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Operation {
    Add,
    Subtract,
}

impl Operation {
    fn random() -> Self {
        *[Operation::Add, Operation::Subtract].choose(&mut rand::thread_rng()).unwrap()
    }

    fn operate_on_ints<I: TestCaseInt>(&self, a: &I, b: &I) -> (I, bool) {
        match self {
            Operation::Add => a.overflowing_add(b),
            Operation::Subtract => a.overflowing_sub(b),
        }
    }

    fn operate_on_flex_ints<I: TestCaseInt>(&self, a: &FlexInt, b: &FlexInt) -> (FlexInt, bool) {
        match self {
            Operation::Add => a.add(&b, I::is_signed()),
            Operation::Subtract => {
                if !I::is_signed() {
                    a.subtract_unsigned(&b)
                } else {
                    todo!()
                }
            },
        }
    }

    fn symbol(&self) -> &'static str {
        match self {
            Operation::Add => "+",
            Operation::Subtract => "-",
        }
    }
}

fn fuzz_once<I: TestCaseInt>() where Standard: Distribution<I> {
    let a = rand::random::<I>();
    let b = rand::random::<I>();

    let op = Operation::random();
    let (expected_result, expected_overflow) = op.operate_on_ints(&a, &b);

    let (a_flex, a_err) = FlexInt::from_decimal_string(&a.to_string(), I::bits());
    assert!(!a_err, "failed to convert {} to {} bits (signedness {})", a, I::bits(), I::is_signed());
    let (b_flex, b_err) = FlexInt::from_decimal_string(&b.to_string(), I::bits());
    assert!(!b_err, "failed to convert {} to {} bits (signedness {})", b, I::bits(), I::is_signed());

    let (flex_result, flex_overflow) = op.operate_on_flex_ints::<I>(&a_flex, &b_flex);

    let desc = format!(
        "expected: {} {} {} = {} (over {}), got: {} {} {} = {} (over {})",
        a, op.symbol(), b, expected_result, expected_overflow,
        // TODO: correct signednesses
        a_flex.to_unsigned_decimal_string(), op.symbol(), b_flex.to_unsigned_decimal_string(),
        flex_result.to_unsigned_decimal_string(), flex_overflow, 
    );
    assert!(flex_result.to_unsigned_decimal_string() == expected_result.to_string(), "{}", &desc);
    assert!(expected_overflow == flex_overflow, "{}", &desc);
}

#[test]
fn fuzz() {
    for _ in 0..10000 {
        fuzz_once::<u32>();
        fuzz_once::<u8>();
    }
}