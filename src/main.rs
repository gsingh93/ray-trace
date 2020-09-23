use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;

use tracerlib::light::PointLight;
use tracerlib::material::{DisplacementMap, Material, NormalMap};
use tracerlib::surface::{Plane, Sphere, Surface};
use tracerlib::texture::{CheckerboardTexture, ImageTexture, Texture};
use tracerlib::{ray_trace, Camera, Scene, Vec3};

use image::imageops::{resize, FilterType};

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
        File::open(filename)
            .unwrap()
            .read_to_string(&mut toml_str)
            .unwrap();

        let toml: toml::Value = toml_str.parse().unwrap();
        let config = toml["config"].as_table().unwrap();
        let width = config["width"].as_integer().unwrap();
        let height = config["height"].as_integer().unwrap();
        let out_file = decode_string(&config["out_file"]);
        let samples = config["samples"].as_integer().unwrap();
        let depth = config["reflection_depth"].as_integer().unwrap();
        let scene_name = decode_string(&config["scene"]);

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

    let im = ray_trace(
        &scene,
        config.samples * config.width,
        config.samples * config.height,
        config.reflection_depth,
    );

    let im = resize(&im, config.width, config.height, FilterType::Triangle);
    im.save(&config.out_file).unwrap();
}

fn setup_scene(scene: &str) -> Scene {
    let mut path = String::new();
    path.push_str("scenes/");
    path.push_str(&scene);

    let mut toml_str = String::new();
    File::open(path)
        .unwrap()
        .read_to_string(&mut toml_str)
        .unwrap();

    let toml: toml::Value = toml_str.parse().unwrap();

    let materials = decode_materials(&toml["material"]);
    decode_scene(&toml["scene"], materials)
}

fn decode_materials(materials: &toml::Value) -> BTreeMap<String, Material> {
    let mut map = BTreeMap::new();
    for material in materials.as_array().unwrap() {
        let (name, m) = decode_material(material);
        map.insert(name, m);
    }
    map
}

fn decode_material(material: &toml::Value) -> (String, Material) {
    let name = decode_string(&material["name"]);
    let color = decode_vec3(&material["color"]);
    let diffuse = material["diffuse"].as_float().unwrap() as f32;
    let specular = material["specular"].as_float().unwrap() as f32;
    let glossiness = material["glossiness"].as_float().unwrap() as f32;
    let reflectivity = material["reflectivity"].as_float().unwrap() as f32;
    let texture = if let Some(checkerboard) = material.get("checkerboard") {
        Some(Box::new(CheckerboardTexture::new(
            checkerboard.as_float().unwrap() as f32
        )) as Box<dyn Texture>)
    } else {
        if let Some(texture) = material.get("texture") {
            Some(Box::new(ImageTexture::new(texture.as_str().unwrap())) as Box<dyn Texture>)
        } else {
            None
        }
    };

    let normal_map = if let Some(map) = material.get("normal_map") {
        let v = map.as_array().unwrap();
        let seed = v[0].as_float().unwrap() as u32;
        let octaves = v[1].as_float().unwrap() as usize;
        let wavelength = v[2].as_float().unwrap() as f32;
        let persistence = v[3].as_float().unwrap() as f32;
        let lacunarity = v[4].as_float().unwrap() as f32;
        Some(NormalMap::new(
            seed,
            octaves,
            wavelength,
            persistence,
            lacunarity,
        ))
    } else {
        None
    };

    let displacement_map = if let Some(map) = material.get("displacement_map") {
        let v = map.as_array().unwrap();
        let seed = v[0].as_float().unwrap() as u32;
        let octaves = v[1].as_float().unwrap() as usize;
        let wavelength = v[2].as_float().unwrap() as f32;
        let persistence = v[3].as_float().unwrap() as f32;
        let lacunarity = v[4].as_float().unwrap() as f32;
        Some(DisplacementMap::new(
            seed,
            octaves,
            wavelength,
            persistence,
            lacunarity,
        ))
    } else {
        None
    };
    let m = Material::new(
        color,
        diffuse,
        specular,
        glossiness,
        reflectivity,
        texture,
        normal_map,
        displacement_map,
    );
    (name, m)
}

fn decode_scene(scene: &toml::Value, materials: BTreeMap<String, Material>) -> Scene {
    let camera = decode_camera(&scene["camera"]);
    let surfaces = decode_surfaces(&scene["surface"], materials);
    let lights = decode_lights(&scene["light"]);
    let ambient_const = scene["ambient_const"].as_float().unwrap() as f32;
    let ambient_color = decode_vec3(&scene["ambient_color"]);

    Scene::new(surfaces, lights, ambient_const, ambient_color, camera)
}

fn decode_camera(camera: &toml::Value) -> Camera {
    let pos = decode_vec3(&camera["pos"]);
    let lookat = decode_vec3(&camera["lookat"]);
    let up = decode_vec3(&camera["up"]);
    Camera::from_lookat(pos, lookat, up)
}

fn decode_surfaces(
    surfaces: &toml::Value,
    materials: BTreeMap<String, Material>,
) -> Vec<Box<dyn Surface>> {
    let mut v = Vec::new();
    for surface in surfaces.as_array().unwrap() {
        v.push(decode_surface(surface, &materials))
    }
    v
}

fn decode_surface(
    surface: &toml::Value,
    materials: &BTreeMap<String, Material>,
) -> Box<dyn Surface> {
    let material_name = surface["material"].as_str().unwrap();
    let material = materials.get(material_name).unwrap().clone();

    let type_ = surface["type"].as_str().unwrap();
    match type_ {
        "plane" => Box::new(decode_plane(surface, material)),
        "sphere" => Box::new(decode_sphere(surface, material)),
        _ => panic!("Unsupported object type: {}", type_),
    }
}

fn decode_sphere(sphere: &toml::Value, material: Material) -> Sphere {
    let pos = decode_vec3(&sphere["pos"]);
    let radius = sphere["radius"].as_float().unwrap() as f32;

    Sphere::new(pos, radius, material)
}

fn decode_plane(plane: &toml::Value, material: Material) -> Plane {
    let pos = decode_vec3(&plane["pos"]);
    let normal = decode_vec3(&plane["normal"]);

    Plane::new(pos, normal, material)
}

fn decode_lights(lights: &toml::Value) -> Vec<PointLight> {
    let mut v = Vec::new();
    for light in lights.as_array().unwrap() {
        v.push(decode_light(light))
    }
    v
}

fn decode_light(light: &toml::Value) -> PointLight {
    let pos = decode_vec3(&light["pos"]);
    let color = decode_vec3(&light["color"]);
    let intensity = light["intensity"].as_float().unwrap() as f32;

    PointLight::new(pos, color, intensity)
}

fn decode_string(s: &toml::Value) -> String {
    s.as_str().unwrap().to_owned()
}

fn decode_vec3(vec: &toml::Value) -> Vec3 {
    let v = vec.as_array().unwrap();
    if v[0].as_float().is_none() {
        Vec3::new(
            v[0].as_integer().unwrap() as f32,
            v[1].as_integer().unwrap() as f32,
            v[2].as_integer().unwrap() as f32,
        )
    } else {
        Vec3::new(
            v[0].as_float().unwrap() as f32,
            v[1].as_float().unwrap() as f32,
            v[2].as_float().unwrap() as f32,
        )
    }
}
