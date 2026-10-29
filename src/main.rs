// src/main.rs
#![no_std]
#![no_main]

extern crate alloc;

mod memory;
mod graphics;
mod console;
mod shutdown;

use crate::graphics::{Colour, Framebuffer};

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("PANIC: {}", info.message());
    loop { }
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    println!("\n\n\nReally cool operating system initialising...\n");

    println!("Initiating memory");
    
    memory::init_heap();

    println!("Sucessfully initiated memory\n");
    
    println!("Initiating framebuffer...");

    let mut framebuffer: Framebuffer = Framebuffer::new();

    println!("Sucessfully initiated framebuffer\n");
    
    framebuffer.clear(&Colour::from_rgb(255, 199, 44)).unwrap();

    framebuffer.rect(&Colour::from_rgb(217, 41, 28), 25, 25, framebuffer.height - 50, framebuffer.width - 50).unwrap();

    framebuffer.swap();
    
    loop { }
}