#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate hex;
extern crate serde;

use deadyet::*;
use rocket::http::RawStr;
use rocket::request::FromParam;
use rocket_contrib::templates::Template;
use serde::Serialize;

struct Hex {
    value: usize,
}

impl<'r> FromParam<'r> for Hex {
    type Error = String;
    fn from_param(param: &'r RawStr) -> Result<Self, String> {
        match usize::from_str_radix(param.as_str(), 16) {
            Ok(num) => Ok(Hex { value: num }),
            Err(e) => Err(e.to_string()),
        }
    }
}

impl std::fmt::Display for Hex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:X}", self.value)
    }
}

impl std::fmt::UpperHex for Hex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::UpperHex::fmt(&self.value, f) // delegate to i32's implementation
    }
}

#[get("/<number>/<pattern>")]
fn check(number: Hex, pattern: Hex) -> String {
    format!(
        "find {} in {} -> {}",
        &pattern,
        &number,
        has_pattern(&number, &pattern)
    )
}

#[get("/<number>")]
fn is_dead_hex(number: Hex) -> &'static str {
    if has_dead(number) {
        "yes"
    } else {
        "no"
    }
}

#[get("/<number>")]
fn is_dead_dec(number: usize) -> &'static str {
    if has_dead(number) {
        "yes"
    } else {
        "no"
    }
}

#[get("/")]
fn is_dead_now() -> Template {
    #[derive(Serialize)]
    struct IsDead {
        dead: bool,
        next_s: u64,
        next_time: u64,
    }
    let (next_s, next_time) = next_dead();
    Template::render(
        "is_it_dead",
        IsDead {
            dead: is_it_dead(),
            next_s,
            next_time,
        },
    )
}

fn main() {
    rocket::ignite()
        .attach(Template::fairing())
        .mount("/check", routes![check])
        .mount("/dead_hex", routes![is_dead_hex])
        .mount("/dead_dec", routes![is_dead_dec])
        .mount("/", routes![is_dead_now])
        .launch();
}
