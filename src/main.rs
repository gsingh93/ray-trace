mod config;

use std::collections::BTreeMap;

use tracerlib::light::PointLight;
use tracerlib::material::{DisplacementMap, Material, NormalMap};
use tracerlib::surface::{Plane, Sphere, Surface};
use tracerlib::texture::{CheckerboardTexture, ImageTexture, Texture};
use tracerlib::{ray_trace, Camera, Scene};

use image::imageops::{resize, FilterType};

use anyhow::{Context, Result};

use config::{
    CameraConfig, Config, LightConfig, MaterialConfig, SceneConfig, SurfaceConfig,
    SurfaceConfigOptions, TextureConfig, TextureConfigOptions,
};

fn main() -> Result<()> {
    let config = Config::new("config.json").context("Failed to load config")?;
    let scene_config =
        SceneConfig::new(&config.scene_config_path).context("Failed to load scene config")?;
    let scene = create_scene(scene_config).context("Failed to create scene")?;

    let im = ray_trace(
        &scene,
        config.supersampling * config.width,
        config.supersampling * config.height,
        config.reflection_depth,
    );

    let im = resize(&im, config.width, config.height, FilterType::Triangle);
    im.save(&config.output_file).context("Failed to save file")
}

fn create_scene(scene: SceneConfig) -> Result<Scene> {
    let textures = load_textures(scene.textures).context("Failed to load textures")?;
    let materials =
        load_materials(scene.materials, &textures).context("Failed to load materials")?;
    let surfaces = load_surfaces(scene.surfaces, &materials).context("Failed to load surfaces")?;
    let lights = load_lights(scene.lights);
    let camera = load_camera(scene.camera);

    Ok(Scene::new(
        surfaces,
        lights,
        scene.ambient_const,
        scene.ambient_color,
        camera,
    ))
}

fn load_textures(textures: Vec<TextureConfig>) -> Result<BTreeMap<String, Box<dyn Texture>>> {
    let mut map = BTreeMap::new();
    for texture in textures {
        let t = load_texture(&texture).context("Failed to load texture")?;
        // TODO
        map.insert(texture.name.clone(), t);
    }
    Ok(map)
}

fn load_texture(texture: &TextureConfig) -> Result<Box<dyn Texture>> {
    match &texture.options {
        TextureConfigOptions::Image { file_path } => {
            ImageTexture::new(file_path).map(|x| Box::new(x) as Box<dyn Texture>)
        }
        TextureConfigOptions::Checkerboard { dim } => Ok(Box::new(CheckerboardTexture::new(*dim))),
    }
}

fn load_materials(
    materials: Vec<MaterialConfig>,
    textures: &BTreeMap<String, Box<dyn Texture>>,
) -> Result<BTreeMap<String, Material>> {
    let mut map = BTreeMap::new();
    for material in materials {
        let m = load_material(&material, &textures)?;
        // TODO
        map.insert(material.name.clone(), m);
    }
    Ok(map)
}

fn load_material(
    material: &MaterialConfig,
    textures: &BTreeMap<String, Box<dyn Texture>>,
) -> Result<Material> {
    let normal_map = material
        .normal_map
        .map(|nm| NormalMap::new(nm.0, nm.1, nm.2, nm.3, nm.4));
    let displacement_map = material
        .displacement_map
        .map(|dm| DisplacementMap::new(dm.0, dm.1, dm.2, dm.3, dm.4));

    let texture = if let Some(ref texture_name) = material.texture {
        Some(
            textures
                .get(texture_name)
                .cloned()
                .with_context(|| format!("Texture {} not found", texture_name))?,
        )
    } else {
        None
    };

    Ok(Material::new(
        material.color,
        material.diffuse,
        material.specular,
        material.glossiness,
        material.reflectivity,
        texture,
        normal_map,
        displacement_map,
    ))
}

fn load_surfaces(
    surfaces: Vec<SurfaceConfig>,
    materials: &BTreeMap<String, Material>,
) -> Result<Vec<Box<dyn Surface>>> {
    let mut v = Vec::new();
    for surface in surfaces {
        v.push(load_surface(surface, materials)?);
    }
    Ok(v)
}

fn load_surface(
    surface: SurfaceConfig,
    materials: &BTreeMap<String, Material>,
) -> Result<Box<dyn Surface>> {
    let material = materials
        .get(&surface.material)
        .with_context(|| format!("Material {} not found", surface.material))?
        .clone();

    match surface.options {
        SurfaceConfigOptions::Sphere { radius } => {
            Ok(Box::new(Sphere::new(surface.pos, radius, material)))
        }
        SurfaceConfigOptions::Plane { normal } => {
            Ok(Box::new(Plane::new(surface.pos, normal, material)))
        }
    }
}

fn load_lights(lights: Vec<LightConfig>) -> Vec<PointLight> {
    let mut v = Vec::new();
    for light in lights {
        v.push(PointLight::new(light.pos, light.color, light.intensity))
    }
    v
}

fn load_camera(camera: CameraConfig) -> Camera {
    Camera::from_lookat(camera.pos, camera.lookat, camera.up)
}
