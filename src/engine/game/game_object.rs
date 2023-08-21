use cgmath::{Vector2, Matrix2, Angle, Deg};

use crate::engine::resources::model::StarryModel;

static mut CURRENT_ID: u32 = 0;

#[derive(Debug, Clone, Copy)]
pub struct Transform2DComponent {
    pub translation: Vector2<f32>,
    pub scale: Vector2<f32>,
    pub rotation: f32,
}

impl Transform2DComponent {
    pub fn get_2d_transform_matrix(self) -> Matrix2<f32> {
        let rotation_rad = Deg(self.rotation);

        let s = rotation_rad.sin();
        let c = rotation_rad.cos();

        let rotation_matrix = Matrix2 {
            x: Vector2 { x: c, y: s },
            y: Vector2 { x: -s, y: c }
        };

        let scale_matrix = Matrix2 { 
            x: Vector2 { x: self.scale.x, y: 0.0 }, 
            y: Vector2 { x: 0.0, y: self.scale.y } 
        };

        rotation_matrix * scale_matrix
    }
}

pub struct StarryGameObject {
    id: u32,
    pub model: StarryModel,
    pub transform: Transform2DComponent
}

impl StarryGameObject {
    fn new(id: u32, model: StarryModel) -> Self {
        Self {
            id,
            model,
            transform: Transform2DComponent { 
                translation: Vector2 { x: 0.0, y: 0.0 }, 
                scale: Vector2 { x: 1.0, y: 1.0 },
                rotation: 0.0,
            }
        }
    }

    fn new_with_transform(id: u32, model: StarryModel, transform: Transform2DComponent) -> Self {
        Self {
            id,
            model,
            transform
        }
    }

    pub fn create_new_game_object(model: StarryModel) -> Self{
        unsafe {
            CURRENT_ID += 1;
            StarryGameObject::new(CURRENT_ID, model)
        }
    }

    pub fn create_new_game_object_with_transform(model: StarryModel, transform: Transform2DComponent) -> Self{
        unsafe {
            CURRENT_ID += 1;
            StarryGameObject::new_with_transform(CURRENT_ID, model, transform)
        }
    }

    pub const fn get_id(self: &Self) -> u32 {
        self.id
    }
}