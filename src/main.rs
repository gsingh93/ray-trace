extern crate image;
extern crate nalgebra;

use image::{ImageBuffer, Rgb, Pixel};

use nalgebra::{cross, dot, normalize, Norm};

type Vec3 = nalgebra::Vec3<f32>;

const OUT_FILE: &'static str = "image.png";
const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

struct Ray {
    origin: Vec3,
    dir: Vec3,
}

impl Ray {
    fn new(origin: Vec3, dir: Vec3) -> Self {
        Ray { origin: origin, dir: normalize(&dir) }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct Intersection {
    pos: Vec3,
}

impl Intersection {
    fn new(pos: Vec3) -> Self {
        Intersection { pos: pos }
    }
}

trait Surface {
    fn intersect(&self, ray: &Ray) -> Option<Intersection>;
}

struct Sphere {
    pos: Vec3,
    radius: f32,
}

impl Surface for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let center_offset = self.pos - ray.origin;
        let b = 2. * dot(&ray.dir, &center_offset);
        let c = center_offset.sqnorm() - self.radius * self.radius;

        let discriminant = b * b - 4. * c;

        if discriminant > 0. {
            // TODO: What about the other solution?
            let d = -b - discriminant.sqrt();
            let pos = ray.origin + ray.dir * d;
            Some(Intersection::new(pos))
        } else {
            None
        }
    }
}

impl Sphere {
    fn new(pos: Vec3, radius: f32) -> Self {
        Sphere { pos: pos, radius: radius }
    }
}

struct Camera {
    pos: Vec3,
    dir: Vec3,
    up: Vec3,
    right: Vec3,
}

impl Camera {
    fn new(pos: Vec3, dir: Vec3, up: Vec3) -> Self {
        let right = normalize(&cross(&up, &dir));
        let up = normalize(&cross(&right, &dir));
        Camera { pos: pos, dir: normalize(&dir), up: up, right: right }
    }

    fn from_lookat(pos: Vec3, lookat: Vec3, up: Vec3) -> Self {
        let dir = lookat - pos;
        Camera::new(pos, dir, up)
    }

    fn get_ray(&self, x: u32, y: u32) -> Ray {
        let norm_x = (x as f32 / WIDTH as f32) - 0.5;
        let norm_y = (y as f32 / HEIGHT as f32) - 0.5;

        let dir = self.right * norm_x + self.up * norm_y + self.dir;
        Ray::new(self.pos, dir)
    }
}

fn main() {
    let sphere_color: Rgb<u8> = Rgb::from_channels(255, 0, 0, 1);
    let mut im: ImageBuffer<Rgb<u8>, _> = ImageBuffer::new(WIDTH, HEIGHT);

    let camera = {
        let pos = Vec3::new(0., 0., -4.);
        let lookat = Vec3::new(0., 0., 0.);
        let up = Vec3::new(0., 1., 0.);
        Camera::from_lookat(pos, lookat, up)
    };
    let sphere = Sphere::new(Vec3::new(0., 0., 0.), 1.);

    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            let ray = camera.get_ray(x, y);
            if let Some(_) = sphere.intersect(&ray) {
                im.put_pixel(x, y, sphere_color);
            }
        }
    }

    im.save(OUT_FILE).unwrap();
}
