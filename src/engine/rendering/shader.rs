#[macro_export]
macro_rules! create_shader {
    ($shader_type:expr, $path:expr, $shader_name:ident) => {
        pub mod $shader_name {
            vulkano_shaders::shader! {
                ty: $shader_type,
                path: $path
            }
        }
    };
}
