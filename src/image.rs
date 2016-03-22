use imagefmt;
use imagefmt::{ColFmt, ColType};

#[derive(Clone, Copy)]
pub struct Color(pub u8, pub u8, pub u8);

pub const WHITE: Color = Color(255, 255, 255);
pub const BLUE: Color = Color(0, 0, 255);
pub const RED: Color = Color(255, 0, 0);
pub const GREEN: Color = Color(0, 255, 0);

impl Color {
    pub fn multiply(self, other: &Color) -> Color {
        Color(
            (self.0 as usize * other.0 as usize / 255) as u8,
            (self.1 as usize * other.1 as usize / 255) as u8,
            (self.2 as usize * other.2 as usize / 255) as u8,
        )
    }
}

pub struct Image {
    pub data: Vec<u8>,
    zbuffer: Vec<isize>,
    pub width: usize,
    pub height: usize,
}

impl Image {
    pub fn set_pixel(&mut self, x: usize, y: usize, color: &Color) {
        if x < self.width && y < self.height {
            let index = ((self.height - y - 1) * self.width + x) * 3;

            self.data[index] = color.0;
            self.data[index+1] = color.1;
            self.data[index+2] = color.2;
        }
    }

    pub fn set_pixel_with_depth(&mut self, x: usize, y: usize, color: &Color, depth: isize) {
        if x < self.width && y < self.height {
            let zbufferindex = (self.height - y - 1) * self.width + x;
            let index = zbufferindex * 3;

            if self.zbuffer[zbufferindex] < depth {
                self.zbuffer[zbufferindex] = depth;

                self.data[index] = color.0;
                self.data[index+1] = color.1;
                self.data[index+2] = color.2;
            }
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Color {
        let index = ((self.height - y - 1) * self.width + x) * 3;
        Color(self.data[index], self.data[index + 1], self.data[index + 2])
    }

    pub fn write(&self, filename: &str) -> imagefmt::Result<()> {
        imagefmt::write(
            filename,
            self.width,
            self.height,
            ColFmt::RGB,
            &self.data,
            ColType::Color)
    }

    pub fn new(width: usize, height: usize) -> Image {
        let mut data = Vec::with_capacity(width * height * 3);
        data.resize(width * height * 3, 0);
        let mut zbuffer = Vec::with_capacity(width * height);
        zbuffer.resize(width * height, isize::min_value());

        Image {
            data: data,
            zbuffer: zbuffer,
            width: width,
            height: height,
        }
    }

    pub fn from(filename: &str) -> Image {
        let img = imagefmt::read(filename, ColFmt::RGB).unwrap();

        Image {
            data: img.buf,
            zbuffer: Vec::new(),
            width: img.w,
            height: img.h,
        }
    }
}
