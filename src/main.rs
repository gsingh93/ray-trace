extern crate tracerlib;

extern crate image;
extern crate nalgebra;
extern crate toml;

use std::fs::File;
use std::io::Read;

use tracerlib::{ray_trace, Camera, Material, PointLight, Scene, Vec3};
use tracerlib::surface::{Plane, Sphere};
use tracerlib::texture::{CheckerboardTexture, Texture};

use image::FilterType;
use image::imageops::resize;

struct Config {
    width: u32,
    height: u32,
    out_file: String,
    samples: u32,
    reflection_depth: u16,
}

impl Config {
    fn new(filename: &str) -> Self {
        let mut toml_str = String::new();
        File::open(filename).unwrap().read_to_string(&mut toml_str).unwrap();

        let toml: toml::Value = toml_str.parse().unwrap();
        let width = toml.lookup("config.width").unwrap().as_integer().unwrap();
        let height = toml.lookup("config.height").unwrap().as_integer().unwrap();
        let out_file = toml.lookup("config.out_file").unwrap().as_str().unwrap();
        let samples = toml.lookup("config.samples").unwrap().as_integer().unwrap();
        let depth = toml.lookup("config.reflection_depth").unwrap().as_integer().unwrap();

        Config {
            width: width as u32,
            height: height as u32,
            out_file: out_file.to_owned(),
            samples: samples as u32,
            reflection_depth: depth as u16,
        }
    }
}

fn main() {
    let config = Config::new("config.toml");
    let scene = setup_scene();

    let im = ray_trace(&scene, config.samples * config.width, config.samples * config.height,
                       config.reflection_depth);

    let im = resize(&im, config.width, config.height, FilterType::Triangle);
    im.save(&config.out_file).unwrap();
}

fn setup_scene() -> Scene {
    let camera = {
        let pos = Vec3::new(0., 2., -5.);
        let lookat = Vec3::new(0., 1., 0.);
        let up = Vec3::new(0., 1., 0.);
        Camera::from_lookat(pos, lookat, up)
    };
    let checkerboard: Box<Texture> = Box::new(CheckerboardTexture::new(1.));
    let plane_material = Material::new(Vec3::new(100., 100., 100.), 0.7, 0., 0., 1.,
                                       Some(checkerboard));
    let plane = Plane::new(Vec3::new(1., 0., 1.), Vec3::new(0., 1., 0.), plane_material);

    let sphere_material = Material::new(Vec3::new(208.,127.,64.), 0.3, 0.2, 20., 0., None);
    let sphere = Sphere::new(Vec3::new(0., 1., 0.), 1., sphere_material);

    let light = PointLight::new(Vec3::new(3., 3., -4.), Vec3::new(0., 255., 0.), 2.);

    Scene::new(vec![Box::new(sphere), Box::new(plane)],
               vec![light],
               0.1, Vec3::new(255., 255., 255.), camera)
}
