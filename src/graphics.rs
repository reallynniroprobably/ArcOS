// src/graphics.rs

use core::ptr::{copy_nonoverlapping};

use alloc::vec;

use limine::request::FramebufferRequest;

#[used]
#[unsafe(link_section = ".requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[derive(Debug)]
pub struct Framebuffer {
    addr: *mut (),
    backbuffer: vec::Vec<u32>,
    pub height: usize,
    pub width: usize,
    pitch: u64,
    bpp: u16,
    bytes_per_pixel: u16,
    red_mask_shift: u8,
    red_mask_size: u8,
    green_mask_shift: u8,
    green_mask_size: u8,
    blue_mask_shift: u8,
    blue_mask_size: u8,
}
impl Framebuffer {
    pub fn new() -> Self {
        let framebuffer_response = FRAMEBUFFER_REQUEST
            .response()
            .expect("No framebuffer response")
            .framebuffers()
            .iter()
            .next()
            .expect("No framebuffers available");

        Framebuffer {
            addr: framebuffer_response.address(),
            backbuffer: vec![0x000000; (framebuffer_response.height * framebuffer_response.width) as usize],
            height: framebuffer_response.height as usize,
            width: framebuffer_response.width as usize,
            pitch: framebuffer_response.pitch,
            bpp: framebuffer_response.bpp,
            bytes_per_pixel: framebuffer_response.bpp / 8,
            red_mask_shift: framebuffer_response.red_mask_shift,
            red_mask_size: framebuffer_response.red_mask_size,
            green_mask_shift: framebuffer_response.green_mask_shift,
            green_mask_size: framebuffer_response.green_mask_size,
            blue_mask_shift: framebuffer_response.blue_mask_shift,
            blue_mask_size: framebuffer_response.blue_mask_size
        }
    }

    pub fn swap(&self) {
        unsafe { copy_nonoverlapping(self.backbuffer.as_ptr(), self.addr as *mut u32, self.backbuffer.len()); }
    }

    fn dst_index(&self, x: u64, y: u64) -> Result<usize, &'static str> {
        if x > self.width as u64|| y > self.height as u64 { return Err("Coordinates are out of bounds"); }
        let index = (y * (self.pitch / (self.bpp as u64 / 8))+ x) as usize;
        Ok(index)
    }

    fn draw_pixel(&mut self, colour: &Colour, x: u64, y: u64) -> Result<(), &'static str> {
        let dst = self.dst_index(x, y)?;
        self.backbuffer[dst] = colour.pack(self);
        Ok(())
    }

    pub fn rect(&mut self, colour: &Colour, x: u64, y: u64, height: usize, width: usize) -> Result<(), &'static str> {
        let mut dst = self.dst_index(x, y)?;
        let colour = colour.pack(self);
        
        let row = vec![colour; width];
        for _ in 0..height {
            self.backbuffer[dst..dst + width].clone_from_slice(&row);
            dst += self.width;
        }
        Ok(())
    }

    pub fn clear(&mut self, colour: &Colour) -> Result<(), &'static str> {
        self.rect(colour, 0, 0, self.height, self.width)
    }

    pub fn bitmap(&mut self, bitmap: &Bitmap, mut x: u64, mut y: u64, scale: f32) -> Result<(), &'static str> {
        for row in &bitmap.rows {
            #[allow(unused_assignments)]
            let mut len: usize = 0;
            let mut last_val: u8 = 0;
            for (index, pixel) in row.pixels.iter().enumerate() {
                let val = if *pixel == Colour::from_hex(0xFFFFFF) { 1 } else { 2 };

                if last_val == 0 {
                    last_val = val;
                    len = 1;
                    continue;
                }

                if last_val == val {
                    len += 1;
                } else {
                    if last_val == 2 {
                        self.rect(&Colour::from_hex(0x000000), x, y, scale as usize, (len as f32 * scale) as usize)?;
                    }
                    x += (row.length as f32 * scale) as u64;
                    len = 1;
                    last_val = val;
                }

                if index == row.length - 1 {
                    if last_val == 2 {
                        self.rect(&Colour::from_hex(0x000000), x, y, scale as usize, (len as f32 * scale) as usize)?;
                    }
                    let offset = ((row.length - len) as f32 * scale) as u64;

                    crate::println!("x: {x}, offset: {offset}");
                    
                    x -= offset;   
                    y += scale as u64;
                    len = 0;
                    last_val = 0;
                }
                crate::println!("Pixel {index} done!");
            }
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Colour {
    red: u8,
    green: u8,
    blue: u8
}
impl Colour {
    pub fn from_rgb(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red,
            green,
            blue
        }
    }

    pub fn from_hex(hex: u32) -> Self {
        Self {
            red: (hex >> 16) as u8,
            green: (hex >> 8) as u8,
            blue: hex as u8
        }
    }

    pub fn pack(&self, framebuffer: &Framebuffer) -> u32 {
        let mut colour = 0u32;

        colour |= (self.red as u32) << framebuffer.red_mask_shift;
        colour |= (self.green as u32) << framebuffer.green_mask_shift;
        colour |= (self.blue as u32) << framebuffer.blue_mask_shift;
        
        colour
    }
}

pub struct Bitmap {
    height: usize,
    rows: vec::Vec<BitmapRow>
}
impl Bitmap {
    pub fn full(height: usize, length: usize) -> Self {
        Self {
            height,
            rows: vec![BitmapRow {
                length,
                pixels: vec![Colour::from_hex(0x000000); length]
            }; height]
        }
    }
}

#[derive(Clone)]
struct BitmapRow {
    length: usize,
    pixels: vec::Vec<Colour>
}