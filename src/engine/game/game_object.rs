use cgmath::{Deg, Vector3, Matrix4, Zero, Angle, prelude::InnerSpace};
use winit::event::{KeyboardInput, VirtualKeyCode};

use crate::engine::resources::model::StarryModel;

static mut CURRENT_ID: u32 = 0;

#[derive(Debug, Clone, Copy)]
pub struct TransformComponent {
    pub translation: Vector3<f32>,
    pub scale: Vector3<f32>,
    pub rotation: Vector3<f32>,
}

impl TransformComponent {
    pub fn get_transform_matrix(&self) -> Matrix4<f32> {
        let translation_matrix = Matrix4::from_translation(self.translation);

        let x_rotation_matrix = Matrix4::from_angle_x(Deg(self.rotation.x));
        let y_rotation_matrix = Matrix4::from_angle_y(Deg(self.rotation.y));
        let z_rotation_matrix = Matrix4::from_angle_z(Deg(self.rotation.z));

        let scale_matrix = Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);
        let transform_matrix = translation_matrix * x_rotation_matrix * y_rotation_matrix * z_rotation_matrix * scale_matrix;
        transform_matrix
    }
}

pub struct StarryGameObject {
    id: u32,
    pub model: StarryModel,
    pub transform: TransformComponent,
}

impl StarryGameObject {
    fn new(id: u32, model: StarryModel) -> Self {
        Self {
            id,
            model,
            transform: TransformComponent {
                translation: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
                scale: Vector3 { x: 1.0, y: 1.0, z: 1.0 },
                rotation: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            },
        }
    }

    fn new_with_transform(id: u32, model: StarryModel, transform: TransformComponent) -> Self {
        Self {
            id,
            model,
            transform,
        }
    }

    pub fn create_new_game_object(model: StarryModel) -> Self {
        unsafe {
            CURRENT_ID += 1;
            StarryGameObject::new(CURRENT_ID, model)
        }
    }

    pub fn create_new_game_object_with_transform(
        model: StarryModel,
        transform: TransformComponent,
    ) -> Self {
        unsafe {
            CURRENT_ID += 1;
            StarryGameObject::new_with_transform(CURRENT_ID, model, transform)
        }
    }

    pub const fn get_id(&self) -> u32 {
        self.id
    }

    pub fn move_in_plane_xz(
        &mut self,
        dt: f32,
        key_pressed: KeyboardInput,
        look_speed: f32,
        move_speed: f32,
    ) {

        let mut rotate: Vector3<f32> = Vector3::zero();
        let mut move_dir: Vector3<f32> = Vector3::zero();

        let yaw = self.transform.rotation.y;
        let forward_dir = Vector3 {
            x: Deg(yaw).sin(),
            y: 0.0,
            z: Deg(yaw).cos()
        };

        let right_dir = Vector3 {
            x: forward_dir.z,
            y: 0.0,
            z: -forward_dir.x
        };

        let up_dir = Vector3 {
            x: 0.0,
            y: -1.0,
            z: 0.0
        };

        match key_pressed.virtual_keycode {
            Some(k) => {
                match k {
                    VirtualKeyCode::Left => {
                        rotate.y -= 1.0;
                    }

                    VirtualKeyCode::Right => {
                        rotate.y += 1.0;
                    }

                    VirtualKeyCode::Up => {
                        rotate.x += 1.0;
                    }

                    VirtualKeyCode::Down => {
                        rotate.x -= 1.0;
                    }

                    VirtualKeyCode::W => {
                        move_dir += forward_dir;
                    }

                    VirtualKeyCode::S => {
                        move_dir -= forward_dir;
                    }

                    VirtualKeyCode::D => {
                        move_dir += right_dir;
                    }

                    VirtualKeyCode::A => {
                        move_dir -= right_dir;
                    }

                    VirtualKeyCode::Q => {
                        move_dir += up_dir;
                    }

                    VirtualKeyCode::E => {
                        move_dir -= up_dir;
                    }

                    _ => ()
                }
            }

            _ => ()
        }

        if rotate.dot(rotate) > f32::EPSILON {
            self.transform.rotation = self.transform.rotation % 360.0 + (look_speed * dt * rotate.normalize());
        }
        if move_dir.dot(move_dir) > f32::EPSILON {
            self.transform.translation += move_speed * dt * move_dir.normalize(); 
        }
    }
}
