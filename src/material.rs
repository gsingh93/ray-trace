use std::f32;

use Vec3;
use texture::Texture;
use ray::{Intersection, Ray};

use noise::{Fbm, MultiFractal, NoiseFn, Seedable};

pub struct Material {
    color: Vec3,
    diffuse_coeff: f32,
    specular_coeff: f32,
    glossiness: f32,
    reflectivity: f32,
    texture: Option<Box<dyn Texture>>,
    normal_map: Option<NormalMap>,
    displacement_map: Option<DisplacementMap>,
}

impl Clone for Material {
    fn clone(&self) -> Material {
        Material {
            color: self.color,
            diffuse_coeff: self.diffuse_coeff,
            specular_coeff: self.specular_coeff,
            glossiness: self.glossiness,
            reflectivity: self.reflectivity,
            texture: self.texture.as_ref().map(|t| t.clone_()),
            normal_map: self.normal_map.as_ref().map(|m| m.clone()),
            displacement_map: self.displacement_map.as_ref().map(|m| m.clone()),
        }
    }
}

impl Material {
    pub fn new(color: Vec3, diffuse_coeff: f32, specular_coeff: f32, glossiness: f32,
               reflectivity: f32, texture: Option<Box<dyn Texture>>,
               normal_map: Option<NormalMap>, displacement_map: Option<DisplacementMap>) -> Self {
        Material { color: color, diffuse_coeff: diffuse_coeff,
                   specular_coeff: specular_coeff, glossiness: glossiness,
                   reflectivity: reflectivity, texture: texture, normal_map: normal_map,
                   displacement_map: displacement_map }
    }

    pub fn reflectivity(&self) -> f32 {
        self.reflectivity
    }

    pub fn raw_color(&self) -> Vec3 {
        self.color
    }

    pub fn color(&self, shadow_ray: &Ray, camera_ray: &Ray, hit: &Intersection) -> Vec3 {
        let f = f32::max(0., hit.normal.dot(&shadow_ray.dir));
        let diffuse_color = (self.color * f * self.diffuse_coeff).component_mul(&match self.texture {
            Some(ref t) => t.color(hit.u, hit.v) / 255.,
            None => Vec3::new(1., 1., 1.)
        });

        // Average the angles, flipping the camera ray because it's in the opposite direction
        let half_vec = ((shadow_ray.dir - camera_ray.dir) / 2.).normalize();
        let f = f32::max(0., half_vec.dot(&hit.normal)).powf(self.glossiness);
        // TODO: Specular default color
        let specular_color = Vec3::new(255., 255., 255.) * f * self.specular_coeff;

        diffuse_color + specular_color
    }

    pub fn has_normal_map(&self) -> bool {
        self.normal_map.is_some()
    }

    pub fn has_displacement_map(&self) -> bool {
        self.displacement_map.is_some()
    }

    pub fn apply_normal_map(&self, normal: &Vec3, hit_pos: &Vec3) -> Vec3 {
        match &self.normal_map {
            &Some(ref map) => map.map(normal, hit_pos),
            &None => *normal
        }
    }

    pub fn apply_displacement_map(&self, hit_pos: &Vec3) -> Vec3 {
        match &self.displacement_map {
            &Some(ref map) => map.map(hit_pos),
            &None => *hit_pos
        }
    }
}

pub struct NormalMap {
    seed_val: u32,
    octaves: usize,
    wavelength: f32,
    persistence: f32,
    lacunarity: f32,
}

impl Clone for NormalMap {
    fn clone(&self) -> Self {
        NormalMap { seed_val: self.seed_val, octaves: self.octaves,
                    wavelength: self.wavelength,
                    persistence: self.persistence, lacunarity: self.lacunarity }
    }
}

impl NormalMap {
    pub fn new(seed_val: u32, octaves: usize, wavelength: f32, persistence: f32,
               lacunarity: f32) -> Self {
        NormalMap { seed_val: seed_val, octaves: octaves, wavelength: wavelength,
                    persistence: persistence, lacunarity: lacunarity }
    }

    fn map(&self, normal: &Vec3, hit_pos: &Vec3) -> Vec3 {
        let noise = Fbm::new()
            .set_octaves(self.octaves)
            .set_seed(self.seed_val)
            .set_frequency(self.wavelength as f64) // TODO: frequency, not wavelength
            .set_persistence(self.persistence as f64) // TODO: f64
            .set_lacunarity(self.lacunarity as f64); // TODO: f64

        let mut val = noise.get([hit_pos.x as f64, hit_pos.y as f64, hit_pos.z as f64]) + 1.0;
        val = val / 2.;

        if val < 0. {
            val = 0.
        }
        (*normal + Vec3::new(val as f32, val as f32, val as f32)).normalize()
    }
}

pub struct DisplacementMap {
    seed_val: u32,
    octaves: usize,
    wavelength: f32,
    persistence: f32,
    lacunarity: f32,
}

impl Clone for DisplacementMap {
    fn clone(&self) -> Self {
        DisplacementMap { seed_val: self.seed_val, octaves: self.octaves,
                          wavelength: self.wavelength,
                          persistence: self.persistence, lacunarity: self.lacunarity }
    }
}

impl DisplacementMap {
    pub fn new(seed_val: u32, octaves: usize, wavelength: f32, persistence: f32,
               lacunarity: f32) -> Self {
        DisplacementMap { seed_val: seed_val, octaves: octaves, wavelength: wavelength,
                          persistence: persistence, lacunarity: lacunarity }
    }

    fn map(&self, hit_pos: &Vec3) -> Vec3 {
        let noise = Fbm::new()
            .set_octaves(self.octaves)
            .set_seed(self.seed_val)
            .set_frequency(self.wavelength as f64) // TODO: frequency, not wavelength
            .set_persistence(self.persistence as f64) // TODO: f64
            .set_lacunarity(self.lacunarity as f64); // TODO: f64

        let mut val = noise.get([hit_pos.x as f64, hit_pos.y as f64, hit_pos.z as f64]) + 1.0;
        //let mut val = val / 2.;

        if val < 0. {
            val = 0.
        }
        *hit_pos + Vec3::new(val as f32, val as f32, val as f32)
    }
}
