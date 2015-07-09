extern crate image;
extern crate nalgebra;

use std::fs::File;

use image::ColorType;
use image::png::PNGEncoder;

use nalgebra::{cross, dot, normalize};

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
        Ray { origin: origin, dir: dir }
    }
}

#[allow(dead_code)]
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
        let dist = ray.origin - self.pos;
        let determinant = dot(&ray.dir, &dist).powi(2) - dot(&dist, &dist) + self.radius.powi(2);
        if determinant > 0. {
            // FIXME: Is this always the right solution to take?
            let d = -dot(&ray.dir, &dist) - determinant;
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
        let norm_x = (x / WIDTH) as f32 - 0.5;
        let norm_y = (y / HEIGHT) as f32 - 0.5;

        let dir = self.right * norm_x + self.up * norm_y + self.dir;
        Ray::new(self.pos, dir)
    }
}

fn main() {
    const BUF_SIZE: usize = (WIDTH * HEIGHT) as usize;

    let mut f = File::create(OUT_FILE).unwrap();
    let mut encoder = PNGEncoder::new(&mut f);
    let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];

    let camera = {
        let pos = Vec3::new(0., 0., -2.);
        let dir = Vec3::new(0., 0., 0.);
        let up = Vec3::new(0., 1., 0.);
        Camera::from_lookat(pos, dir, up)
    };
    let sphere = Sphere::new(Vec3::new(0., 0., 0.), 1.);

    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            let ray = camera.get_ray(x, y);
            if let Some(_) = sphere.intersect(&ray) {
                let idx = (y * WIDTH + x) as usize;
                buf[idx] = 100;
            }
        }
    }
    encoder.encode(&buf, WIDTH, HEIGHT, ColorType::RGB(8)).unwrap();
}
