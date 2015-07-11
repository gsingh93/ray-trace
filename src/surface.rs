use {Intersection, Ray, Vec3};

use nalgebra::{dot, Norm};

pub trait Surface {
    fn intersect(&self, &Ray) -> Option<Intersection>;
    // For debugging
    fn name(&self) -> &'static str;
}

#[derive(Clone, Debug)]
pub struct Sphere {
    pos: Vec3,
    radius: f32,
}

impl Sphere {
    pub fn new(pos: Vec3, radius: f32) -> Self {
        Sphere { pos: pos, radius: radius }
    }
}


impl Surface for Sphere {
    fn name(&self) -> &'static str {
        "Sphere"
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

pub struct Plane {
    point: Vec3,
    normal: Vec3,
}

impl Plane {
    pub fn new(point: Vec3, normal: Vec3) -> Self {
        Plane { point: point, normal: normal }
    }
}

impl Surface for Plane {
    fn name(&self) -> &'static str {
        "Plane"
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

#[test]
fn test() {
    let plane = Plane::new(Vec3::new(0., 0., 0.), Vec3::new(0., 1., 0.));
    let sphere = Sphere::new(Vec3::new(0., 0., 0.), 1.);
    let ray = Ray::new(Vec3::new(4., 4., 0.), Vec3::new(-4., -4., 0.));
    println!("plane: {:?}", plane.intersect(&ray));
    println!("sphere: {:?}", sphere.intersect(&ray));
}
