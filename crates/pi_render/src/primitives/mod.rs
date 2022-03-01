//! Aabb    包围盒
//! Shpere  球
//! Plane   平面
//! Frustum 视锥体
//! CubemapFrusta

use crate::{Vec3, Vec4, Mat4};

/// An Axis-Aligned Bounding Box
#[derive(Clone, Debug, Default)]
pub struct Aabb {
    pub center: Vec3,
    pub half_extents: Vec3,
}

impl Aabb {
    pub fn from_min_max(minimum: Vec3, maximum: Vec3) -> Self {
        let center = 0.5 * (maximum + minimum);
        let half_extents = 0.5 * (maximum - minimum);
        Self {
            center,
            half_extents,
        }
    }

    /// Calculate the relative radius of the AABB with respect to a plane
    pub fn relative_radius(&self, p_normal: &Vec3, axes: &[Vec3]) -> f32 {
        // NOTE: dot products on Vec3 use SIMD and even with the overhead of conversion are net faster than Vec3
        let half_extents = Vec3::from(self.half_extents);
        Vec3::new(
            p_normal.dot(&axes[0]),
            p_normal.dot(&axes[1]),
            p_normal.dot(&axes[2]),
        )
        .abs()
        .dot(&half_extents)
    }

    pub fn min(&self) -> Vec3 {
        self.center - self.half_extents
    }

    pub fn max(&self) -> Vec3 {
        self.center + self.half_extents
    }
}


/// A plane defined by a normal and distance value along the normal
/// Any point p is in the plane if n.p = d
/// For planes defining half-spaces such as for frusta, if n.p > d then p is on the positive side of the plane.
#[derive(Clone, Copy, Debug, Default)]
pub struct Plane {
    pub normal_d: Vec4,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Frustum {
    pub planes: [Plane; 6],
}


#[derive(Debug, Default)]
pub struct CubemapFrusta {
    pub frusta: [Frustum; 6],
}

impl CubemapFrusta {
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Frustum> {
        self.frusta.iter()
    }
    pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut Frustum> {
        self.frusta.iter_mut()
    }
}
