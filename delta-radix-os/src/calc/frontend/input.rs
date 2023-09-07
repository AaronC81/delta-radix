use alloc::string::ToString;
use delta_radix_hal::{Hal, Key, Glyph};

use super::{CalculatorApplication, ApplicationState, Base};

impl<'h, H: Hal> CalculatorApplication<'h, H> {
    pub async fn process_input_and_redraw(&mut self, key: Key) {
        if key == Key::DebugTerminate {
            panic!("debug terminate");
        }

        match self.state {
            ApplicationState::Normal =>
                if self.input_shifted {
                    match key {
                        Key::Shift => {
                            self.input_shifted = false;
                            self.draw_header();
                        }
                        Key::Delete => {
                            self.clear_all(true);
                            self.draw_full();
                        }

                        Key::Digit(0) => {
                            self.input_shifted = false;
                            
                            // Insert two characters and move between them
                            self.glyphs.insert(self.cursor_pos, Glyph::LeftParen);
                            self.cursor_pos += 1;
                            self.glyphs.insert(self.cursor_pos, Glyph::RightParen);
                            self.draw_expression();
                            self.clear_evaluation(true);
                        }

                        Key::Right => {
                            self.input_shifted = false;
                            self.insert_and_redraw(Glyph::Align);
                        }

                        Key::Variable => {
                            self.input_shifted = false;
                            if let Some(Ok(_)) = self.eval_result {
                                self.state = ApplicationState::VariableSet;
                                self.draw_header();
                                self.draw_result();
                            } else {
                                self.draw_full();
                            }
                        }

                        Key::FormatSelect => {
                            self.input_shifted = false;
                            self.state = ApplicationState::OutputSignedMenu;
                            self.draw_full();
                        }

                        Key::Menu => {
                            self.input_shifted = false;
                            self.state = ApplicationState::MainMenu;
                            self.draw_full();
                        }

                        _ => (),
                    }
                } else {
                    match key {
                        Key::Digit(d) => self.insert_and_redraw(Glyph::Digit(d)),
                        Key::HexBase => self.insert_and_redraw(Glyph::HexBase),
                        Key::BinaryBase => self.insert_and_redraw(Glyph::BinaryBase),
            
                        Key::Add => self.insert_and_redraw(Glyph::Add),
                        Key::Subtract => self.insert_and_redraw(Glyph::Subtract),
                        Key::Multiply => self.insert_and_redraw(Glyph::Multiply),
                        Key::Divide => self.insert_and_redraw(Glyph::Divide),

                        // TODO: nicer insertion mechanism, and treat as one token?
                        Key::Variable => self.insert_and_redraw(Glyph::Variable),
            
                        Key::Left => {
                            if self.cursor_pos > 0 {
                                self.cursor_pos -= 1;
                                self.draw_expression();
                                self.clear_evaluation(true);
                            }
                        },
                        Key::Right => {
                            if self.cursor_pos < self.glyphs.len() {
                                self.cursor_pos += 1;
                                self.draw_expression();
                                self.clear_evaluation(true);
                            }
                        }
                        Key::Delete => {
                            if self.cursor_pos > 0 {
                                self.cursor_pos -= 1;
                                self.glyphs.remove(self.cursor_pos);
                                self.draw_expression();
                                self.clear_evaluation(true);
                            }
                        },
                        Key::Exe => {
                            self.evaluate();
                            self.draw_header();
                            self.draw_result();
                        }

                        Key::FormatSelect => {
                            self.state = ApplicationState::OutputBaseSelect;
                            self.draw_full(); // May change "big mode" state, so redraw everything
                        }
            
                        Key::Shift => {
                            self.input_shifted = true;
                            self.draw_header();
                        }

                        Key::Sleep => {
                            // Do not redraw - the HAL is expected to deal with this
                            self.clear_all(false);
                        }

                        Key::Menu => {
                            let bits_digits = self.eval_config.data_type.bits.to_string();
                            self.state = ApplicationState::FormatMenu {
                                bits_cursor_pos: bits_digits.len(),
                                bits_digits,
                            };
                            self.draw_full();
                        }
                        
                        Key::DebugTerminate => (),
                    }
                },
            
            ApplicationState::OutputBaseSelect => match key {
                Key::HexBase => self.set_output_format_and_redraw(Base::Hexadecimal),
                Key::BinaryBase => self.set_output_format_and_redraw(Base::Binary),
                Key::FormatSelect => self.set_output_format_and_redraw(Base::Decimal),

                _ => (),
            }

            ApplicationState::VariableSet => match key {
                Key::Digit(d) => {
                    self.variables[d as usize] = Glyph::from_string(&self.eval_result_to_string().unwrap()).unwrap();

                    self.state = ApplicationState::Normal;
                    self.draw_full();
                },

                Key::Exe | Key::Variable => {
                    self.state = ApplicationState::Normal;
                    self.draw_full();
                }

                _ => (),
            }

            ApplicationState::FormatMenu { ref mut bits_digits, ref mut bits_cursor_pos } => match key {
                Key::Digit(d) => {
                    bits_digits.push(char::from_digit(d as u32, 10).unwrap());
                    *bits_cursor_pos += 1;
                    self.draw_full();
                }

                Key::Delete => {
                    if *bits_cursor_pos > 0 {
                        bits_digits.remove(*bits_cursor_pos - 1);
                        *bits_cursor_pos -= 1;
                        self.draw_full();
                    }
                }
                Key::Left => {
                    if *bits_cursor_pos > 0 {
                        *bits_cursor_pos -= 1;
                        self.draw_full();
                    }
                }
                Key::Right => {
                    if *bits_cursor_pos < bits_digits.len() {
                        *bits_cursor_pos += 1;
                        self.draw_full();
                    }
                }

                Key::Add => {
                    self.eval_config.data_type.signed = false;
                    self.draw_full();
                }
                Key::Subtract => {
                    self.eval_config.data_type.signed = true;
                    self.draw_full();
                }

                Key::FormatSelect | Key::Menu | Key::Exe => {
                    // Apply bits evaluation settings
                    if let Ok(mut bits) = bits_digits.parse() {
                        // Minimum supported number of bits
                        if bits < 3 {
                            bits = 3;
                        }

                        self.eval_config.data_type.bits = bits;
                    }

                    self.state = ApplicationState::Normal;
                    self.clear_evaluation(true);
                    self.draw_full();
                }

                _ => (),
            }

            ApplicationState::OutputSignedMenu => match key {
                Key::Delete => {
                    self.signed_result = None;
                    self.draw_full();
                }
                Key::Add => {
                    self.signed_result = Some(false);
                    self.draw_full();
                }
                Key::Subtract => {
                    self.signed_result = Some(true);
                    self.draw_full();
                }

                Key::FormatSelect | Key::Menu | Key::Exe => {
                    self.state = ApplicationState::Normal;
                    self.clear_evaluation(true);
                    self.draw_full();
                }

                _ => (),
            }

            ApplicationState::MainMenu => match key {
                Key::Digit(1) => {
                    self.state = ApplicationState::VariableView { page: 0 };
                    self.draw_full();
                }
                Key::Delete => self.hal.enter_bootloader().await,
                Key::Menu => {
                    self.state = ApplicationState::Normal;
                    self.draw_full();
                }

                _ => (),
            }

            ApplicationState::VariableView { ref mut page } => match key {
                Key::Left if *page > 0 => {
                    *page -= 1;
                    self.draw_full();
                }
                Key::Right if *page < 3 => {
                    *page += 1;
                    self.draw_full();
                }

                Key::FormatSelect | Key::Menu | Key::Exe => {
                    self.state = ApplicationState::Normal;
                    self.clear_evaluation(true);
                    self.draw_full();
                }

                _ => (),
            }
        }
        
    }
}
