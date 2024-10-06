use std::collections::HashMap;
use nalgebra::{Matrix2, Matrix3, Matrix4, Vector2, Vector3, Vector4};
use uuid::Uuid;
use eldra_macro::Reflection;
use crate::entity::Component;

pub mod shader_graph;

pub fn register_shader_graph_components(reg: &mut HashMap<Uuid, fn()->Box<dyn Component>>) {

}

#[derive(Default)]
enum ShaderVar {
    #[default]
    UNKNOWN,
    INT(i32),
    FLOAT(f32),
    RGB(Vector3<f32>),
    RGBA(Vector3<f32>),
    VEC2(Vector2<f32>),
    VEC3(Vector3<f32>),
    VEC4(Vector4<f32>),
    MAT2(Matrix2<f32>),
    MAT3(Matrix3<f32>),
    MAT4(Matrix4<f32>),
    TEXTURE(String),
}

#[derive(Default,Reflection)]
enum ShaderDefaultValue {
    #[default]
    NONE = 0,

    // engine
    CAMERA_DIR = 1,
    TIME_DELTA = 2,

    // shader stage
    VERTEX_UV0 = 100,
    VERTEX_UV1 = 101,
    VERTEX_COLOR0 = 102,
    VERTEX_COLOR1 = 103,

    // pipeline texture
    PIPELINE_SCENE_COLOR = 300,
    PIPELINE_DEPTH = 301,
    PIPELINE_SCREEN_UV = 302,

}