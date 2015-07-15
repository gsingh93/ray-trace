use Vec3;

pub struct PointLight {
    pos: Vec3,
    color: Vec3,
    intensity: f32,
}

impl PointLight {
    pub fn new(pos: Vec3, color: Vec3, intensity: f32) -> Self {
        PointLight { pos: pos, color: color, intensity: intensity }
    }
}
