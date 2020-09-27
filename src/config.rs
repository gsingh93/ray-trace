use std::fs::read_to_string;

use tracerlib::Vec3;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub scene_config_path: String,
    pub output_file: String,
    pub width: u32,
    pub height: u32,
    pub supersampling: u32,
    pub reflection_depth: u16,
}

impl Config {
    pub fn new(filename: &str) -> Result<Self> {
        let json_str = read_to_string(filename)
            .with_context(|| format!("Failed to open file {}", filename))?;

        Ok(serde_json::from_str(&json_str).context("Failed to parse JSON")?)
    }
}

impl SceneConfig {
    pub fn new(filename: &str) -> Result<Self> {
        let json_str = read_to_string(filename)
            .with_context(|| format!("Failed to open file {}", filename))?;

        Ok(serde_json::from_str(&json_str).context("Failed to parse JSON")?)
    }
}

#[derive(Deserialize, Debug)]
pub struct SceneConfig {
    pub ambient_const: f32,
    pub ambient_color: Vec3,
    pub camera: CameraConfig,
    pub textures: Vec<TextureConfig>,
    pub materials: Vec<MaterialConfig>,
    pub surfaces: Vec<SurfaceConfig>,
    pub lights: Vec<LightConfig>,
}

#[derive(Deserialize, Debug)]
pub struct CameraConfig {
    pub pos: Vec3,
    pub lookat: Vec3,
    pub up: Vec3,
}

#[derive(Deserialize, Debug)]
pub struct MaterialConfig {
    pub name: String,
    pub color: Vec3,
    pub diffuse: f32,
    pub specular: f32,
    pub glossiness: f32,
    pub reflectivity: f32,
    pub texture: Option<String>,
    pub normal_map: Option<(u32, usize, f32, f32, f32)>, // TODO
    pub displacement_map: Option<(u32, usize, f32, f32, f32)>, // TODO
}

#[derive(Deserialize, Debug)]
pub struct TextureConfig {
    pub name: String,
    pub options: TextureConfigOptions,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum TextureConfigOptions {
    Image { file_path: String },
    Checkerboard { dim: f32 },
}

#[derive(Deserialize, Debug)]
pub struct SurfaceConfig {
    pub material: String,
    pub pos: Vec3,
    pub options: SurfaceConfigOptions,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum SurfaceConfigOptions {
    Sphere { radius: f32 },
    Plane { normal: Vec3 },
}

#[derive(Deserialize, Debug)]
pub struct SphereSurfaceConfig {
    pub radius: f32,
}

#[derive(Deserialize, Debug)]
pub struct PlaneSurfaceConfig {
    pub normal: Vec3,
}

#[derive(Deserialize, Debug)]
pub struct LightConfig {
    pub pos: Vec3,
    pub color: Vec3,
    pub intensity: f32,
}
