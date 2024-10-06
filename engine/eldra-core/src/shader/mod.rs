use std::collections::HashMap;
use uuid::Uuid;
use eldra_macro::Reflection;
use crate::entity::Component;
use crate::reflection::{*};

pub mod shader_graph;

pub fn register_shader_graph_components(reg: &mut HashMap<Uuid, fn()->Box<dyn Component>>) {

}

#[derive(Default,Reflection)]
enum ShaderVar {
    #[default]
    UNKNOWN,
    INT(i32),
    FLOAT(f32),
    RGB(Vec3f),
    RGBA(Vec4f),
    VEC2(Vec2f),
    VEC3(Vec3f),
    VEC4(Vec4f),
    MAT2(Mat2f),
    MAT3(Mat3f),
    MAT4(Mat4f),
    TEXTURE(String),
    COLORSPACE(ColorSpace),
    SAMPLEMODE(SampleMode),
}

#[derive(Default,Reflection)]
enum ColorSpace {
    #[default]
    RGB = 0,
    SRGB = 1,
    TANGENT = 2,
}

#[derive(Default,Reflection)]
enum SampleMode {
    #[default]
    CLAMP = 0,
    WRAP = 1,
    MIRROR = 2,
}
