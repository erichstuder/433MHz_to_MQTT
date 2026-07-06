//! Miscellaneous utilities.

pub fn parse_ip(mqtt_host_ip: &[u8]) -> Result<(u8, u8, u8, u8), ()> {
    let mut ip = [0u8; 4];
    let mut count = 0;
    for (n, part) in mqtt_host_ip.split(|&b| b == b'.').enumerate() {
        if n >= ip.len() {
            return Err(())
        }
        let part = str::from_utf8(part).unwrap();
        let part = part.parse::<u8>().unwrap();
        ip[n] = part;
        count += 1;
    }
    if count != 4 {
        return Err(())
    }
    Ok((ip[0], ip[1], ip[2], ip[3]))
}

#[cfg(test)]
mod test_for_parse_ip {
    use super::*;

    #[test]
    fn pass() {
        let mqtt_host_ip = "123.55.6.2".as_bytes();
        let (ip0, ip1, ip2, ip3) = parse_ip(mqtt_host_ip).unwrap();
        assert_eq!(ip0, 123);
        assert_eq!(ip1, 55);
        assert_eq!(ip2, 6);
        assert_eq!(ip3, 2);
    }

    #[test]
    fn too_long() {
        assert!(parse_ip("123.55.6.2.42".as_bytes()).is_err());
    }

    #[test]
    fn too_short() {
        assert!(parse_ip("123.55.6".as_bytes()).is_err());
    }

    #[test]
    #[should_panic = "called `Result::unwrap()` on an `Err` value: ParseIntError { kind: InvalidDigit }"]
    fn letters() {
        let _ = parse_ip("123.55.6.X".as_bytes());
    }
}
