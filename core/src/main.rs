#[cfg(not(test))]
use std::process::exit;

#[cfg(not(test))]
fn main() {
    exit(core::run());
}
