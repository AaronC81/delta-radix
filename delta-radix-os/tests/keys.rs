use delta_radix_hal::Key;

macro_rules! keys {
    ($($x:expr),+ $(,)?) => { 
        vec![$(crate::keys::KeySequence::keys(&$x)),+].into_iter().flatten().collect::<Vec<Key>>()
    };
}

pub trait KeySequence {
    fn keys(&self) -> Vec<Key>;
}

impl KeySequence for Key {
    fn keys(&self) -> Vec<Key> {
        vec![*self]
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Shifted(pub Key);
impl KeySequence for Shifted {
    fn keys(&self) -> Vec<Key> {
        vec![
            Key::Shift,
            self.0,
        ]
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct SetFormat(pub usize, pub bool);
impl KeySequence for SetFormat {
    fn keys(&self) -> Vec<Key> {
        let SetFormat(size, signed) = *self;
        let mut keys = vec![];

        // Open the format menu
        keys.push(Key::Menu);

        // Delete the existing size - good enough!
        for _ in 0..10 {
            keys.push(Key::Delete);
        }

        // Write the new size
        keys.extend(size.to_string().chars()
            .map(|c| Key::Digit(char::to_digit(c, 10).unwrap() as u8)));

        // Set signedness
        if signed {
            keys.push(Key::Subtract)
        } else {
            keys.push(Key::Add)
        }

        // Exit
        keys.push(Key::Exe);

        keys
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Number(pub isize);
impl KeySequence for Number {
    fn keys(&self) -> Vec<Key> {
        self.0.to_string().chars()
            .map(|c|
                if c == '-' {
                    Key::Subtract
                } else {
                    Key::Digit(char::to_digit(c, 10).unwrap() as u8)
                })
            .collect()
    }
}
