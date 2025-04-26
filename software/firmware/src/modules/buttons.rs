//! Parses a button code into a button number.

pub struct Buttons {
    last_value: Option<u32>,
    value_cnt: u8,
}

impl Buttons {
    pub fn new() -> Self {
        Self {
            last_value: None,
            value_cnt: 0,
        }
    }

    pub fn match_button(&mut self, value: u32) -> Option<&'static str> {
        // Return the the button if it was read more than once in a row.
        match self.last_value {
            Some(last) if value == last => {
                self.value_cnt += 1;
            }
            _ => {
                self.last_value = Some(value);
                self.value_cnt = 1;
            }
        }

        if self.value_cnt >= 2 {
            match value {
                0x017E9E90u32 => return Some("button 1"),
                0x017E9E88u32 => return Some("button 2"),
                0x017E9E98u32 => return Some("button 3"),
                0x017E9E84u32 => return Some("button 4"),
                0x017E9E94u32 => return Some("button 5"),
                0x017E9E8Cu32 => return Some("button 6"),
                0x017E9E9Cu32 => return Some("button 7"),
                0x017E9E82u32 => return Some("button 8"),
                0x017E9E92u32 => return Some("button 9"),
                0x017E9E8Au32 => return Some("button 10"),
                _ => return Some("undefined button"),
            }
        }
        None
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buttons() {
        let mut buttons = Buttons::new();
        let values: Vec<(u32, &str)> = vec![
            (0x017E9E90u32, "button 1"),
            (0x017E9E88u32, "button 2"),
            (0x017E9E98u32, "button 3"),
            (0x017E9E84u32, "button 4"),
            (0x017E9E94u32, "button 5"),
            (0x017E9E8Cu32, "button 6"),
            (0x017E9E9Cu32, "button 7"),
            (0x017E9E82u32, "button 8"),
            (0x017E9E92u32, "button 9"),
            (0x017E9E8Au32, "button 10"),
            (42u32, "undefined button"),
        ];

        for (value, button) in values {
            // first time is expected None
            let result_button = buttons.match_button(value);
            assert_eq!(result_button, None, "expected button: {}", button);

            // second time is expected the correct button
            let result_button = buttons.match_button(value);
            assert_eq!(result_button.unwrap(), button, "expected button: {}", button);

            // third and more times are also expected the correct button
            let result_button = buttons.match_button(value);
            assert_eq!(result_button.unwrap(), button, "expected button: {}", button);
        }
    }
}
