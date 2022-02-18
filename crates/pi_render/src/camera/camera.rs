use crate::{view::window::{RenderWindows, WindowId}, Vec2, Vec3, Mat4};

#[derive(Default, Debug)]
pub struct Camera {
    pub projection_matrix: Mat4<f32>,
    pub name: Option<String>,
    pub window: WindowId,
    pub depth_calculation: DepthCalculation,
    pub near: f32,
    pub far: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum DepthCalculation {
    /// Pythagorean distance; works everywhere, more expensive to compute.
    Distance,
    /// Optimization for 2D; assuming the camera points towards -Z.
    ZDifference,
}

impl Default for DepthCalculation {
    fn default() -> Self {
        DepthCalculation::Distance
    }
}

impl Camera {
    /// Given a position in world space, use the camera to compute the screen space coordinates.
    pub fn world_to_screen(
        &self,
        windows: &RenderWindows,
        camera_transform: &Mat4,
        world_position: Vec3,
    ) -> Option<Vec2> {
        let window = windows.get(&self.window)?;
        let window_size = Vec2::new(window.width(), window.height());
        // Build a transform to convert from world to NDC using camera data
        let world_to_ndc: Mat4 =
            self.projection_matrix * camera_transform.compute_matrix().inverse();
        let ndc_space_coords: Vec3 = world_to_ndc.project_point3(world_position);
        // NDC z-values outside of 0 < z < 1 are outside the camera frustum and are thus not in screen space
        if ndc_space_coords.z < 0.0 || ndc_space_coords.z > 1.0 {
            return None;
        }
        // Once in NDC space, we can discard the z element and rescale x/y to fit the screen
        let screen_space_coords =
            (ndc_space_coords.truncate() + Vec2::ONE) / 2.0 * window_size;
        if !screen_space_coords.is_nan() {
            Some(screen_space_coords)
        } else {
            None
        }
    }
}
