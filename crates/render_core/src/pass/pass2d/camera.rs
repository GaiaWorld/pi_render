use crate::Matrix4f;
use nalgebra::Orthographic3;

// 2D相机，正交投影
pub struct Camera2D {
    pub view: Matrix4f,
    pub project: Matrix4f,
}

impl Default for Camera2D {
    fn default() -> Self {
        Self {
            view: Matrix4f::identity(),
            project: Matrix4f::identity(),
        }
    }
}

impl Camera2D {
    /// 相机 的 ViewMatrix 是 相机自己的 WorldMatrix的 逆矩阵
    pub fn set_view_matrix(&mut self, world_matrix: &Matrix4f) {
        self.view = world_matrix.try_inverse().unwrap();
    }

    /// 取 视图矩阵，返回 连续的 16个 f32 切片
    pub fn get_view(&self) -> &[f32] {
        let r = self.view.as_slice();
        r
    }

    /// 设置 正交投影
    pub fn set_projection(
        &mut self,
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) {
        let o = Orthographic3::new(left, right, bottom, top, near, far);
        self.project = o.to_homogeneous();
    }

    /// 取 [正交]投影矩阵，返回 连续的 16个 f32 切片
    pub fn get_projection(&self) -> &[f32] {
        let r = self.project.as_slice();
        r
    }
}
