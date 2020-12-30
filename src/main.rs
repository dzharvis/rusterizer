mod tga;

fn main() {
    let mut img = tga::Image::new(640, 480);
    for y in 0u32..480 {
        for x in 0u32..640 {
            let r = ((x ^ y) % 256) as u8;
            let g = ((x + y) % 256) as u8;
            let b = ((y.wrapping_sub(x)) % 256) as u8;
            img.set_pixel(x as i32, y as i32, tga::Color(r, g, b));
        }
    }
    img.apply_gamma(2.2);
    img.write_to_tga("test.tga").unwrap();
}