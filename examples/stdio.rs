use std::{
    io::{BufRead, Cursor, Write},
    time::Duration,
};

fn main() {
    let mut ctx = libusb::Context::new().expect("LibUSB context could not be created");

    let mut printer =
        pos58_usb::POS58USB::new(&mut ctx, Duration::from_secs(2)).expect("Failed to get printer");

    let stdin = std::io::stdin().lock();

    for line in stdin.lines() {
        let line = line.expect("Failed to get line");
        writeln!(printer, "{}", line).expect("Failed to print");
    }
}
