use image::{DynamicImage, GenericImage, GenericImageView, RgbImage, Rgba, RgbaImage};

#[derive(Debug)]
pub struct Texture {
    pub image: DynamicImage,
}

impl Texture {
    pub fn from_file(path: &std::path::Path) -> Self {
        let image = image::io::Reader::open(path)
            .expect("Failed to load texture")
            .decode()
            .expect("Failed to decode texture");

        Self { image }
    }

    pub fn new_rgba(width: u32, height: u32) -> Self {
        let image_buffer = RgbaImage::new(width, height);
        let image = DynamicImage::ImageRgba8(image_buffer);

        Self { image }
    }

    pub fn new_rgb(width: u32, height: u32) -> Self {
        let image_buffer = RgbImage::new(width, height);
        let image = DynamicImage::ImageRgb8(image_buffer);

        Self { image }
    }

    pub fn flood_fill(&mut self, x: i32, y: i32, color: Rgba<u8>, wrap_x: bool, wrap_y: bool) {
        let start_color = self.image.get_pixel(x as u32, y as u32);
        self.image.put_pixel(x as u32, y as u32, color);
        let mut to_color = vec![(x, y)];

        while let Some((x, y)) = to_color.pop() {
            if self.image.get_pixel(x as u32, y as u32) == start_color {
                self.image.put_pixel(x as u32, y as u32, color);

                self.add_for_fill(&mut to_color, x + 1, y, start_color, wrap_x, wrap_y);
                self.add_for_fill(&mut to_color, x, y + 1, start_color, wrap_x, wrap_y);
                self.add_for_fill(&mut to_color, x - 1, y, start_color, wrap_x, wrap_y);
                self.add_for_fill(&mut to_color, x, y - 1, start_color, wrap_x, wrap_y);
            }
        }
    }

    fn add_for_fill(
        &self,
        to_color: &mut Vec<(i32, i32)>,
        mut x: i32,
        mut y: i32,
        start_color: Rgba<u8>,
        wrap_x: bool,
        wrap_y: bool,
    ) {
        if wrap_x {
            x %= self.image.width() as i32;
        }

        if wrap_y {
            y %= self.image.height() as i32;
        }

        if x > 0
            && y > 0
            && (x as u32) < self.image.width()
            && (y as u32) < self.image.height()
            && self.image.get_pixel(x as u32, y as u32) == start_color
        {
            to_color.push((x, y));
        }
    }
}
