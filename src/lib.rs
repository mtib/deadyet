use std::fmt::UpperHex;

pub trait HexPatternable {
    /// Converts the input into a vec of values 0 to 15
    /// ```
    /// use deadyet::HexPatternable;
    /// let dead: u64 = 0xDEAD;
    /// println!("{:?}", dead.to_hex());
    /// ```
    ///
    /// ```
    /// use deadyet::HexPatternable;
    /// assert_eq!(0x0123456789ABCDEFu64.to_hex(), [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])
    /// ```
    fn to_hex(&self) -> Vec<u8>;
}

impl<H: UpperHex> HexPatternable for H {
    fn to_hex(&self) -> Vec<u8> {
        let hex = format!("{:X}", self);
        hex.chars()
            .map(|x| match x {
                'A'..='F' => 10 + (x as u8) - ('A' as u8),
                _ => (x as u8) - ('0' as u8),
            })
            .collect()
    }
}

/// Checks if `number` contains the hex pattern "DEAD".
pub fn has_dead<H: HexPatternable>(number: H) -> bool {
    let dead: u64 = 0xDEAD;
    has_pattern(number, &dead)
}

/// Checks whether or not the pattern of `pattern` is withing the hex pattern of `number`.
///
/// ```
/// use deadyet::*;
/// let number: u64 = 0x12DE_AD34;
/// let dead: u64 = 0xDEAD;
/// assert!(has_pattern(number, dead));
/// assert!(!has_pattern(number ^ 0xFFFF_FFFF, dead));
/// ```
pub fn has_pattern<H: HexPatternable, P: HexPatternable>(number: H, pattern: P) -> bool {
    let nhex = number.to_hex();
    let phex = pattern.to_hex();
    let possible_starts = nhex.len() as isize - phex.len() as isize + 1;
    if possible_starts <= 0 {
        return false;
    }
    for i in 0..possible_starts as usize {
        if phex.as_slice() == &nhex[i..i + phex.len()] {
            return true;
        }
    }
    false
}
