use delta_radix_hal::{Hal, Display, Keypad, Key};

pub async fn check_menu(hal: &mut impl Hal, key: Key, shifted: bool) -> bool {
    if key == Key::DebugTerminate {
        panic!("debug terminate");
    }
    
    if key == Key::Menu && shifted {
        show_menu(hal).await;
        true
    } else {
        false
    }
}

pub async fn show_menu(hal: &mut impl Hal) {
    let (display, keypad, _) = hal.common_mut();

    display.clear();
    display.set_position(0, 0);
    display.print_string("1) Bootloader");

    loop {
        match keypad.wait_key().await {
            Key::Digit(1) => {
                hal.enter_bootloader().await;
                return
            },

            Key::Menu => return,
            _ => (),
        }
    }
}