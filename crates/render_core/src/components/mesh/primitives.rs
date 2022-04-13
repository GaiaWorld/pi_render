use nalgebra::SimdValue;

use crate::{Mat4, Vec3, Vec4};

/// An Axis-Aligned Bounding Box
#[derive(Clone, Debug, Default)]
pub struct Aabb {
    pub center: Vec3,
    pub half_extents: Vec3,
}

impl Aabb {
    #[inline]
    pub fn from_min_max(minimum: Vec3, maximum: Vec3) -> Self {
        let minimum = Vec3::from(minimum);
        let maximum = Vec3::from(maximum);
        let center = 0.5 * (maximum + minimum);
        let half_extents = 0.5 * (maximum - minimum);
        Self {
            center,
            half_extents,
        }
    }

    /// Calculate the relative radius of the AABB with respect to a plane
    #[inline]
    pub fn relative_radius(&self, p_normal: &Vec3, axes: &[Vec3]) -> f32 {
        // NOTE: dot products on Vec3 use SIMD and even with the overhead of conversion are net faster than Vec3
        let half_extents = self.half_extents;
        Vec3::new(
            p_normal.dot(&axes[0]),
            p_normal.dot(&axes[1]),
            p_normal.dot(&axes[2]),
        )
        .abs()
        .dot(&half_extents)
    }

    #[inline]
    pub fn min(&self) -> Vec3 {
        self.center - self.half_extents
    }

    #[inline]
    pub fn max(&self) -> Vec3 {
        self.center + self.half_extents
    }
}

/// A plane defined by a unit normal and distance from the origin along the normal
/// Any point p is in the plane if n.p + d = 0
/// For planes defining half-spaces such as for frusta, if n.p + d > 0 then p is on
/// the positive side (inside) of the plane.
#[derive(Clone, Copy, Debug, Default)]
pub struct Plane {
    normal_d: Vec4,
}

// impl Plane {
//     /// Constructs a `Plane` from a 4D vector whose first 3 components
//     /// are the normal and whose last component is the distance along the normal
//     /// from the origin.
//     /// This constructor ensures that the normal is normalized and the distance is
//     /// scaled accordingly so it represents the signed distance from the origin.
//     #[inline]
//     pub fn new(normal_d: Vec4) -> Self {
//         Self {
//             normal_d: normal_d * normal_d.xyz().length_recip(),
//         }
//     }

//     /// `Plane` unit normal
//     #[inline]
//     pub fn normal(&self) -> Vec3 {
//         Vec3::from(self.normal_d)
//     }

//     /// Signed distance from the origin along the unit normal such that n.p + d = 0 for point p in
//     /// the `Plane`
//     #[inline]
//     pub fn d(&self) -> f32 {
//         self.normal_d.w
//     }

//     /// `Plane` unit normal and signed distance from the origin such that n.p + d = 0 for point p
//     /// in the `Plane`
//     #[inline]
//     pub fn normal_d(&self) -> Vec4 {
//         self.normal_d
//     }
// }

#[derive(Clone, Copy, Debug, Default)]
pub struct Frustum {
    pub planes: [Plane; 6],
}

// impl Frustum {
//     // NOTE: This approach of extracting the frustum planes from the view
//     // projection matrix is from Foundations of Game Engine Development 2
//     // Rendering by Lengyel. Slight modification has been made for when
//     // the far plane is infinite but we still want to cull to a far plane.
//     #[inline]
//     pub fn from_view_projection(
//         view_projection: &Mat4,
//         view_translation: &Vec3,
//         view_backward: &Vec3,
//         far: f32,
//     ) -> Self {
//         let row3 = view_projection.row(3);
//         let mut planes = [Plane::default(); 6];
//         for (i, plane) in planes.iter_mut().enumerate().take(5) {
//             let row = view_projection.row(i / 2);
//             *plane = Plane::new(if (i & 1) == 0 && i != 4 {
//                 row3 + row
//             } else {
//                 row3 - row
//             });
//         }
//         let far_center = *view_translation - far * *view_backward;
//         planes[5] = Plane::new(view_backward.extend(-view_backward.dot(&far_center)));
//         Self { planes }
//     }

//     #[inline]
//     pub fn intersects_obb(&self, aabb: &Aabb, model_to_world: &Mat4, intersect_far: bool) -> bool {
//         let aabb_center_world = model_to_world.transform_point3a(aabb.center).extend(1.0);
//         let axes = [
//             Vec3::from(model_to_world.x_axis),
//             Vec3::from(model_to_world.y_axis),
//             Vec3::from(model_to_world.z_axis),
//         ];

//         let max = if intersect_far { 6 } else { 5 };
//         for plane in &self.planes[..max] {
//             let p_normal = Vec3::from(plane.normal_d());
//             let relative_radius = aabb.relative_radius(&p_normal, &axes);
//             if plane.normal_d().dot(aabb_center_world) + relative_radius <= 0.0 {
//                 return false;
//             }
//         }
//         true
//     }
// }

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