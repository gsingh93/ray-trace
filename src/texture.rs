use Vec3;

use image::{self, DynamicImage, RgbImage};

pub trait Texture {
    fn color(&self, u: f32, v: f32) -> Vec3;
    fn clone_(&self) -> Box<dyn Texture>;
}

#[derive(Clone)]
pub struct CheckerboardTexture {
    pub dim: f32,
}

impl CheckerboardTexture {
    pub fn new(dim: f32) -> Self {
        CheckerboardTexture { dim: dim }
    }
}

impl Texture for CheckerboardTexture {
    fn color(&self, u: f32, v: f32) -> Vec3 {
        let half = self.dim / 2.;

        let mut s = u % self.dim;
        let mut t = v % self.dim;
        if s > 0. {
            s -= half;
        } else {
            s += half;
        }

        if t > 0. {
            t -= half;
        } else {
            t += half;
        }

        let color1 = Vec3::new(0., 0., 0.);
        let color2 = Vec3::new(255., 255., 255.);

        if s > 0. && t < 0. || s < 0. && t > 0. {
            color1
        } else {
            color2
        }
    }

    fn clone_(&self) -> Box<dyn Texture> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct ImageTexture {
    image: RgbImage,
}

impl ImageTexture {
    pub fn new(filename: &str) -> Self {
        let image = image::open(filename).unwrap();
        if let DynamicImage::ImageRgb8(im) = image {
            ImageTexture { image: im }
        } else {
            panic!("Only RGB textures are supported");
        }
    }
}

impl Texture for ImageTexture {
    fn color(&self, u: f32, v: f32) -> Vec3 {
        let u = (u % 1.) * (self.image.width() as f32 - 1.);
        let v = (v % 1.) * (self.image.height() as f32 - 1.);

        // TODO: Bilinear sampling
        let &image::Rgb([r, g, b]) = self.image.get_pixel(u as u32, v as u32);
        Vec3::new(r as f32, g as f32, b as f32)
    }

    fn clone_(&self) -> Box<dyn Texture> {
        Box::new(self.clone())
    }
}
