#![cfg_attr(not(test), no_std)]

pub fn ten() -> [u8;2] {
    [b'1',b'0']
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn my_first_test() {
        assert_eq!(ten(), [b'1',b'0']);
    }
}
