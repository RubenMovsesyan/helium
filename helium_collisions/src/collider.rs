use cgmath::{InnerSpace, Quaternion, Rotation, Vector3, Zero};
use std::{any::Any, ops::Range};

const PLANE_LOCAL_NORMAL: Vector3<f32> = Vector3 {
    x: 0.0,
    y: 1.0,
    z: 0.0,
};

pub trait Collider {
    fn is_colliding(&self, other: &dyn Collider) -> bool;

    fn is_colliding_x(&self, other: &dyn Collider) -> bool;
    fn is_colliding_y(&self, other: &dyn Collider) -> bool;
    fn is_colliding_z(&self, other: &dyn Collider) -> bool;

    fn contains_x(&self, range: &Range<f32>) -> bool;
    fn contains_y(&self, range: &Range<f32>) -> bool;
    fn contains_z(&self, range: &Range<f32>) -> bool;

    fn snap(&mut self, other: &dyn Collider);
    fn snap_x(&mut self, other: &dyn Collider);
    fn snap_y(&mut self, other: &dyn Collider);
    fn snap_z(&mut self, other: &dyn Collider);

    fn origin(&self) -> &Vector3<f32>;
    fn width(&self) -> f32;
    fn height(&self) -> f32;
    fn length(&self) -> f32;

    fn set_origin(&mut self, new_origin: &Vector3<f32>);

    fn as_any(&self) -> &dyn Any;
}

#[derive(PartialEq, Debug)]
pub struct RectangleCollider {
    // x
    width: f32,
    // y
    height: f32,
    // z
    length: f32,
    // origin
    origin: Vector3<f32>,

    vertices: [Vector3<f32>; 8],
}

impl RectangleCollider {
    fn compute_vertices(
        width: f32,
        height: f32,
        length: f32,
        origin: &Vector3<f32>,
    ) -> [Vector3<f32>; 8] {
        let (width_2, height_2, length_2) = (width / 2.0, height / 2.0, length / 2.0);

        let vertices = [
            Vector3 {
                x: origin.x + width_2,
                y: origin.y + height_2,
                z: origin.z + length_2,
            },
            Vector3 {
                x: origin.x + width_2,
                y: origin.y + height_2,
                z: origin.z - length_2,
            },
            Vector3 {
                x: origin.x - width_2,
                y: origin.y + height_2,
                z: origin.z + length_2,
            },
            Vector3 {
                x: origin.x - width_2,
                y: origin.y + height_2,
                z: origin.z - length_2,
            },
            Vector3 {
                x: origin.x + width_2,
                y: origin.y - height_2,
                z: origin.z + length_2,
            },
            Vector3 {
                x: origin.x + width_2,
                y: origin.y - height_2,
                z: origin.z - length_2,
            },
            Vector3 {
                x: origin.x - width_2,
                y: origin.y - height_2,
                z: origin.z + length_2,
            },
            Vector3 {
                x: origin.x - width_2,
                y: origin.y - height_2,
                z: origin.z - length_2,
            },
        ];

        vertices
    }

    pub fn new(width: f32, height: f32, length: f32, origin: Vector3<f32>) -> Self {
        let vertices = Self::compute_vertices(width, height, length, &origin);
        Self {
            width,
            height,
            length,
            origin,
            vertices,
        }
    }
}

impl Collider for RectangleCollider {
    fn is_colliding(&self, other: &dyn Collider) -> bool {
        // let (width_2, height_2, length_2) =
        //     (self.width / 2.0, self.height / 2.0, self.length / 2.0);

        // HACK: this is bad and needs to be fixed
        if let Some(plane) = other.as_any().downcast_ref::<StationaryPlaneCollider>() {
            let mut distances = Vec::new();

            for verticie in self.vertices {
                distances.push(plane.local_normal.dot(verticie - plane.origin));
            }

            // This calculates if the rectangular collider is intersecting the plane
            let mut sum: f32 = 0.0;
            let mut abs_sum: f32 = 0.0;
            for distance in distances {
                sum += distance;
                abs_sum += f32::abs(distance);
            }
            sum = f32::abs(sum);

            if abs_sum != sum {
                // Now we need to calculate if the point is in the range of the plane

                // Project all the points onto the plane
                let mut projected_points: [Vector3<f32>; 8] = [Vector3::zero(); 8];

                for (index, verticie) in self.vertices.iter().enumerate() {
                    let v = verticie - plane.origin();
                    let dist = v.dot(plane.local_normal.normalize());
                    let projected_point = verticie - dist * plane.local_normal.normalize();
                    projected_points[index] = projected_point;
                }

                // Draw a line from each projected point to the corners of the plane
                // Measure the angles between each sequential point and if they add up to 360 then it is projected on the plane
                // If the angles add up to 0, then it is not projected on the plane
                // NOTE: This approach only works for a plane like this because it is a perfect shape
                let num_plane_points = plane.plane_points.len();
                for projected_point in projected_points {
                    // For every point in the plane
                    let mut signs: [f32; 4] = [0.0; 4];
                    for plane_point_index in 0..num_plane_points {
                        // Find the vector from the current point to one of the points on the plane
                        // and also the vector to the next one
                        let vec_a = plane.plane_points[plane_point_index] - projected_point;
                        let vec_b = plane.plane_points[(plane_point_index + 1) % num_plane_points]
                            - projected_point;

                        // This is the area of the triangle created by the vector a and b
                        let cross = vec_a.cross(vec_b);
                        // Find the direction of each vector relative to the normal of the plane
                        signs[plane_point_index] = if plane.local_normal.dot(cross) < 0.0 {
                            -1.0
                        } else {
                            1.0
                        };
                    }

                    if signs == [1.0; 4] || signs == [-1.0; 4] {
                        return true;
                    }
                }
            }
        }

        // Make sure the range of our rectangular collider is contained within the other collider
        // other.contains_x(&((self.origin.x - width_2)..(self.origin.x + width_2)))
        //     && other.contains_y(&((self.origin.y - height_2)..(self.origin.y + height_2)))
        //     && other.contains_z(&((self.origin.z - length_2)..(self.origin.z + length_2)))
        false
    }

    fn is_colliding_x(&self, other: &dyn Collider) -> bool {
        let width_2 = self.width / 2.0;

        other.contains_x(&((self.origin.x - width_2)..(self.origin.x + width_2)))
    }

    fn is_colliding_y(&self, other: &dyn Collider) -> bool {
        let height_2 = self.height / 2.0;

        other.contains_y(&((self.origin.y - height_2)..(self.origin.y + height_2)))
    }

    fn is_colliding_z(&self, other: &dyn Collider) -> bool {
        let length_2 = self.length / 2.0;

        other.contains_z(&((self.origin.z - length_2)..(self.origin.z + length_2)))
    }

    fn contains_x(&self, range: &Range<f32>) -> bool {
        let width_2 = self.width / 2.0;
        let x_range = (self.origin.x - width_2)..(self.origin.x + width_2);

        x_range.contains(&range.start) || x_range.contains(&range.end)
    }

    fn contains_y(&self, range: &Range<f32>) -> bool {
        let height_2 = self.height / 2.0;
        let y_range = (self.origin.y - height_2)..(self.origin.y + height_2);

        y_range.contains(&range.start) || y_range.contains(&range.end)
    }

    fn contains_z(&self, range: &Range<f32>) -> bool {
        let length_2 = self.length / 2.0;
        let z_range = (self.origin.z - length_2)..(self.origin.z + length_2);

        z_range.contains(&range.start) || z_range.contains(&range.end)
    }

    fn snap(&mut self, other: &dyn Collider) {
        let (self_width_2, self_height_2, self_length_2) =
            (self.width / 2.0, self.height / 2.0, self.length / 2.0);

        let (other_width_2, other_height_2, other_length_2) = (
            other.width() / 2.0,
            other.height() / 2.0,
            other.length() / 2.0,
        );

        // Snap the x position
        if self.origin.x < other.origin().x {
            self.origin.x = other.origin().x - other_width_2 - self_width_2;
        } else if self.origin.x > other.origin().x {
            self.origin.x = other.origin().x + other_width_2 + self_width_2;
        }

        // Snap the y position
        if self.origin.y < other.origin().y {
            self.origin.y = other.origin().y - other_height_2 - self_height_2;
        } else if self.origin.y > other.origin().y {
            self.origin.y = other.origin().y + other_height_2 + self_height_2;
        }

        // Snap the z position
        if self.origin.z < other.origin().z {
            self.origin.z = other.origin().z - other_length_2 - self_length_2;
        } else if self.origin.y > other.origin().y {
            self.origin.z = other.origin().z + other_length_2 + self_length_2;
        }

        self.vertices = Self::compute_vertices(self.width, self.height, self.length, &self.origin);
    }

    fn snap_x(&mut self, other: &dyn Collider) {
        let self_width_2 = self.width / 2.0;

        let other_width_2 = other.width() / 2.0;

        // Snap the x position
        if self.origin.x < other.origin().x {
            self.origin.x = other.origin().x - other_width_2 - self_width_2;
        } else if self.origin.x > other.origin().x {
            self.origin.x = other.origin().x + other_width_2 + self_width_2;
        }

        self.vertices = Self::compute_vertices(self.width, self.height, self.length, &self.origin);
    }

    fn snap_y(&mut self, other: &dyn Collider) {
        let self_height_2 = self.height / 2.0;

        let other_height_2 = other.height() / 2.0;

        // Snap the y position
        if self.origin.y < other.origin().y {
            self.origin.y = other.origin().y - other_height_2 - self_height_2;
        } else if self.origin.y > other.origin().y {
            self.origin.y = other.origin().y + other_height_2 + self_height_2;
        }

        self.vertices = Self::compute_vertices(self.width, self.height, self.length, &self.origin);
    }

    fn snap_z(&mut self, other: &dyn Collider) {
        let self_length_2 = self.length / 2.0;

        let other_length_2 = other.length() / 2.0;

        // Snap the z position
        if self.origin.z < other.origin().z {
            self.origin.z = other.origin().z - other_length_2 - self_length_2;
        } else if self.origin.z > other.origin().z {
            self.origin.z = other.origin().z + other_length_2 + self_length_2;
        }

        self.vertices = Self::compute_vertices(self.width, self.height, self.length, &self.origin);
    }

    fn set_origin(&mut self, new_origin: &Vector3<f32>) {
        self.origin = new_origin.clone();
        self.vertices = Self::compute_vertices(self.width, self.height, self.length, &self.origin);
    }

    fn origin(&self) -> &Vector3<f32> {
        &self.origin
    }

    fn width(&self) -> f32 {
        self.width
    }

    fn height(&self) -> f32 {
        self.height
    }

    fn length(&self) -> f32 {
        self.length
    }

    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }
}

#[derive(Debug)]
pub struct StationaryPlaneCollider {
    pub width: f32,
    pub length: f32,
    pub origin: Vector3<f32>,

    plane_points: [Vector3<f32>; 4],
    local_normal: Vector3<f32>,
}

impl StationaryPlaneCollider {
    /// Creates a new Stationary plane collider a normalized normal
    pub fn new(
        width: f32,
        length: f32,
        origin: Vector3<f32>,
        mut orientation: Quaternion<f32>,
    ) -> Self {
        let width_2 = width / 2.0;
        let length_2 = length / 2.0;
        orientation = orientation.normalize();
        let mut plane_points = [
            Vector3 {
                x: -width_2,
                y: 0.0,
                z: length_2,
            },
            Vector3 {
                x: width_2,
                y: 0.0,
                z: length_2,
            },
            Vector3 {
                x: width_2,
                y: 0.0,
                z: -length_2,
            },
            Vector3 {
                x: -width_2,
                y: 0.0,
                z: -length_2,
            },
        ];

        for point in plane_points.iter_mut() {
            *point = orientation.rotate_vector(*point);
            *point += origin;
        }

        Self {
            width,
            length,
            origin,
            plane_points,
            local_normal: orientation.rotate_vector(PLANE_LOCAL_NORMAL).normalize(),
        }
    }
}

impl Collider for StationaryPlaneCollider {
    // Alwasy false because it is stationary
    fn is_colliding(&self, _other: &dyn Collider) -> bool {
        false
    }

    fn is_colliding_x(&self, _other: &dyn Collider) -> bool {
        false
    }

    fn is_colliding_y(&self, _other: &dyn Collider) -> bool {
        false
    }

    fn is_colliding_z(&self, _other: &dyn Collider) -> bool {
        false
    }

    fn contains_x(&self, range: &Range<f32>) -> bool {
        let x_range = {
            let mut min_x: Option<f32> = None;
            let mut max_x: Option<f32> = None;
            for point in self.plane_points {
                if let Some(min_x) = min_x.as_mut() {
                    *min_x = min_x.min(point.x);
                } else {
                    min_x = Some(point.x);
                }

                if let Some(max_x) = max_x.as_mut() {
                    *max_x = max_x.max(point.x);
                } else {
                    max_x = Some(point.x);
                }
            }

            min_x.unwrap()..max_x.unwrap()
        };

        if x_range.is_empty() {
            return range.contains(&x_range.start);
        }

        x_range.contains(&range.start) || x_range.contains(&range.end)
    }

    fn contains_y(&self, range: &Range<f32>) -> bool {
        let y_range = {
            let mut min_y: Option<f32> = None;
            let mut max_y: Option<f32> = None;
            for point in self.plane_points {
                if let Some(min_y) = min_y.as_mut() {
                    *min_y = min_y.min(point.y);
                } else {
                    min_y = Some(point.y);
                }

                if let Some(max_y) = max_y.as_mut() {
                    *max_y = max_y.max(point.y);
                } else {
                    max_y = Some(point.y);
                }
            }

            min_y.unwrap()..max_y.unwrap()
        };

        if y_range.is_empty() {
            return range.contains(&y_range.start);
        }

        y_range.contains(&range.start) || y_range.contains(&range.end)
    }

    fn contains_z(&self, range: &Range<f32>) -> bool {
        let z_range = {
            let mut min_z: Option<f32> = None;
            let mut max_z: Option<f32> = None;
            for point in self.plane_points {
                if let Some(min_z) = min_z.as_mut() {
                    *min_z = min_z.min(point.z);
                } else {
                    min_z = Some(point.z);
                }

                if let Some(max_z) = max_z.as_mut() {
                    *max_z = max_z.max(point.z);
                } else {
                    max_z = Some(point.z);
                }
            }

            min_z.unwrap()..max_z.unwrap()
        };

        if z_range.is_empty() {
            return range.contains(&z_range.start);
        }

        z_range.contains(&range.start) || z_range.contains(&range.end)
    }

    // No snapping on stationalry colliders
    fn snap(&mut self, _other: &dyn Collider) {}
    fn snap_x(&mut self, _other: &dyn Collider) {}
    fn snap_y(&mut self, _other: &dyn Collider) {}
    fn snap_z(&mut self, _other: &dyn Collider) {}

    /// No setting on stationary colliders
    fn set_origin(&mut self, _new_origin: &Vector3<f32>) {}

    fn origin(&self) -> &Vector3<f32> {
        &self.origin
    }

    fn width(&self) -> f32 {
        self.width
    }

    fn height(&self) -> f32 {
        0.0
    }

    fn length(&self) -> f32 {
        self.length
    }

    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cgmath::{One, Zero};

    #[test]
    fn test_rectangle_colliders_x() {
        let collider_1 = RectangleCollider::new(5.0, 5.0, 5.0, Vector3::zero());

        let collider_2 = RectangleCollider::new(
            5.0,
            5.0,
            5.0,
            Vector3 {
                x: 4.0,
                y: 0.0,
                z: 0.0,
            },
        );

        assert!(collider_1.is_colliding(&collider_2));

        let collider_2 = RectangleCollider::new(
            5.0,
            5.0,
            5.0,
            Vector3 {
                x: 6.0,
                y: 0.0,
                z: 0.0,
            },
        );

        assert!(!collider_1.is_colliding(&collider_2));

        let collider_2 = RectangleCollider::new(
            5.0,
            5.0,
            5.0,
            Vector3 {
                x: 5.0,
                y: 0.0,
                z: 0.0,
            },
        );

        assert!(collider_1.is_colliding(&collider_2));
    }

    #[test]
    fn test_rectangle_colliders_y() {
        let collider_1 = RectangleCollider::new(5.0, 5.0, 5.0, Vector3::zero());

        let collider_2 = RectangleCollider::new(
            5.0,
            5.0,
            5.0,
            Vector3 {
                x: 0.0,
                y: 4.0,
                z: 0.0,
            },
        );

        assert!(collider_1.is_colliding(&collider_2));

        let collider_2 = RectangleCollider::new(
            5.0,
            5.0,
            5.0,
            Vector3 {
                x: 0.0,
                y: 6.0,
                z: 0.0,
            },
        );

        assert!(!collider_1.is_colliding(&collider_2));

        let collider_2 = RectangleCollider::new(
            5.0,
            5.0,
            5.0,
            Vector3 {
                x: 0.0,
                y: 5.0,
                z: 0.0,
            },
        );

        assert!(collider_1.is_colliding(&collider_2));
    }

    #[test]
    fn test_rectangle_colliders_z() {
        let collider_1 = RectangleCollider::new(5.0, 5.0, 5.0, Vector3::zero());

        let collider_2 = RectangleCollider::new(
            5.0,
            5.0,
            5.0,
            Vector3 {
                x: 0.0,
                y: 0.0,
                z: 4.0,
            },
        );

        assert!(collider_1.is_colliding(&collider_2));

        let collider_2 = RectangleCollider::new(
            5.0,
            5.0,
            5.0,
            Vector3 {
                x: 0.0,
                y: 0.0,
                z: 6.0,
            },
        );

        assert!(!collider_1.is_colliding(&collider_2));

        let collider_2 = RectangleCollider::new(
            5.0,
            5.0,
            5.0,
            Vector3 {
                x: 0.0,
                y: 0.0,
                z: 5.0,
            },
        );

        assert!(collider_1.is_colliding(&collider_2));
    }

    #[test]
    fn test_rectangle_rectangle_snap() {
        let mut collider_1 = RectangleCollider::new(
            3.0,
            6.0,
            1.0,
            Vector3 {
                x: 3.5,
                y: 4.0,
                z: 0.0,
            },
        );

        let collider_2 = RectangleCollider::new(
            4.0,
            4.0,
            1.0,
            Vector3 {
                x: 6.0,
                y: 7.0,
                z: 0.0,
            },
        );

        assert!(collider_1.is_colliding(&collider_2));

        collider_1.snap(&collider_2);

        assert_eq!(
            collider_1,
            RectangleCollider::new(
                3.0,
                6.0,
                1.0,
                Vector3 {
                    x: 2.5,
                    y: 2.0,
                    z: 0.0,
                }
            )
        );
    }

    #[test]
    fn test_rectangle_rectangle_snap_x() {
        let mut collider_1 = RectangleCollider::new(
            3.0,
            6.0,
            1.0,
            Vector3 {
                x: 3.5,
                y: 4.0,
                z: 0.0,
            },
        );

        let collider_2 = RectangleCollider::new(
            4.0,
            4.0,
            1.0,
            Vector3 {
                x: 6.0,
                y: 7.0,
                z: 0.0,
            },
        );

        assert!(collider_1.is_colliding(&collider_2));

        collider_1.snap_x(&collider_2);

        assert_eq!(
            collider_1,
            RectangleCollider::new(
                3.0,
                6.0,
                1.0,
                Vector3 {
                    x: 2.5,
                    y: 4.0,
                    z: 0.0,
                }
            )
        );
    }

    #[test]
    fn test_rectangle_rectangle_snap_y() {
        let mut collider_1 = RectangleCollider::new(
            3.0,
            6.0,
            1.0,
            Vector3 {
                x: 3.5,
                y: 4.0,
                z: 0.0,
            },
        );

        let collider_2 = RectangleCollider::new(
            4.0,
            4.0,
            1.0,
            Vector3 {
                x: 6.0,
                y: 7.0,
                z: 0.0,
            },
        );

        assert!(collider_1.is_colliding(&collider_2));

        collider_1.snap_y(&collider_2);

        assert_eq!(
            collider_1,
            RectangleCollider::new(
                3.0,
                6.0,
                1.0,
                Vector3 {
                    x: 3.5,
                    y: 2.0,
                    z: 0.0,
                }
            )
        );
    }

    #[test]
    fn test_rectangle_rectangle_snap_z() {
        let mut collider_1 = RectangleCollider::new(
            3.0,
            6.0,
            1.0,
            Vector3 {
                x: 3.5,
                y: 4.0,
                z: 0.0,
            },
        );

        let collider_2 = RectangleCollider::new(
            4.0,
            4.0,
            1.0,
            Vector3 {
                x: 6.0,
                y: 7.0,
                z: 0.0,
            },
        );

        assert!(collider_1.is_colliding(&collider_2));

        collider_1.snap_z(&collider_2);

        assert_eq!(
            collider_1,
            RectangleCollider::new(
                3.0,
                6.0,
                1.0,
                Vector3 {
                    x: 3.5,
                    y: 4.0,
                    z: 0.0,
                }
            )
        );
    }

    #[test]
    fn test_rectangle_plane_collision() {
        let rectangle_collider = RectangleCollider::new(
            5.0,
            5.0,
            5.0,
            Vector3 {
                x: 0.0,
                y: 2.5,
                z: 0.0,
            },
        );

        let plane_collider = StationaryPlaneCollider::new(
            10.0,
            10.0,
            Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Quaternion::one(),
        );

        assert!(rectangle_collider.is_colliding(&plane_collider));

        let rectangle_collider = RectangleCollider::new(
            5.0,
            5.0,
            5.0,
            Vector3 {
                x: 0.0,
                y: 6.0,
                z: 0.0,
            },
        );

        assert!(!rectangle_collider.is_colliding(&plane_collider));
    }
}
