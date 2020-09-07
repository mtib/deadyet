use deadyet::*;

#[test]
fn u64_to_hex_test() {
    assert_eq!(
        0x0123456789ABCDEFu64.to_hex(),
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF]
    );
    assert_eq!(0xDEADu64.to_hex(), [0xD, 0xE, 0xA, 0xD]);
}

#[test]
fn next_dead_at_end_test() {
    assert_eq!(to_next_dead_at_end(0xDEAC, 0), 1);
    assert_eq!(to_next_dead_at_end(0xDEAD, 0), 0);
    assert_eq!(to_next_dead_at_end(0xDEAE, 0), 0xFFFF);

    assert_eq!(to_next_dead_at_end(0xDEAC0, 1), 0x10);
    assert_eq!(to_next_dead_at_end(0xDEAD0, 1), 0x0);
    assert_eq!(to_next_dead_at_end(0xDEAE0, 1), 0xFFFF0);

    assert_eq!(to_next_dead_at_end(0xDEAC1, 1), 15);
}

#[test]
fn next_dead_test() {
    assert_eq!(to_next_dead(0xDEAC), 1);
    assert_eq!(to_next_dead(0xDEAD), 0);
    assert_eq!(to_next_dead(0xDEAE), 0xFFFF);

    assert_eq!(to_next_dead(0xDEAC0), 0x10);
    assert_eq!(to_next_dead(0xDEAD0), 0x0);
    assert_eq!(to_next_dead(0xDEAE0), 62413);

    assert_eq!(to_next_dead(0xDEAC1), 15);

    assert_eq!(to_next_dead(0xDEACFF), 0x1);

    assert_eq!(to_next_dead(0xDEACEAD), 0x1000 - 0xEAD);
}

#[test]
fn range_iterator_test() {
    let mut ri = PatternRangeIterator::new(0, 0xDEAD, 0xFFFF);
    assert_eq!(ri.next(), Some((0xDEAD, 0xDEAD)));
    assert_eq!(ri.next(), Some((0x1DEAD, 0x1DEAD)));
    assert_eq!(ri.next(), Some((0x2DEAD, 0x2DEAD)));
    assert_eq!(ri.next(), Some((0x3DEAD, 0x3DEAD)));
    assert_eq!(ri.next(), Some((0x4DEAD, 0x4DEAD)));
    assert_eq!(ri.next(), Some((0x5DEAD, 0x5DEAD)));
    assert_eq!(ri.next(), Some((0x6DEAD, 0x6DEAD)));
    assert_eq!(ri.next(), Some((0x7DEAD, 0x7DEAD)));
    assert_eq!(ri.next(), Some((0x8DEAD, 0x8DEAD)));
    assert_eq!(ri.next(), Some((0x9DEAD, 0x9DEAD)));
    assert_eq!(ri.next(), Some((0xADEAD, 0xADEAD)));
    assert_eq!(ri.next(), Some((0xBDEAD, 0xBDEAD)));
    assert_eq!(ri.next(), Some((0xCDEAD, 0xCDEAD)));
    assert_eq!(ri.next(), Some((0xDDEAD, 0xDDEAD)));
    assert_eq!(ri.next(), Some((0xDEAD0, 0xDEADF)));
    assert_eq!(ri.next(), Some((0xEDEAD, 0xEDEAD)));
}
