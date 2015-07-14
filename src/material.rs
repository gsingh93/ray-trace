use std::f32;

use {Intersection, Ray, Vec3};
use texture::Texture;

use nalgebra::{dot, Norm};

pub struct Material {
    color: Vec3,
    diffuse_coeff: f32,
    specular_coeff: f32,
    glossiness: f32,
    reflectivity: f32,
    texture: Option<Box<Texture>>,
}

impl Material {
    pub fn new(color: Vec3, diffuse_coeff: f32, specular_coeff: f32, glossiness: f32,
               reflectivity: f32, texture: Option<Box<Texture>>) -> Self {
        Material { color: color, diffuse_coeff: diffuse_coeff,
                   specular_coeff: specular_coeff, glossiness: glossiness,
                   reflectivity: reflectivity, texture: texture }
    }

    pub fn reflectivity(&self) -> f32 {
        self.reflectivity
    }

    pub fn raw_color(&self) -> Vec3 {
        self.color
    }

    pub fn color(&self, shadow_ray: &Ray, camera_ray: &Ray, hit: &Intersection) -> Vec3 {
        let f = f32::max(0., dot(&hit.normal, &shadow_ray.dir));
        let diffuse_color = self.color * f * self.diffuse_coeff * match self.texture {
            Some(ref t) => t.color(hit.u, hit.v) / 255.,
            None => Vec3::new(1., 1., 1.)
        };


        // Average the angles, flipping the camera ray because it's in the opposite direction
        let half_vec = ((shadow_ray.dir - camera_ray.dir) / 2.).normalize();
        let f = f32::max(0., dot(&half_vec, &hit.normal)).powf(self.glossiness);
        // TODO: Specular default color
        let specular_color = Vec3::new(255., 255., 255.) * f * self.specular_coeff;

        diffuse_color + specular_color
    }
}
