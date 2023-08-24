use cgmath::{Matrix4, SquareMatrix, Deg, Angle, Zero, Vector3, prelude::InnerSpace};

pub struct StarryCamera {
    projection_matrix: Matrix4<f32>,
    view_matrix: Matrix4<f32>
}

impl StarryCamera {
    pub fn new() -> Self {
        Self { 
            projection_matrix: Matrix4::zero(),
            view_matrix: Matrix4::identity(), 
        }
    }

    pub fn set_orthographic_projection(
        &mut self,
        left: f32,
        right: f32,
        top: f32,
        bottom: f32,
        near: f32, 
        far: f32
    ) {
        self.projection_matrix = Matrix4::identity();
        self.projection_matrix[0][0] = 2.0 / (right - left);
        self.projection_matrix[1][1] = 2.0 / (bottom - top);
        self.projection_matrix[2][2] = 1.0 / (far - near);
        self.projection_matrix[3][0] = -(right + left) / (right - left);
        self.projection_matrix[3][1] = -(bottom + top) / (bottom - top);
        self.projection_matrix[3][2] = -near / (far - near);
    }

    pub fn set_perspective_projection(
        &mut self,
        fov_y: f32,
        aspect: f32,
        near: f32,
        far: f32
    ) {
        if fov_y >= 180.0 {
            panic!("FOV too large!");
        }

        let tan_half_fov_y = Deg(fov_y / 2.0).tan();
        self.projection_matrix[0][0] = 1.0 / (aspect * tan_half_fov_y);
        self.projection_matrix[1][1] = 1.0 / (tan_half_fov_y);
        self.projection_matrix[2][2] = far / (far - near);
        self.projection_matrix[2][3] = 1.0;
        self.projection_matrix[3][2] = -(far * near) / (far - near);
    }

    pub fn set_view_direction(
        &mut self,
        position: Vector3<f32>,
        direction: Vector3<f32>,
        up: Vector3<f32>
    ) {
        let w = direction.normalize();
        let u = w.cross(up).normalize();
        let v = w.cross(u);

        self.view_matrix = Matrix4::identity();
        self.view_matrix[0][0] = u.x;
        self.view_matrix[1][0] = u.y;
        self.view_matrix[2][0] = u.z;
        self.view_matrix[0][1] = v.x;
        self.view_matrix[1][1] = v.y;
        self.view_matrix[2][1] = v.z;
        self.view_matrix[0][2] = w.x;
        self.view_matrix[1][2] = w.y;
        self.view_matrix[2][2] = w.z;
        self.view_matrix[3][0] = -u.dot(position);
        self.view_matrix[3][1] = -v.dot(position);
        self.view_matrix[3][2] = -w.dot(position);
    }

    pub fn set_view_target(
        &mut self,
        position: Vector3<f32>,
        target: Vector3<f32>,
        up: Vector3<f32>
    ) {
        self.set_view_direction(position, target - position, up)
    }

    pub fn set_view_xyz(
        &mut self,
        position: Vector3<f32>,
        rotation: Vector3<f32>
    ) {
        let c3 = Deg(rotation.z).cos();
        let s3 = Deg(rotation.z).sin();
        let c2 = Deg(rotation.x).cos();
        let s2 = Deg(rotation.x).sin();
        let c1 = Deg(rotation.y).cos();
        let s1 = Deg(rotation.y).sin();
        let u = Vector3 {
            x: (c1 * c3 + s1 * s2 * s3), 
            y: (c2 * s3), 
            z: (c1 * s2 * s3 - c3 * s1)
        };
        let v = Vector3 {
            x: (c3 * s1 * s2 - c1 * s3), 
            y: (c2 * c3), 
            z: (c1 * c3 * s2 + s1 * s3)
        };
        let w = Vector3 {
            x: (c2 * s1), 
            y: (-s2), 
            z: (c1 * c2)
        };
        self.view_matrix = Matrix4::identity();
        self.view_matrix[0][0] = u.x;
        self.view_matrix[1][0] = u.y;
        self.view_matrix[2][0] = u.z;
        self.view_matrix[0][1] = v.x;
        self.view_matrix[1][1] = v.y;
        self.view_matrix[2][1] = v.z;
        self.view_matrix[0][2] = w.x;
        self.view_matrix[1][2] = w.y;
        self.view_matrix[2][2] = w.z;
        self.view_matrix[3][0] = -u.dot(position);
        self.view_matrix[3][1] = -v.dot(position);
        self.view_matrix[3][2] = -w.dot(position);
    }

    pub fn get_projection_matrix(&self) -> Matrix4<f32> {
        self.projection_matrix
    }

    pub fn get_view_matrix(&self) -> Matrix4<f32> {
        self.view_matrix
    }
}