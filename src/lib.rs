//! This package allows you to efficiently search for hex patterns in the
//! hex representation of [Decodable](self::Decodable) values.
//!
//! It is intended to find and search for these patterns in unix timestamps, specifically
//! occurences of `DEAD` within the timestamp but is not limited to this.
//!
//! ```
//! use deadyet::{to_next_dead, to_next_pattern, has_pattern, Decodable};
//!
//! assert_eq!(to_next_dead(0xDEAE), 0xFFFF);
//! assert_eq!(to_next_dead(0xDEACFF), 1);
//! assert_eq!(to_next_dead(0xDEAD0), 0);
//! assert_eq!(to_next_dead(0xDEAC0), 0x10);
//!
//! assert_eq!(to_next_pattern(0xAAAAA, 0xABBA, 0xFFFF), 0x110);
//!
//! assert!(has_pattern(0xAABBAA, 0xABBA));
//! ```

use std::fmt::UpperHex;
use std::time::SystemTime;
#[macro_use]
extern crate cached;
#[macro_use]
extern crate lazy_static;

/// Implementors of this trait can be numerically expressed and reasonably mapped to
/// a `Vec<u8>` of hex digits.
///
/// The blanket trait implementation is [UpperHex](std::fmt::UpperHex) + Clone.
pub trait Decodable: Clone {
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

    /// Maps the hex representation back to a u64.
    fn to_pattern_u64(&self) -> u64;
}

impl<H: UpperHex + Clone> Decodable for H {
    fn to_hex(&self) -> Vec<u8> {
        let hex = format!("{:X}", self);
        hex.chars()
            .map(|x| match x {
                'A'..='F' => 10 + (x as u8) - b'A',
                _ => (x as u8) - b'0',
            })
            .collect()
    }

    /// Converts the pattern to a u64: `[13, 12, 10, 13] -> 0xDEAD`
    ///
    /// ```
    /// use deadyet::Decodable;
    /// assert_eq!(0xDEADu64.to_pattern_u64(), 0xDEADu64);
    /// ```
    fn to_pattern_u64(&self) -> u64 {
        let hex = &self.to_hex();
        hex.iter().fold(0u64, |a, x| a * 16 + *x as u64)
    }
}

/// Iterator to go through results of pattern search in order.
///
/// ```
/// use deadyet::*;
/// let mut deadpi = PatternIterator::new(0, 0xDEAD, 0xFFFF);
/// assert_eq!(deadpi.next(), Some(0xDEAD));
/// assert_eq!(deadpi.next(), Some(0x1DEAD));
/// assert_eq!(deadpi.next(), Some(0x2DEAD));
/// ```
///
/// ```
/// use deadyet::*;
/// let mut deadpi = PatternIterator::new(0xDEACAD, 0xDEAD, 0xFFFF);
/// assert_eq!(deadpi.next(), Some(0xDEAD00));
/// assert_eq!(deadpi.next(), Some(0xDEAD01));
/// assert_eq!(deadpi.next(), Some(0xDEAD02));
/// ```
pub struct PatternIterator {
    current: u64,
    pattern: u64,
    pattern_mask: u64,
}

impl PatternIterator {
    pub fn new<H: Decodable, I: Decodable, J: Decodable>(
        start: J,
        pattern: H,
        pattern_mask: I,
    ) -> PatternIterator {
        PatternIterator {
            current: start.to_pattern_u64(),
            pattern: pattern.to_pattern_u64(),
            pattern_mask: pattern_mask.to_pattern_u64(),
        }
    }
}

impl Iterator for PatternIterator {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        self.current += to_next_pattern(self.current, self.pattern, self.pattern_mask) + 1;
        Some(self.current - 1)
    }
}

/// Similar to [PatternIterator](self::PatternIterator), however efficiently describes ranges of pattern-truthness.
///
/// This iterator currenly only works with *full* pattern_mask ($2^i-1$)
///
/// ```
/// use deadyet::*;
///
/// let mut deadpri = PatternRangeIterator::new(0xDEAC0, 0xDEAD, 0xFFFF);
/// assert_eq!(deadpri.next(), Some((0xDEAD0, 0xDEADF)));
///
/// let mut b00b5pri = PatternRangeIterator::new(0xB00B00, 0xB00B5, 0xFFFFF);
/// assert_eq!(b00b5pri.next(), Some((0xB00B50, 0xB00B5F)));
/// assert_eq!(b00b5pri.next(), Some((0xBB00B5, 0xBB00B5)));
/// ```
pub struct PatternRangeIterator {
    current: u64,
    pattern: u64,
    pattern_mask: u64,
}
impl PatternRangeIterator {
    pub fn new<H: Decodable, I: Decodable, J: Decodable>(
        start: J,
        pattern: H,
        pattern_mask: I,
    ) -> PatternRangeIterator {
        PatternRangeIterator {
            current: start.to_pattern_u64(),
            pattern: pattern.to_pattern_u64(),
            pattern_mask: pattern_mask.to_pattern_u64(),
        }
    }
}

impl Iterator for PatternRangeIterator {
    type Item = (u64, u64);

    fn next(&mut self) -> Option<Self::Item> {
        let start_of_next =
            self.current + to_next_pattern(self.current, self.pattern, self.pattern_mask);
        let len = start_of_next.to_hex().len() as u64;
        let mut end = start_of_next;
        for i in (0..len).rev() {
            if has_pattern(start_of_next >> (i * 4), self.pattern) {
                end = start_of_next + 2u64.pow((i * 4) as u32) - 1;
                break;
            }
        }
        self.current = end + 1;
        Some((start_of_next, end))
    }
}

/// Creates a PatternRangeIterator for the pattern `DEAD`
pub fn dead_iterator<H: Decodable>(start: H) -> PatternIterator {
    PatternIterator::new(start, 0xDEAD, 0xFFFF)
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

/// Returns the time to the next dead ignoring the `lshd` least significant hex digits or 0 if `after` already contains `DEAD`.
/// In the case that there can be no `DEAD` after restricting the least significant bits, u64::MAX is returned.
pub fn to_next_dead_at_end(number: u64, lshd: usize) -> u64 {
    to_next_pattern_at_end(number, lshd, 0xDEAD, 0xFFFF)
}

use std::cmp::Ordering;

/// Returns the different to the next greater occurrence of the pattern.
pub fn to_next_pattern(number: u64, pattern: u64, pattern_mask: u64) -> u64 {
    cached_to_next_pattern(number, pattern, pattern_mask)
}

use cached::SizedCache;

cached! {
    COMPUTE: SizedCache<(u64, u64, u64), u64> = SizedCache::with_size(8192);
    fn cached_to_next_pattern( number: u64, pattern: u64, pattern_mask: u64) -> u64 = {
        let mut min = u64::MAX;
        let hexa = number.to_hex();
        for i in 0..(hexa.len()) {
            let x = to_next_pattern_at_end(number, i, pattern, pattern_mask);
            min = if min > x { x } else { min };
        }
        min
    }
}

/// Returns the difference to the next greater occurrence of the `pattern` in relation to `number`.
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
