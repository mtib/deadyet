use std::fmt::UpperHex;
use std::time::SystemTime;

pub trait Decodable {
    /// Converts the input into a vec of values 0 to 15
    /// ```
    /// use deadyet::Decodable;
    /// let dead: u64 = 0xDEAD;
    /// println!("{:?}", dead.to_hex());
    /// ```
    ///
    /// ```
    /// use deadyet::Decodable;
    /// assert_eq!(0x0123456789ABCDEFu64.to_hex(), [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])
    /// ```
    fn to_hex(&self) -> Vec<u8>;
}

impl<H: UpperHex> Decodable for H {
    fn to_hex(&self) -> Vec<u8> {
        let hex = format!("{:X}", self);
        hex.chars()
            .map(|x| match x {
                'A'..='F' => 10 + (x as u8) - b'A',
                _ => (x as u8) - b'0',
            })
            .collect()
    }
}

/// Checks if `number` contains the hex pattern "DEAD".
pub fn has_dead<H: Decodable>(number: H) -> bool {
    let dead: u64 = 0xDEAD;
    has_pattern(number, &dead)
}

/// Checks whether or not the pattern of `pattern` is within the hex pattern of `number`.
///
/// ```
/// use deadyet::*;
/// let number: u64 = 0x12DE_AD34;
/// let dead: u64 = 0xDEAD;
/// assert!(has_pattern(number, dead));
/// assert!(!has_pattern(number ^ 0xFFFF_FFFF, dead));
/// ```
pub fn has_pattern<H: Decodable, P: Decodable>(number: H, pattern: P) -> bool {
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

fn current_unix() -> u64 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

/// Returns whether the current unix timecode contains a `DEAD`.
pub fn is_it_dead() -> bool {
    has_dead(current_unix())
}

/// Returns the number of seconds until the next `DEAD` in the unix
/// timestamp.
pub fn secs_until_dead() -> u64 {
    to_next_dead(current_unix())
}

/// Returns the tuple (diff, abs) for the time until the next
/// `DEAD` as well as the unix timestamp of that event.
pub fn next_dead() -> (u64, u64) {
    let now = current_unix();
    let diff = to_next_dead(now);
    (diff, now + diff)
}

// Returns the time to next dead (or 0 if `after` is dead)
pub fn to_next_dead(number: u64) -> u64 {
    to_next_pattern(number, 0xDEAD, 0xFFFF)
}

/// Returns time time to the next dead ignoring the `lshd` least significant hex digits or 0 if `after` already contains `DEAD`.
/// In the case that there can be no `DEAD` after restricting the least significant bits, u64::MAX is returned.
pub fn to_next_dead_at_end(number: u64, lshd: usize) -> u64 {
    to_next_pattern_at_end(number, lshd, 0xDEAD, 0xFFFF)
}

use std::cmp::Ordering;

pub fn to_next_pattern(number: u64, pattern: u64, pattern_mask: u64) -> u64 {
    let mut min = u64::MAX;
    let hexa = number.to_hex();
    for i in 0..(hexa.len()) {
        let x = to_next_pattern_at_end(number, i, pattern, pattern_mask);
        min = if min > x { x } else { min };
    }
    min
}

pub fn to_next_pattern_at_end(number: u64, lshd: usize, pattern: u64, pattern_mask: u64) -> u64 {
    let remainder_mask = 2u64.pow(lshd as u32 * 4) - 1;
    //let remainder = remainder_mask - (number & remainder_mask);
    let restricted = (number >> (lshd * 4)) & pattern_mask;
    match restricted.cmp(&pattern) {
        Ordering::Greater => {
            ((pattern_mask - restricted + pattern + 1) << (lshd * 4)) - (number & remainder_mask)
        }
        Ordering::Equal => 0,
        Ordering::Less => ((pattern - restricted) << (lshd * 4)) - (number & remainder_mask),
    }
}
