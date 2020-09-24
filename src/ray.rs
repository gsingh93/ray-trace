use crate::Vec3;

#[derive(Debug)]
pub struct Ray {
    pub origin: Vec3,
    pub dir: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, dir: Vec3) -> Self {
        Ray {
            origin,
            dir: dir.normalize(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Intersection {
    pub pos: Vec3,
    pub normal: Vec3,
    pub dist: f32,
    pub u: f32,
    pub v: f32,
}

impl Intersection {
    pub fn new(pos: Vec3, normal: Vec3, dist: f32, u: f32, v: f32) -> Self {
        Intersection {
            pos,
            normal,
            dist,
            u,
            v,
        }
    }
}
