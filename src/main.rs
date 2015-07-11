extern crate image;
extern crate nalgebra;

mod surface;

use std::f32;

use surface::{Plane, Sphere, Surface};

use image::{ImageBuffer, Rgb, Pixel};

use nalgebra::{clamp, cross, dot, Norm};

pub type Vec3 = nalgebra::Vec3<f32>;

const OUT_FILE: &'static str = "image.png";
const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

#[derive(Debug)]
pub struct Ray {
    origin: Vec3,
    dir: Vec3,
}

impl Ray {
    fn new(origin: Vec3, dir: Vec3) -> Self {
        Ray { origin: origin, dir: dir.normalize() }
    }
}

#[derive(Clone, Debug)]
pub struct Intersection {
    pos: Vec3,
    normal: Vec3,
    dist: f32,
}

impl Intersection {
    fn new(pos: Vec3, normal: Vec3, dist: f32) -> Self {
        Intersection { pos: pos, normal: normal, dist: dist }
    }
}


trait Material {
    fn color(&self, shadow_ray: &Ray, camera_ray: &Ray, &Intersection) -> Vec3;
}

struct SphereMaterial {
    color: Vec3,
    diffuse_coeff: f32,
    specular_coeff: f32,
}

impl SphereMaterial {
    fn new(color: Vec3, diffuse_coeff: f32, specular_coeff: f32) -> Self {
        SphereMaterial { color: color, diffuse_coeff: diffuse_coeff,
                         specular_coeff: specular_coeff }
    }
}

impl Material for SphereMaterial {
    fn color(&self, shadow_ray: &Ray, camera_ray: &Ray, hit: &Intersection) -> Vec3 {
        let f = f32::max(0., dot(&hit.normal, &shadow_ray.dir));
        let diffuse_color = self.color * f * self.diffuse_coeff;

        let half_vec = ((shadow_ray.dir + camera_ray.dir) / 2.).normalize();
        let f = f32::max(0., dot(&half_vec, &hit.normal)).powi(10); // TODO
        // TODO
        let specular_color = Vec3::new(255., 255., 255.) * f * self.specular_coeff;

        diffuse_color + specular_color
    }
}

struct Camera {
    pos: Vec3,
    dir: Vec3,
    up: Vec3,
    right: Vec3,
}

struct PointLight {
    pos: Vec3,
    color: Vec3,
    intensity: f32,
}

impl PointLight {
    fn new(pos: Vec3, color: Vec3, intensity: f32) -> Self {
        PointLight { pos: pos, color: color, intensity: intensity }
    }
}

struct Scene {
    objects: Vec<Box<Surface>>,
    lights: Vec<PointLight>,
    ambient_coeff: f32,
    ambient_color: Vec3,
    camera: Camera,
}

impl Scene {
    fn new(objects: Vec<Box<Surface>>,
           lights: Vec<PointLight>,
           ambient_coeff: f32,
           ambient_color: Vec3,
           camera: Camera) -> Self {
        Scene {
            objects: objects,
            lights: lights,
            ambient_coeff: ambient_coeff,
            ambient_color: ambient_color,
            camera: camera,
        }
    }

    fn intersect(&self, ray: &Ray) -> Option<(&Box<Surface>, Intersection)> {
        let mut result = None;
        for obj in self.objects.iter() {
            if let Some(hit) = obj.intersect(ray) {
                match result.clone() {
                    None => result = Some((obj, hit)),
                    Some((_, ref old_hit)) =>
                        if hit.dist < old_hit.dist { result = Some((obj, hit)) }
                }
            }
        }
        result
    }
}

impl Camera {
    fn new(pos: Vec3, dir: Vec3, up: Vec3) -> Self {
        let right = cross(&up, &dir).normalize();
        let up = cross(&right, &dir).normalize();
        Camera { pos: pos, dir: dir.normalize(), up: up, right: right }
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
    let mut im: ImageBuffer<Rgb<u8>, _> = ImageBuffer::new(WIDTH, HEIGHT);

    let camera = {
        let pos = Vec3::new(0., 1., -4.);
        let lookat = Vec3::new(0., 0., 0.);
        let up = Vec3::new(0., 1., 0.);
        Camera::from_lookat(pos, lookat, up)
    };
    let plane = Plane::new(Vec3::new(0., 0., 0.), Vec3::new(0., 1., 0.));
    let sphere = Sphere::new(Vec3::new(0., 0., 0.), 1.);
    let material = SphereMaterial::new(Vec3::new(0., 0., 255.), 0.7, 0.);
    let light = PointLight::new(Vec3::new(4., 4., 0.), Vec3::new(0., 255., 0.), 2.);

    let scene = Scene::new(vec![Box::new(sphere), Box::new(plane)],
                           vec![light],
                           0.1, Vec3::new(255., 255., 255.), camera);

    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            let ray = scene.camera.get_ray(x, y);
            if let Some((_, hit)) = scene.intersect(&ray) {
                // Ambient
                let mut color = material.color * scene.ambient_coeff;
                for light in scene.lights.iter() {
                    let pos = hit.pos + hit.normal * f32::EPSILON.sqrt();
                    let shadow_ray = Ray::new(pos, light.pos - pos);
                    if scene.intersect(&shadow_ray).is_none() {
                        color = color + material.color(&shadow_ray, &ray, &hit) * light.intensity;
                    }
                }
                let color = Rgb::from_channels(clamp(color.x, 0., 255.) as u8,
                                               clamp(color.y, 0., 255.) as u8,
                                               clamp(color.z, 0., 255.) as u8,
                                               255);
                im.put_pixel(x, y, color);
            }
        }
    }

    im.save(OUT_FILE).unwrap();
}
