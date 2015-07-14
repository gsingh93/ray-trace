use Vec3;

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
