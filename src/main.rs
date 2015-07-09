extern crate image;
extern crate nalgebra;

use std::fs::File;

use image::ColorType;
use image::png::PNGEncoder;

type Vec3 = nalgebra::Vec3<f32>;

const OUT_FILE: &'static str = "image.png";
const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

struct Ray {
    origin: Vec3,
    dir: Vec3,
}

struct Intersection {
    pos: Vec3,
}

trait Surface {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        unimplemented!()
    }
}

struct Sphere {
    pos: Vec3,
    radius: f32,
}

impl Surface for Sphere {}

impl Sphere {
    fn new(pos: Vec3, radius: f32) -> Self {
        Sphere { pos: pos, radius: radius }
    }
}

struct Camera {
    pos: Vec3,
    dir: Vec3,
}

impl Camera {
    fn new(pos: Vec3, dir: Vec3) -> Self {
        Camera { pos: pos, dir: dir }
    }

    fn from_lookat(pos: Vec3, lookat: Vec3) -> Self {
        Camera { pos: pos, dir: lookat - pos }
    }

    fn get_ray(&self, x: u32, y: u32) -> Ray {
        unimplemented!()
    }
}

fn main() {
    const BUF_SIZE: usize = (WIDTH * HEIGHT) as usize;

    let mut f = File::create(OUT_FILE).unwrap();
    let mut encoder = PNGEncoder::new(&mut f);
    let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];

    let camera = Camera::from_lookat(Vec3::new(0., 0., -2.), Vec3::new(0., 0., 0.));
    let sphere = Sphere::new(Vec3::new(0., 0., 0.), 1.);

    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            let ray = camera.get_ray(x, y);
            if let Some(intersection) = sphere.intersect(&ray) {
                let idx = (y * WIDTH + x) as usize;
                buf[idx] = 0;
            }
        }
    }
    encoder.encode(&buf, WIDTH, HEIGHT, ColorType::RGB(24)).unwrap();
}
