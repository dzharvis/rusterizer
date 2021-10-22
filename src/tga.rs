use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::mem;
use std::slice;

#[derive(Clone, Debug, Copy)]
pub struct Color(pub u8, pub u8, pub u8);

impl Color {
    pub fn highlight(self, p: f32) -> Self {
        let Color(r, g, b) = self;
        // Color((r as f32 *p).min(255.0) as u8, (g as f32 *p).min(255.0) as u8, (b  as f32 * p).min(255.0) as u8,)
        let fr = ((r as f32) / 255.0).powf(1.0 - p / 2.3);
        let fg = ((g as f32) / 255.0).powf(1.0 - p / 2.3);
        let fb = ((b as f32) / 255.0).powf(1.0 - p / 2.3);
        Color((fr * 255.0) as u8, (fg * 255.0) as u8, (fb * 255.0) as u8)
    }
}

#[derive(Clone, Debug)]
pub struct ColorA(pub u8, pub u8, pub u8, pub u8);

pub struct Image {
    pub width: i32,
    pub height: i32,
    pub data: Vec<Color>,
}

unsafe fn struct_to_u8_slice<T>(s: &T) -> &[u8] {
    let data_ptr: *const u8 = mem::transmute(s);
    slice::from_raw_parts(data_ptr, mem::size_of::<T>())
}

unsafe fn slice_to_u8_slice<T>(s: &[T]) -> &[u8] {
    let data_ptr: *const u8 = mem::transmute(&s[0]);
    slice::from_raw_parts(data_ptr, mem::size_of::<T>() * s.len())
}

impl Image {
    pub fn new(width: i32, height: i32) -> Image {
        let v = vec![Color(0, 0, 0); (width * height) as usize];
        Image {
            width,
            height,
            data: v,
        }
    }

    pub fn pixel_at(&self, x: i32, y: i32) -> Color {
        *self.data
            .get((x + y * self.width) as usize)
            .unwrap_or(&Color(0, 0, 0))
    }

    pub fn pixel_atf(&self, u: f32, v: f32) -> Color {
        let x = (((u + 1.0) / 2.0) * self.width as f32) as i32;
        let y = (((v + 1.0) / 2.0) * self.height as f32) as i32;
        *self.data
            .get((x + y * self.width) as usize)
            .unwrap_or(&Color(0, 0, 0))
    }

    pub fn apply_gamma(self: &mut Image, gamma: f32) {
        for c in self.data.iter_mut() {
            let Color(r, g, b) = *c;
            let fr = ((r as f32) / 255.0).powf(gamma);
            let fg = ((g as f32) / 255.0).powf(gamma);
            let fb = ((b as f32) / 255.0).powf(gamma);
            c.0 = (fr * 255.0) as u8;
            c.1 = (fg * 255.0) as u8;
            c.2 = (fb * 255.0) as u8;
        }
    }

    pub fn set_pixel(self: &mut Image, x: i32, y: i32, c: Color) {
        self.data[(x + y * self.width) as usize] = c;
    }

    pub fn get_raw_bytes(&self) -> Vec<u8> {
        let mut res: Vec<u8> = Vec::new();

        let mut flipped: Vec<Color> = vec![Color(0, 0, 0,); (self.width * self.height) as usize];
        for y in 0..self.height {
            for x in 0..self.width {
                flipped[(x + ((self.height - 1) - y) * self.width) as usize] =
                    self.data[(x + y * self.width) as usize]
            }
        }

        for Color(r, g, b) in flipped {
            res.push(b);
            res.push(g);
            res.push(r);
            res.push(255);
        }
        res
    }

    pub fn from_raw_vec(v: Vec<u8>) -> Self {
        #[repr(C, packed)]
        #[derive(Debug, Copy, Clone)]
        struct Header {
            id_length: u8,
            color_map_type: u8,
            image_type: u8,
            c_map_start: u16,
            c_map_length: u16,
            c_map_depth: u8,
            x_offset: u16,
            y_offset: u16,
            width: u16,
            height: u16,
            pixel_depth: u8,
            image_descriptor: u8,
        }

        let mut header: Header = unsafe { mem::zeroed() };
        let header_size = mem::size_of::<Header>();
        unsafe {
            let header_slice =
                slice::from_raw_parts_mut(&mut header as *mut _ as *mut u8, header_size);
            let mut r = BufReader::new(&v[..]);
            r.read_exact(header_slice).unwrap();

            let pixels = vec![ColorA(0, 0, 0, 0); header.width as usize * header.height as usize];
            let pixels_size = mem::size_of::<ColorA>() * pixels.len();
            let data_ptr: *mut u8 = mem::transmute(&pixels[..][0]);
            let pixels_slice = slice::from_raw_parts_mut(data_ptr, pixels_size);
            r.read_exact(pixels_slice).unwrap();

            let data_correct = {
                let mut v = vec![Color(0, 0, 0); pixels.len()];
                for y in 0..header.height {
                    for x in 0..header.width {
                        let p = &pixels[y as usize * header.width as usize + x as usize];
                        v[((header.height - 1) - y) as usize * header.width as usize
                            + x as usize] = Color(p.0, p.1, p.2);
                    }
                }
                v
            };

            Image {
                width: header.width as i32,
                height: header.height as i32,
                data: data_correct,
            }
        }
    }

    pub fn from_file(f: String) -> Self {
        let mut f = File::open(f).unwrap();
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).unwrap();
        Image::from_raw_vec(buf)
    }

    pub fn write_to_tga(self: &Image, filename: &str) -> io::Result<()> {
        #[repr(C, packed)]
        #[derive(Default)]
        struct Header {
            id_length: u8,
            color_map_type: u8,
            image_type: u8,
            c_map_start: u16,
            c_map_length: u16,
            c_map_depth: u8,
            x_offset: u16,
            y_offset: u16,
            width: u16,
            height: u16,
            pixel_depth: u8,
            image_descriptor: u8,
        }
        let h = Header {
            image_type: 2,
            width: self.width as u16,
            height: self.height as u16,
            pixel_depth: 24,
            ..Header::default()
        };

        let mut f = File::create(filename)?;
        unsafe {
            f.write_all(struct_to_u8_slice(&h))?;
            f.write_all(slice_to_u8_slice(&self.data[..]))?;
        }
        Ok(())
    }
}
