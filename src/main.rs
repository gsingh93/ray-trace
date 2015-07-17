extern crate tracerlib;

extern crate image;
extern crate nalgebra;
extern crate toml;

use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;

use tracerlib::{ray_trace, Camera, Scene, Vec3};
use tracerlib::light::PointLight;
use tracerlib::material::{DisplacementMap, Material, NormalMap};
use tracerlib::surface::{Plane, Sphere, Surface};
use tracerlib::texture::{CheckerboardTexture, ImageTexture, Texture};

use image::FilterType;
use image::imageops::resize;

struct Config {
    width: u32,
    height: u32,
    out_file: String,
    samples: u32,
    reflection_depth: u16,
    scene: String,
}

impl Config {
    fn new(filename: &str) -> Self {
        let mut toml_str = String::new();
        File::open(filename).unwrap().read_to_string(&mut toml_str).unwrap();

        let toml: toml::Value = toml_str.parse().unwrap();
        let width = toml.lookup("config.width").unwrap().as_integer().unwrap();
        let height = toml.lookup("config.height").unwrap().as_integer().unwrap();
        let out_file = decode_string(toml.lookup("config.out_file").unwrap());
        let samples = toml.lookup("config.samples").unwrap().as_integer().unwrap();
        let depth = toml.lookup("config.reflection_depth").unwrap().as_integer().unwrap();
        let scene_name = decode_string(toml.lookup("config.scene").unwrap());

        Config {
            width: width as u32,
            height: height as u32,
            out_file: out_file,
            samples: samples as u32,
            reflection_depth: depth as u16,
            scene: scene_name,
        }
    }
}

fn main() {
    let config = Config::new("config.toml");
    let scene = setup_scene(&config.scene);

    let im = ray_trace(&scene, config.samples * config.width, config.samples * config.height,
                       config.reflection_depth);

    let im = resize(&im, config.width, config.height, FilterType::Triangle);
    im.save(&config.out_file).unwrap();
}

fn setup_scene(scene: &str) -> Scene {
    let mut path = String::new();
    path.push_str("scenes/");
    path.push_str(&scene);

    let mut toml_str = String::new();
    File::open(path).unwrap().read_to_string(&mut toml_str).unwrap();

    let toml: toml::Value = toml_str.parse().unwrap();

    let materials = decode_materials(toml.lookup("material").unwrap());
    decode_scene(toml.lookup("scene").unwrap(), materials)
}

fn decode_materials(materials: &toml::Value) -> BTreeMap<String, Material> {
    let mut map = BTreeMap::new();
    for material in materials.as_slice().unwrap() {
        let (name, m) = decode_material(material);
        map.insert(name, m);
    }
    map
}

fn decode_material(material: &toml::Value) -> (String, Material) {
    let name = decode_string(material.lookup("name").unwrap());
    let color = decode_vec3(material.lookup("color").unwrap());
    let diffuse = material.lookup("diffuse").unwrap().as_float().unwrap() as f32;
    let specular = material.lookup("specular").unwrap().as_float().unwrap() as f32;
    let glossiness = material.lookup("glossiness").unwrap().as_float().unwrap() as f32;
    let reflectivity = material.lookup("reflectivity").unwrap().as_float().unwrap() as f32;
    let texture = if let Some(checkerboard) = material.lookup("checkerboard") {
        Some(Box::new(CheckerboardTexture::new(checkerboard.as_float().unwrap() as f32))
             as Box<Texture>)
    } else {
        if let Some(texture) = material.lookup("texture") {
            Some(Box::new(ImageTexture::new(texture.as_str().unwrap())) as Box<Texture>)
        } else {
            None
        }
    };

    let normal_map = if let Some(map) = material.lookup("normal_map") {
        let v = map.as_slice().unwrap();
        let seed = v[0].as_float().unwrap() as u32;
        let octaves = v[1].as_float().unwrap() as usize;
        let wavelength = v[2].as_float().unwrap() as f32;
        let persistence = v[3].as_float().unwrap() as f32;
        let lacunarity = v[4].as_float().unwrap() as f32;
        Some(NormalMap::new(seed, octaves, wavelength, persistence, lacunarity))
    } else {
        None
    };

    let displacement_map = if let Some(map) = material.lookup("displacement_map") {
        let v = map.as_slice().unwrap();
        let seed = v[0].as_float().unwrap() as u32;
        let octaves = v[1].as_float().unwrap() as usize;
        let wavelength = v[2].as_float().unwrap() as f32;
        let persistence = v[3].as_float().unwrap() as f32;
        let lacunarity = v[4].as_float().unwrap() as f32;
        Some(DisplacementMap::new(seed, octaves, wavelength, persistence, lacunarity))
    } else {
        None
    };
    let m = Material::new(color, diffuse, specular, glossiness, reflectivity, texture, normal_map,
                          displacement_map);
    (name, m)
}

fn decode_scene(scene: &toml::Value, materials: BTreeMap<String, Material>) -> Scene {
    let camera = decode_camera(scene.lookup("camera").unwrap());
    let surfaces = decode_surfaces(scene.lookup("surface").unwrap(), materials);
    let lights = decode_lights(scene.lookup("light").unwrap());
    let ambient_const = scene.lookup("ambient_const").unwrap().as_float().unwrap() as f32;
    let ambient_color = decode_vec3(scene.lookup("ambient_color").unwrap());

    Scene::new(surfaces, lights, ambient_const, ambient_color, camera)
}

fn decode_camera(camera: &toml::Value) -> Camera {
    let pos = decode_vec3(camera.lookup("pos").unwrap());
    let lookat = decode_vec3(camera.lookup("lookat").unwrap());
    let up = decode_vec3(camera.lookup("up").unwrap());
    Camera::from_lookat(pos, lookat, up)
}

fn decode_surfaces(surfaces: &toml::Value, materials: BTreeMap<String, Material>)
                   -> Vec<Box<Surface>> {
    let mut v = Vec::new();
    for surface in surfaces.as_slice().unwrap() {
        v.push(decode_surface(surface, &materials))
    }
    v
}

fn decode_surface(surface: &toml::Value, materials: &BTreeMap<String, Material>) -> Box<Surface> {
    let material_name = surface.lookup("material").unwrap().as_str().unwrap();
    let material = materials.get(material_name).unwrap().clone();

    let type_ = surface.lookup("type").unwrap().as_str().unwrap();
    match type_ {
        "plane" => Box::new(decode_plane(surface, material)),
        "sphere" => Box::new(decode_sphere(surface, material)),
        _ => panic!("Unsupported object type: {}", type_)
    }
}

fn decode_sphere(sphere: &toml::Value, material: Material) -> Sphere {
    let pos = decode_vec3(sphere.lookup("pos").unwrap());
    let radius = sphere.lookup("radius").unwrap().as_float().unwrap() as f32;

    Sphere::new(pos, radius, material)
}

fn decode_plane(plane: &toml::Value, material: Material) -> Plane {
    let pos = decode_vec3(plane.lookup("pos").unwrap());
    let normal = decode_vec3(plane.lookup("normal").unwrap());

    Plane::new(pos, normal, material)
}

fn decode_lights(lights: &toml::Value) -> Vec<PointLight> {
    let mut v = Vec::new();
    for light in lights.as_slice().unwrap() {
        v.push(decode_light(light))
    }
    v
}

fn decode_light(light: &toml::Value) -> PointLight {
    let pos = decode_vec3(light.lookup("pos").unwrap());
    let color = decode_vec3(light.lookup("color").unwrap());
    let intensity = light.lookup("intensity").unwrap().as_float().unwrap() as f32;

    PointLight::new(pos, color, intensity)
}

fn decode_string(s: &toml::Value) -> String {
    s.as_str().unwrap().to_owned()
}

fn decode_vec3(vec: &toml::Value) -> Vec3 {
    let v = vec.as_slice().unwrap();
    if v[0].as_float().is_none() {
        Vec3::new(v[0].as_integer().unwrap() as f32,
                  v[1].as_integer().unwrap() as f32,
                  v[2].as_integer().unwrap() as f32)
    } else {
        Vec3::new(v[0].as_float().unwrap() as f32,
                  v[1].as_float().unwrap() as f32,
                  v[2].as_float().unwrap() as f32)
    }
}
