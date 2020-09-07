use deadyet::HexPatternable;

#[test]
fn to_hex() {
    assert_eq!(
        0x0123456789ABCDEFu64.to_hex(),
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF]
    );
    assert_eq!(0xDEADu64.to_hex(), [0xD, 0xE, 0xA, 0xD]);
}
