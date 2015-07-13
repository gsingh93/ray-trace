use {Intersection, Ray, Vec3};

use nalgebra::{dot, Norm};

pub trait Surface<M> {
    fn intersect(&self, &Ray) -> Option<Intersection>;
    fn material(&self) -> &M;
    // For debugging
    fn name(&self) -> &'static str;
}

#[derive(Clone, Debug)]
pub struct Sphere<M> {
    pos: Vec3,
    radius: f32,
    material: M,
}

impl<M: Material> Sphere<M> {
    pub fn new(pos: Vec3, radius: f32, material: M) -> Self {
        Sphere { pos: pos, radius: radius, material: material }
    }
}


impl<M: Material> Surface<M> for Sphere<M> {
    fn name(&self) -> &'static str {
        "Sphere"
    }

    fn material(&self) -> &M {
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
            Some(Intersection::new(pos, normal, d))
        } else {
            None
        }
    }
}

pub struct Plane<M: Material> {
    point: Vec3,
    normal: Vec3,
    material: M,
}

impl<M: Material> Plane<M> {
    pub fn new(point: Vec3, normal: Vec3, material: M) -> Self {
        Plane { point: point, normal: normal, material: material }
    }
}

impl<M: Material> Surface<M> for Plane<M> {
    fn name(&self) -> &'static str {
        "Plane"
    }

    fn material(&self) -> &M {
        &self.material
    }

    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let denom = dot(&ray.dir, &self.normal);
        if denom == 0. {
            return None;
        }
        let d = dot(&self.normal, &(self.point - ray.origin)) / denom;
        if d > 0. {
            Some(Intersection::new(ray.origin + ray.dir * d, self.normal, d))
        } else {
            None
        }
    }
}

pub trait Material {
    fn raw_color(&self) -> Vec3;
    fn color(&self, shadow_ray: &Ray, camera_ray: &Ray, &Intersection) -> Vec3;
}

pub struct SphereMaterial {
    color: Vec3,
    diffuse_coeff: f32,
    specular_coeff: f32,
    glossiness: f32,
}

impl SphereMaterial {
    pub fn new(color: Vec3, diffuse_coeff: f32, specular_coeff: f32, glossiness: f32) -> Self {
        SphereMaterial { color: color, diffuse_coeff: diffuse_coeff,
                         specular_coeff: specular_coeff, glossiness: glossiness }
    }
}

impl Material for SphereMaterial {
    fn raw_color(&self) -> Vec3 {
        self.color
    }

    fn color(&self, shadow_ray: &Ray, camera_ray: &Ray, hit: &Intersection) -> Vec3 {
        let f = f32::max(0., dot(&hit.normal, &shadow_ray.dir));
        let diffuse_color = self.color * f * self.diffuse_coeff;

        // Average the angles, flipping the camera ray because it's in the opposite direction
        let half_vec = ((shadow_ray.dir - camera_ray.dir) / 2.).normalize();
        let f = f32::max(0., dot(&half_vec, &hit.normal)).powf(self.glossiness);
        // TODO: Specular default color
        let specular_color = Vec3::new(255., 255., 255.) * f * self.specular_coeff;

        diffuse_color + specular_color
    }
}
