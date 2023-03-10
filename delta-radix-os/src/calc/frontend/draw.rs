use alloc::{vec::Vec, string::{String, ToString}};
use delta_radix_hal::{Hal, Display, DisplaySpecialCharacter, Glyph};

use crate::calc::backend::parse::ConstantOverflowChecker;

use super::{CalculatorApplication, ApplicationState};


impl<'h, H: Hal> CalculatorApplication<'h, H> {
    pub fn draw_full(&mut self) {
        self.hal.display_mut().clear();
        match self.state {
            ApplicationState::Normal | ApplicationState::OutputBaseSelect | ApplicationState::VariableSet => {
                self.draw_header();
                self.draw_expression();
                self.draw_result();
            }

            ApplicationState::FormatMenu { ref bits_digits, bits_cursor_pos } => {
                let display = self.hal.display_mut();
                let bits_header = "Bits: ";

                display.set_position((bits_header.len() as u8 + bits_cursor_pos as u8) - 1, 0);
                display.print_special(DisplaySpecialCharacter::CursorLeft);
                display.print_special(DisplaySpecialCharacter::CursorRight);

                display.set_position(0, 1);
                display.print_string(bits_header);
                display.print_string(bits_digits);

                display.set_position(0, 2);
                display.print_string("-) Signed  ");
                if self.eval_config.data_type.signed {
                    display.print_string(" <");
                }
                display.set_position(0, 3);
                display.print_string("+) Unsigned");
                if !self.eval_config.data_type.signed {
                    display.print_string(" <");
                }
            }

            ApplicationState::OutputSignedMenu => {
                let display = self.hal.display_mut();

                display.clear();
                display.print_string("Ans signedness ovrd.");

                display.set_position(0, 1);
                display.print_string("DEL) None    ");
                if self.signed_result.is_none() { display.print_string(" <"); }

                display.set_position(0, 2);
                display.print_string("  -) Signed  ");
                if self.signed_result == Some(true) { display.print_string(" <"); }

                display.set_position(0, 3);
                display.print_string("  +) Unsigned");
                if self.signed_result == Some(false) { display.print_string(" <"); }
            }

            ApplicationState::MainMenu => {
                let display = self.hal.display_mut();

                display.clear();
                display.print_string("  1) Variables");
                display.set_position(0, 3);
                display.print_string("DEL) Bootloader");            
            }

            ApplicationState::VariableView { page } => {
                let display = self.hal.display_mut();
                let start = page * 4;

                display.clear();
                for i in start..(start + 4) {
                    display.set_position(0, i - start);
                    display.print_glyph(Glyph::Digit(i));
                    display.print_char('=');

                    let var_glyphs = &self.variables[i as usize];
                    for g in 2..Self::WIDTH {
                        if g + 1 == Self::WIDTH && var_glyphs.len() > Self::WIDTH - 2 {
                            display.print_char('>')
                        } else if g < var_glyphs.len() {
                            display.print_glyph(var_glyphs[g - 2])
                        }
                    }
                }
            }
        }
    }
    
    pub fn draw_header(&mut self) {
        let has_overflow = self.eval_result_has_overflow();

        let disp = self.hal.display_mut();
        disp.set_position(0, 0);

        let name = self.eval_config.data_type.concise_name();
        disp.print_string(&name);
        let mut format_len = name.len();

        if let Some(sign) = self.signed_result {
            disp.print_char('>');
            disp.print_char(if sign { 'S' } else { 'U' });
            format_len += 2;
        }

        disp.print_char(' ');

        let overflow_marker = " OVER";

        let mut ptr = format_len + 1;
        let ptr_target = if has_overflow { Self::WIDTH - overflow_marker.len() } else { Self::WIDTH };
        while ptr < ptr_target {
            if self.input_shifted {
                disp.print_char('^');
            } else {
                disp.print_char('=');
            }
            ptr += 1;
        }

        if has_overflow {
            disp.print_string(overflow_marker);
        }
    }

    pub fn draw_expression(&mut self) {
        self.adjust_scroll();

        // Try to parse and get warning spans
        let (parser, _) = self.parse::<ConstantOverflowChecker>();
        let warning_indices = parser.constant_overflow_spans.iter()
            .flat_map(|s| s.indices().collect::<Vec<_>>())
            .collect::<Vec<_>>();

        self.constant_overflows = !warning_indices.is_empty();
        
        let disp = self.hal.display_mut();

        // Draw expression
        disp.set_position(0, 2);
        let mut chars_written = 0;
        for glyph in self.glyphs.iter().skip(self.scroll_offset).take(Self::WIDTH) {
            disp.print_glyph(*glyph);
            chars_written += 1;
        }
        for _ in chars_written..Self::WIDTH {
            disp.print_char(' ');
        }

        // Draw cursor
        disp.set_position(0, 1);
        for i in self.scroll_offset..(self.scroll_offset + Self::WIDTH) {
            let warn = warning_indices.contains(&i);
            if i + 1 == self.cursor_pos {
                if warn {
                    disp.print_special(DisplaySpecialCharacter::CursorLeftWithWarning)
                } else {
                    disp.print_special(DisplaySpecialCharacter::CursorLeft)
                }
            } else if i == self.cursor_pos {
                if warn {
                    disp.print_special(DisplaySpecialCharacter::CursorRightWithWarning)
                } else {
                    disp.print_special(DisplaySpecialCharacter::CursorRight)
                }
            } else {
                if warn {
                    disp.print_special(DisplaySpecialCharacter::Warning)
                } else {
                    disp.print_char(' ')
                }
            }
        }
    }

    pub fn draw_result(&mut self) {
        let has_overflow = self.eval_result_has_overflow();

        let disp = self.hal.display_mut();

        if self.state == ApplicationState::OutputBaseSelect {
            disp.set_position(0, 3);
            disp.print_string("BASE? ");
            return;
        }

        if self.state == ApplicationState::VariableSet {
            disp.set_position(0, 3);
            disp.print_string("SET? ");
            return;
        }

        // Briefly drop and re-borrow the display so we can call a method on `&self`
        drop(disp);
        let mut str = self.eval_result_to_string()
            .unwrap_or_else(|| str::repeat(" ", Self::WIDTH));
        let disp = self.hal.display_mut();

        // Alright, how long is this result?
        // We can activate ***BIG MODE*** if it's longer than a line
        if str.len() <= Self::WIDTH {
            // Cool, it fits on a line! This should be the average case
            disp.set_position((Self::WIDTH - str.len()) as u8, 3);
            disp.print_string(&str);
        } else if str.len() <= Self::WIDTH * 3 {
            // It fits on three lines... we can leave just the header
            // (Add a marker to the header to say we did this, though)
            disp.set_position(7, 0);
            disp.print_string(" BIG ");
            disp.set_position(0, 1);

            for y in 1..=3 {
                disp.set_position(0, y);
                disp.print_string(&str::repeat(" ", Self::WIDTH));    
            }

            for (i, line) in str.chars().collect::<Vec<_>>().chunks(20).enumerate() {
                disp.set_position(0, i as u8 + 1);
                disp.print_string(&line.iter().collect::<String>());
            }
        } else if !has_overflow && str.len() <= Self::WIDTH * 4 {
            // If there's no overflow, we can occupy the entire screen with the result
            for y in 0..=3 {
                disp.set_position(0, y);
                disp.print_string(&str::repeat(" ", Self::WIDTH));    
            }

            disp.set_position(0, 0);
            for (i, line) in str.chars().collect::<Vec<_>>().chunks(Self::WIDTH).enumerate() {
                disp.set_position(0, i as u8);
                disp.print_string(&line.iter().collect::<String>());
            }
        } else if has_overflow && str.len() <= Self::WIDTH * 4 - 5 {
            // If there's overflow, we can occupy almost the entire screen but must account for an
            // "OVER " marker
            for y in 0..=3 {
                disp.set_position(0, y);
                disp.print_string(&str::repeat(" ", Self::WIDTH));    
            }

            str = ["OVER ".to_string(), str.clone()].join("");
            disp.set_position(0, 0);
            for line in str.chars().collect::<Vec<_>>().chunks(Self::WIDTH) {
                disp.print_string(&line.iter().collect::<String>());
            }
        } else {
            // Nothing will fit!
            let message = "result too wide :(";
            disp.set_position((Self::WIDTH - message.len()) as u8, 3);
            disp.print_string(message);
        }
    }

    fn clear_row(disp: &mut impl Display, y: u8) {
    }
}