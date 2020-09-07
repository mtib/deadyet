use deadyet::*;

fn main() {
    println!("{}", has_dead(0xEADDEADu64));
    println!("{:?}", (0xDEADu64 ^ 0xFFFF).to_hex());
}
