use std::f32;

use crate::material::Material;
use crate::ray::{Intersection, Ray};
use crate::Vec3;

pub trait Surface {
    fn intersect(&self, ray: &Ray) -> Option<Intersection>;
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
        Sphere {
            pos: pos,
            radius: radius,
            material: material,
        }
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
        let b = 2. * ray.dir.dot(&center_offset);
        let c = center_offset.norm_squared() - self.radius * self.radius;

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
                return None;
            };

            let pos = ray.origin + ray.dir * d;
            let normal = (pos - self.pos).normalize();

            let normal = if self.material.has_normal_map() {
                self.material.apply_normal_map(&normal, &pos)
            } else {
                normal
            };

            let pos = if self.material.has_displacement_map() {
                self.material.apply_displacement_map(&pos)
            } else {
                pos
            };

            let center_vec = (self.pos - pos).normalize();
            let u = 0.5 + center_vec.z.atan2(center_vec.x) / (2. * f32::consts::PI);
            let v = 0.5 - center_vec.y.atan() / f32::consts::PI;

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
        Plane {
            point: point,
            normal: normal,
            material: material,
        }
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
        let denom = ray.dir.dot(&self.normal);
        if denom == 0. {
            return None;
        }
        let d = self.normal.dot(&(self.point - ray.origin)) / denom;
        if d > 0. {
            let pos = ray.origin + ray.dir * d;

            let n = &self.normal;
            let u_axis = Vec3::new(n.y, n.z, -n.x);
            let v_axis = u_axis.cross(n);
            let u = pos.dot(&u_axis);
            let v = pos.dot(&v_axis);

            let normal = if self.material.has_normal_map() {
                self.material.apply_normal_map(&self.normal, &pos)
            } else {
                self.normal
            };

            let pos = if self.material.has_displacement_map() {
                self.material.apply_displacement_map(&pos)
            } else {
                pos
            };

            Some(Intersection::new(pos, normal, d, u, v))
        } else {
            None
        }
    }
}
