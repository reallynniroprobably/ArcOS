// src/shutdown.rs

use crate::println;

pub fn shutdown() -> ! {
    println!("\nStarting shutdown processs");
    loop {}
}