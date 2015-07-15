use std::f32;

use Vec3;
use material::Material;
use ray::{Intersection, Ray};

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
