use core::fmt::{Write, Arguments};

use lazy_static::lazy_static;
use spin::mutex::Mutex;
use uart_16550::{Config, Uart16550Tty, backend::PioBackend};

lazy_static! { pub static ref SERIAL: Mutex<Serial> = Mutex::new(Serial::init()); }

pub struct Serial {
    uart: Uart16550Tty<PioBackend>
}
impl Serial {
    fn init() -> Self {
        Self {
            uart: unsafe {
                Uart16550Tty::new_port(
                    0x3F8,
                    Config::default()
                ).expect("Should initialise device")
            }
        }
    }
}
impl Write for Serial {
    fn write_char(&mut self, c: char) -> core::fmt::Result {
        self.uart.write_char(c)
    }
    
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.uart.write_str(s)
    }
}

#[doc(hidden)]
pub fn _print(args: Arguments) {
    SERIAL.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\r\n"));
    ($($arg:tt)*) => ($crate::print!("{}\r\n", format_args!($($arg)*)));
}