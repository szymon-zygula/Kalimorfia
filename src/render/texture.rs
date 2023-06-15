use std::ops::RangeInclusive;

use image::{DynamicImage, GenericImage, GenericImageView, RgbImage, Rgba, RgbaImage};
use itertools::Itertools;
use nalgebra::{vector, Vector2};

use crate::math::geometry::{
    intersection::Intersection, parametric_form::DifferentialParametricForm,
};

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

    pub fn empty_intersection(resolution: u32) -> Self {
        let mut texture = Self::new_rgba(resolution, resolution);
        texture.fill(Rgba([0, 0, 255, 255]));
        texture
    }

    pub fn intersection_texture(
        intersection: &Intersection,
        surface_0: &dyn DifferentialParametricForm<2, 3>,
        surface_1: &dyn DifferentialParametricForm<2, 3>,
        resolution: u32,
    ) -> [Texture; 2] {
        let surface_0_texture = Self::surface_intersection_texture(
            &intersection
                .points
                .iter()
                .map(|p| p.surface_0)
                .collect::<Vec<_>>(),
            surface_0,
            intersection.looped,
            resolution,
        );

        let surface_1_texture = Self::surface_intersection_texture(
            &intersection
                .points
                .iter()
                .map(|p| p.surface_1)
                .collect::<Vec<_>>(),
            surface_1,
            intersection.looped,
            resolution,
        );

        [surface_0_texture, surface_1_texture]
    }

    fn surface_intersection_texture(
        points: &[Vector2<f64>],
        surface: &dyn DifferentialParametricForm<2, 3>,
        looped: bool,
        resolution: u32,
    ) -> Self {
        let mut texture = Self::empty_intersection(resolution);
        let bounds = surface.bounds();
        let ranges = bounds.map(|b| b.1 - b.0);

        for (pt_0, pt_1) in points.iter().tuple_windows() {
            let pt_0_x = (pt_0.x + bounds.x.0) / ranges.x;
            let pt_0_y = (pt_0.y + bounds.y.0) / ranges.y;
            let pt_0 = vector![pt_0_x, pt_0_y];

            let pt_1_x = (pt_1.x + bounds.x.0) / ranges.x;
            let pt_1_y = (pt_1.y + bounds.y.0) / ranges.y;
            let pt_1 = vector![pt_1_x, pt_1_y];

            texture.wrapped_line(&pt_0, &pt_1, surface.wrapped(0), surface.wrapped(1));
        }

        if looped {
            let pt_0_x = (points[0].x + bounds.x.0) / ranges.x;
            let pt_0_y = (points[0].y + bounds.y.0) / ranges.y;
            let pt_0 = vector![pt_0_x, pt_0_y];

            let pt_1_x = (points[points.len() - 1].x + bounds.x.0) / ranges.x;
            let pt_1_y = (points[points.len() - 1].y + bounds.y.0) / ranges.y;
            let pt_1 = vector![pt_1_x, pt_1_y];

            texture.wrapped_line(&pt_0, &pt_1, surface.wrapped(0), surface.wrapped(1));
        }

        texture
    }

    pub fn flood_fill_inv(&mut self, x: i32, y: i32, wrap_x: bool, wrap_y: bool) {
        if self.image.get_pixel(x as u32, y as u32) == Rgba([0, 0, 255, 255]) {
            self.flood_fill(x, y, Rgba([255, 0, 0, 255]), wrap_x, wrap_y);
        } else if self.image.get_pixel(x as u32, y as u32) == Rgba([255, 0, 0, 255]) {
            self.flood_fill(x, y, Rgba([0, 0, 255, 255]), wrap_x, wrap_y);
        }
    }

    pub fn fill(&mut self, color: Rgba<u8>) {
        for (x, y) in Itertools::cartesian_product(0..self.image.width(), 0..self.image.height()) {
            self.image.put_pixel(x, y, color);
        }
    }

    pub fn flood_fill(&mut self, x: i32, y: i32, color: Rgba<u8>, wrap_x: bool, wrap_y: bool) {
        let start_color = self.image.get_pixel(x as u32, y as u32);
        let mut to_color = vec![(x, y)];
        to_color.reserve(self.image.height() as usize * self.image.width() as usize);

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
            x = x.rem_euclid(self.image.width() as i32);
        }

        if wrap_y {
            y = y.rem_euclid(self.image.height() as i32);
        }

        if self.image.in_bounds(x as u32, y as u32)
            && self.image.get_pixel(x as u32, y as u32) == start_color
        {
            to_color.push((x, y));
        }
    }

    pub fn normal_to_img(&self, pt: &Vector2<f64>) -> Vector2<f64> {
        vector![
            pt.x * self.image.width() as f64,
            pt.y * self.image.height() as f64
        ]
    }

    /// Points are in range [0, 1]
    pub fn line(&mut self, pt_0: &Vector2<f64>, pt_1: &Vector2<f64>) {
        // This algorithm is slow and stupid but simple to implement

        let pt_0_img = self.normal_to_img(pt_0);
        let pt_1_img = self.normal_to_img(pt_1);

        let distance = Vector2::metric_distance(&pt_0_img, &pt_1_img);
        let x_diff = (pt_1_img.x - pt_0_img.x) / distance / 2.0;
        let y_diff = (pt_1_img.y - pt_0_img.y) / distance / 2.0;

        let mut current = pt_0_img;
        for _ in 0..=((distance * 2.0).round() as u32) {
            let x = current.x.floor() as u32 % self.image.width();
            let y = current.y.floor() as u32 % self.image.height();

            self.image.put_pixel(x, y, Rgba([0, 255, 0, 255]));

            let x = current.x.ceil() as u32 % self.image.width();
            let y = current.y.ceil() as u32 % self.image.height();

            self.image.put_pixel(x, y, Rgba([0, 255, 0, 255]));

            current.x += x_diff;
            current.y += y_diff;
        }
    }

    pub fn wrapped_line(
        &mut self,
        pt_0: &Vector2<f64>,
        pt_1: &Vector2<f64>,
        wrap_x: bool,
        wrap_y: bool,
    ) {
        let x_range = Self::wrap_range(wrap_x);
        let y_range = Self::wrap_range(wrap_y);

        let best_pt1 = x_range
            .cartesian_product(y_range)
            .map(|(off_x, off_y)| pt_1 + vector![off_x as f64, off_y as f64])
            .min_by(|a, b| {
                Vector2::metric_distance(pt_0, a).total_cmp(&Vector2::metric_distance(pt_0, b))
            })
            .unwrap();

        self.line(pt_0, &best_pt1);
    }

    fn wrap_range(wrap: bool) -> RangeInclusive<i32> {
        if wrap {
            -1..=1
        } else {
            0..=0
        }
    }

    pub fn width(&self) -> f32 {
        self.image.width() as f32
    }

    pub fn height(&self) -> f32 {
        self.image.height() as f32
    }
}
