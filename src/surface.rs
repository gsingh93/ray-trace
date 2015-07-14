use std::f32;

use {Intersection, Ray, Vec3};

use nalgebra::{dot, cross, Norm};

pub trait Surface {
    fn intersect(&self, &Ray) -> Option<Intersection>;
    fn material(&self) -> &Material;
    // For debugging
    fn name(&self) -> &'static str;
}

pub struct Sphere {
    pos: Vec3,
    radius: f32,
    material: Material,
}

impl Sphere {
    pub fn new(pos: Vec3, radius: f32, material: Material) -> Self {
        Sphere { pos: pos, radius: radius, material: material }
    }
}

impl Surface for Sphere {
    fn name(&self) -> &'static str {
        "Sphere"
    }

    fn material(&self) -> &Material {
        &self.material
    }

    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let center_offset = ray.origin - self.pos;
        let b = 2. * dot(&ray.dir, &center_offset);
        let c = center_offset.sqnorm() - self.radius * self.radius;

        let discriminant = b * b - 4. * c;

        if discriminant >= 0. {
            let disc_sqrt = discriminant.sqrt();
            let d1 = 0.5 * (-b + disc_sqrt);
            let d2 = 0.5 * (-b - disc_sqrt);

            // d1 should always be larger than d2, we want the smallest positive distance
            let d = if d2 > 0. {
                d2
            } else if d1 > 0. {
                // We are inside the sphere
                d1
            } else {
                // Both are negative, sphere is behind camera
                return None
            };

            let pos = ray.origin + ray.dir * d;
            let normal = (pos - self.pos).normalize();
            let center_vec = (self.pos - pos).normalize();
            let u = 0.5 + center_vec.z.atan2(center_vec.x) / (2. * f32::consts::PI);
            let v = 0.5 - center_vec.y.atan() / f32::consts::PI;

            // println!("u: {}", u);
            // println!("v: {}", v);

            Some(Intersection::new(pos, normal, d, u, v))
        } else {
            None
        }
    }
}

pub struct Plane {
    point: Vec3,
    normal: Vec3,
    material: Material,
}

impl Plane {
    pub fn new(point: Vec3, normal: Vec3, material: Material) -> Self {
        Plane { point: point, normal: normal, material: material }
    }
}

impl Surface for Plane {
    fn name(&self) -> &'static str {
        "Plane"
    }

    fn material(&self) -> &Material {
        &self.material
    }

    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let denom = dot(&ray.dir, &self.normal);
        if denom == 0. {
            return None;
        }
        let d = dot(&self.normal, &(self.point - ray.origin)) / denom;
        if d > 0. {
            let pos = ray.origin + ray.dir * d;

            let n = &self.normal;
            let u_axis = Vec3 { x: n.y, y: n.z, z: -n.x };
            let v_axis = cross(&u_axis, n);
            //println!("v_axis: {:?}", v_axis);
            let u = dot(&pos, &u_axis);
            let v = dot(&pos, &v_axis);

            // println!("u: {}", u);
            // println!("v: {}", v);

            Some(Intersection::new(pos, self.normal, d, u, v))
        } else {
            None
        }
    }
}

pub trait Texture {
    fn color(&self, u: f32, v: f32) -> Vec3;
}

pub struct CheckerboardTexture {
    pub dim: f32,
}

impl Texture for CheckerboardTexture {
    fn color(&self, u: f32, v: f32) -> Vec3 {
        let half = self.dim / 2.0;

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
}

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
