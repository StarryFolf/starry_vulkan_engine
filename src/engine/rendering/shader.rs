use std::sync::Arc;

use cgmath::Matrix4;
use vulkano::{device::Device, shader::ShaderModule};

pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/assets/shaders/triangle.vert"
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/assets/shaders/triangle.frag"
    }
}

pub struct StarryShader;

impl StarryShader {
    pub fn load_vertex_shader(device: Arc<Device>) -> Arc<ShaderModule> {
        vs::load(device).unwrap()
    }

    pub fn load_fragment_shader(device: Arc<Device>) -> Arc<ShaderModule> {
        fs::load(device).unwrap()
    }

    pub fn create_push_constant_data_struct(transform: Matrix4<f32>) -> vs::PushConstantData {
        vs::PushConstantData {
            transform: transform.into(),
        }
    }
}
