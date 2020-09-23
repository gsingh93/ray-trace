extern crate image;
extern crate nalgebra;
extern crate noise;

pub mod light;
pub mod material;
mod ray;
pub mod surface;
pub mod texture;

use std::f32;

use light::PointLight;
use ray::{Intersection, Ray};
use surface::Surface;

use image::{RgbImage, Rgb, Pixel};

use nalgebra::clamp;

pub type Vec3 = nalgebra::Vector3<f32>;

#[derive(Debug)]
pub struct Camera {
    pos: Vec3,
    dir: Vec3,
    up: Vec3,
    right: Vec3,
}

impl Camera {
    pub fn new(pos: Vec3, dir: Vec3, up: Vec3) -> Self {
        let right = up.cross(&dir).normalize();
        let up = right.cross(&dir).normalize();
        Camera { pos: pos, dir: dir.normalize(), up: up, right: right }
    }

    pub fn from_lookat(pos: Vec3, lookat: Vec3, up: Vec3) -> Self {
        let dir = lookat - pos;
        Camera::new(pos, dir, up)
    }

    fn get_ray(&self, x: u32, y: u32, width: u32, height: u32, aspect_ratio: f32) -> Ray {
        let norm_x = (x as f32 / width as f32) - 0.5;
        let norm_y = (y as f32 / height as f32) - 0.5;
        let norm_x = norm_x * aspect_ratio;

        let dir = self.right * norm_x + self.up * norm_y + self.dir;
        Ray::new(self.pos, dir)
    }
}

pub struct Scene {
    objects: Vec<Box<dyn Surface>>,
    lights: Vec<PointLight>,
    ambient_coeff: f32,
    ambient_color: Vec3,
    camera: Camera,
}

impl Scene {
    pub fn new(objects: Vec<Box<dyn Surface>>,
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

    fn intersect(&self, ray: &Ray) -> Option<(&Box<dyn Surface>, Intersection)> {
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

pub fn ray_trace(scene: &Scene, width: u32, height: u32, max_depth: u16) -> RgbImage {
    let aspect_ratio = width as f32 / height as f32;

    let mut im: RgbImage = RgbImage::new(width, height);
    for x in 0..width {
        for y in 0..height {
            let ray = scene.camera.get_ray(x, y, width, height, aspect_ratio);
            let color = trace_ray(&scene, &ray, 0, max_depth);

            let color = Rgb::from_channels(clamp(color.x, 0., 255.) as u8,
                                           clamp(color.y, 0., 255.) as u8,
                                           clamp(color.z, 0., 255.) as u8,
                                           255);
            im.put_pixel(x, y, color);
        }
    }
    im
}

fn trace_ray(scene: &Scene, ray: &Ray, depth: u16, max_depth: u16) -> Vec3 {
    let mut color = Vec3::new(0., 0., 0.); // TODO: Background color
    if let Some((obj, hit)) = scene.intersect(ray) {
        let material = obj.material();

        // Ambient color
        color = material.raw_color().component_mul(&((scene.ambient_color / 255.) * scene.ambient_coeff));

        // Trace shadow rays
        for light in scene.lights.iter() {
            let pos = hit.pos + hit.normal * f32::EPSILON.sqrt();
            let dir = *light.pos() - pos;
            let dist = dir.norm();
            let shadow_ray = Ray::new(pos, dir);
            if let Some((_, shadow_hit)) = scene.intersect(&shadow_ray) {
                if shadow_hit.dist > dist {
                    // Diffuse/specular color
                    color = color + material.color(&shadow_ray, &ray, &hit).component_mul(
                        &((*light.color() / 255.) * light.intensity()));
                }
            } else {
                // Diffuse/specular color
                color = color + material.color(&shadow_ray, &ray, &hit).component_mul(
                    &((*light.color() / 255.) * light.intensity()));
            }
        }

        if depth >= max_depth {
            return color;
        }

        // Get reflected color
        let reflectivity = material.reflectivity();
        if reflectivity > 0. {
            let reflected_ray = reflected_ray(ray, &hit);
            let reflected_color = trace_ray(scene, &reflected_ray, depth + 1, max_depth);
            color = color + reflected_color * reflectivity;
        }
    }
    color
}

fn reflected_ray(ray: &Ray, hit: &Intersection) -> Ray {
    let pos = hit.pos + hit.normal * f32::EPSILON.sqrt();
    let dir = ray.dir - hit.normal * 2. * ray.dir.dot(&hit.normal);
    Ray::new(pos, dir)
}
