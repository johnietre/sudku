pub fn encoded_len(s: impl AsRef<[u8]>) -> usize {
    (s.as_ref().len() * 8 + 5) / 6
}

pub fn decoded_len(s: impl AsRef<str>) -> usize {
    s.as_ref().len() * 6 / 8
}

pub fn encode(s: impl AsRef<[u8]>) -> String {
    let s = s.as_ref();
    let mut res = String::with_capacity(encoded_len(s));
    for chunk in s.chunks(3) {
        match chunk {
            [b1] => {
                res.push(byte_to_char(b1 >> 2).unwrap() as char);
                res.push(byte_to_char((b1 << 6) >> 2).unwrap() as char);
            }
            [b1, b2] => {
                res.push(byte_to_char(b1 >> 2).unwrap() as char);
                res.push(byte_to_char(((b1 << 6) >> 2) | (b2 >> 4)).unwrap() as char);
                res.push(byte_to_char((b2 << 4) >> 2).unwrap() as char);
            }
            [b1, b2, b3] => {
                res.push(byte_to_char(b1 >> 2).unwrap() as char);
                res.push(byte_to_char(((b1 << 6) >> 2) | (b2 >> 4)).unwrap() as char);
                res.push(byte_to_char(((b2 << 4) >> 2) | (b3 >> 6)).unwrap() as char);
                res.push(byte_to_char((b3 << 2) >> 2).unwrap() as char);
                /*
                println!("{:06b}", b1 >> 2);
                println!("{:06b}", ((b1 << 6) >> 2) | (b2 >> 4));
                println!("{:06b}", ((b2 << 4) >> 2) | (b3 >> 6));
                println!("{:06b}", (b3 << 2) >> 2);
                */
            }
            _ => unreachable!(),
        }
    }
    res
}

pub fn decode(s: impl AsRef<str>) -> Option<Vec<u8>> {
    let s = s.as_ref();
    let mut res = Vec::with_capacity(decoded_len(s));
    for chunk in s.as_bytes().chunks(4) {
        match chunk {
            [c1, c2] => {
                let (b1, b2) = (char_to_byte(*c1)?, char_to_byte(*c2)?);
                res.push(b1 << 2 | b2 >> 4);
            }
            [c1, c2, c3] => {
                let (b1, b2, b3) = (char_to_byte(*c1)?, char_to_byte(*c2)?, char_to_byte(*c3)?);
                res.push(b1 << 2 | b2 >> 4);
                res.push(b2 << 4 | b3 >> 2);
            }
            [c1, c2, c3, c4] => {
                let (b1, b2, b3, b4) = (
                    char_to_byte(*c1)?, char_to_byte(*c2)?,
                    char_to_byte(*c3)?, char_to_byte(*c4)?,
                );
                res.push(b1 << 2 | b2 >> 4);
                res.push(b2 << 4 | b3 >> 2);
                res.push(b3 << 6 | b4);
            }
            _ => return None,
        }
    }
    Some(res)
}

const fn byte_to_char(b: u8) -> Option<u8> {
    match b {
        0 => Some(b'A'),
        1 => Some(b'B'),
        2 => Some(b'C'),
        3 => Some(b'D'),
        4 => Some(b'E'),
        5 => Some(b'F'),
        6 => Some(b'G'),
        7 => Some(b'H'),
        8 => Some(b'I'),
        9 => Some(b'J'),
        10 => Some(b'K'),
        11 => Some(b'L'),
        12 => Some(b'M'),
        13 => Some(b'N'),
        14 => Some(b'O'),
        15 => Some(b'P'),
        16 => Some(b'Q'),
        17 => Some(b'R'),
        18 => Some(b'S'),
        19 => Some(b'T'),
        20 => Some(b'U'),
        21 => Some(b'V'),
        22 => Some(b'W'),
        23 => Some(b'X'),
        24 => Some(b'Y'),
        25 => Some(b'Z'),
        26 => Some(b'a'),
        27 => Some(b'b'),
        28 => Some(b'c'),
        29 => Some(b'd'),
        30 => Some(b'e'),
        31 => Some(b'f'),
        32 => Some(b'g'),
        33 => Some(b'h'),
        34 => Some(b'i'),
        35 => Some(b'j'),
        36 => Some(b'k'),
        37 => Some(b'l'),
        38 => Some(b'm'),
        39 => Some(b'n'),
        40 => Some(b'o'),
        41 => Some(b'p'),
        42 => Some(b'q'),
        43 => Some(b'r'),
        44 => Some(b's'),
        45 => Some(b't'),
        46 => Some(b'u'),
        47 => Some(b'v'),
        48 => Some(b'w'),
        49 => Some(b'x'),
        50 => Some(b'y'),
        51 => Some(b'z'),
        52 => Some(b'0'),
        53 => Some(b'1'),
        54 => Some(b'2'),
        55 => Some(b'3'),
        56 => Some(b'4'),
        57 => Some(b'5'),
        58 => Some(b'6'),
        59 => Some(b'7'),
        60 => Some(b'8'),
        61 => Some(b'9'),
        62 => Some(b'+'),
        63 => Some(b'/'),
        _ => None,
    }
}

const fn char_to_byte(c: u8) -> Option<u8> {
    match c {
        b'A' => Some(0),
        b'B' => Some(1),
        b'C' => Some(2),
        b'D' => Some(3),
        b'E' => Some(4),
        b'F' => Some(5),
        b'G' => Some(6),
        b'H' => Some(7),
        b'I' => Some(8),
        b'J' => Some(9),
        b'K' => Some(10),
        b'L' => Some(11),
        b'M' => Some(12),
        b'N' => Some(13),
        b'O' => Some(14),
        b'P' => Some(15),
        b'Q' => Some(16),
        b'R' => Some(17),
        b'S' => Some(18),
        b'T' => Some(19),
        b'U' => Some(20),
        b'V' => Some(21),
        b'W' => Some(22),
        b'X' => Some(23),
        b'Y' => Some(24),
        b'Z' => Some(25),
        b'a' => Some(26),
        b'b' => Some(27),
        b'c' => Some(28),
        b'd' => Some(29),
        b'e' => Some(30),
        b'f' => Some(31),
        b'g' => Some(32),
        b'h' => Some(33),
        b'i' => Some(34),
        b'j' => Some(35),
        b'k' => Some(36),
        b'l' => Some(37),
        b'm' => Some(38),
        b'n' => Some(39),
        b'o' => Some(40),
        b'p' => Some(41),
        b'q' => Some(42),
        b'r' => Some(43),
        b's' => Some(44),
        b't' => Some(45),
        b'u' => Some(46),
        b'v' => Some(47),
        b'w' => Some(48),
        b'x' => Some(49),
        b'y' => Some(50),
        b'z' => Some(51),
        b'0' => Some(52),
        b'1' => Some(53),
        b'2' => Some(54),
        b'3' => Some(55),
        b'4' => Some(56),
        b'5' => Some(57),
        b'6' => Some(58),
        b'7' => Some(59),
        b'8' => Some(60),
        b'9' => Some(61),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}
