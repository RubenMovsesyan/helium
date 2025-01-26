use cgmath::{InnerSpace, Point3, Quaternion, Rotation, Vector3};
use std::ops::Range;

pub trait Collider {
    fn is_colliding(&self, other: &dyn Collider) -> bool;

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
}

#[derive(PartialEq, Debug)]
pub struct RectangleCollider {
    // x
    pub width: f32,
    // y
    pub height: f32,
    // z
    pub length: f32,
    // origin
    pub origin: Vector3<f32>,
}

impl Collider for RectangleCollider {
    fn is_colliding(&self, other: &dyn Collider) -> bool {
        let (width_2, height_2, length_2) =
            (self.width / 2.0, self.height / 2.0, self.length / 2.0);

        // Make sure the range of our rectangular collider is contained within the other collider
        other.contains_x(&((self.origin.x - width_2)..(self.origin.x + width_2)))
            && other.contains_y(&((self.origin.y - height_2)..(self.origin.y + height_2)))
            && other.contains_z(&((self.origin.z - length_2)..(self.origin.z + length_2)))
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
}

pub struct StationaryPlaneCollider {
    pub width: f32,
    pub length: f32,
    pub origin: Vector3<f32>,

    plane_points: [Point3<f32>; 4],
}

impl StationaryPlaneCollider {
    /// Creates a new Stationary plane collider a normalized normal
    pub fn new(
        width: f32,
        length: f32,
        origin: Vector3<f32>,
        orientation: Quaternion<f32>,
    ) -> Self {
        let width_2 = width / 2.0;
        let length_2 = length / 2.0;
        let mut plane_points = [
            Point3 {
                x: -width_2,
                y: 0.0,
                z: -length_2,
            },
            Point3 {
                x: width_2,
                y: 0.0,
                z: -length_2,
            },
            Point3 {
                x: -width_2,
                y: 0.0,
                z: length_2,
            },
            Point3 {
                x: width_2,
                y: 0.0,
                z: length_2,
            },
        ];

        for point in plane_points.iter_mut() {
            *point = orientation.normalize().rotate_point(*point);
        }

        Self {
            width,
            length,
            origin,
            plane_points,
        }
    }
}

impl Collider for StationaryPlaneCollider {
    // Alwasy false because it is stationary
    fn is_colliding(&self, _other: &dyn Collider) -> bool {
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

        x_range.contains(&range.start)
            || x_range.contains(&range.end)
            || range.contains(&x_range.start)
            || range.contains(&x_range.end)
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

        y_range.contains(&range.start)
            || y_range.contains(&range.end)
            || range.contains(&y_range.start)
            || range.contains(&y_range.end)
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

        z_range.contains(&range.start)
            || z_range.contains(&range.end)
            || range.contains(&z_range.start)
            || range.contains(&z_range.end)
    }

    // No snapping on stationalry colliders
    fn snap(&mut self, _other: &dyn Collider) {}
    fn snap_x(&mut self, _other: &dyn Collider) {}
    fn snap_y(&mut self, _other: &dyn Collider) {}
    fn snap_z(&mut self, _other: &dyn Collider) {}

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use cgmath::{One, Zero};

    #[test]
    fn test_rectangle_colliders_x() {
        let collider_1 = RectangleCollider {
            width: 5.0,
            height: 5.0,
            length: 5.0,
            origin: Vector3::zero(),
        };

        let collider_2 = RectangleCollider {
            width: 5.0,
            height: 5.0,
            length: 5.0,
            origin: Vector3 {
                x: 4.0,
                y: 0.0,
                z: 0.0,
            },
        };

        assert!(collider_1.is_colliding(&collider_2));

        let collider_2 = RectangleCollider {
            width: 5.0,
            height: 5.0,
            length: 5.0,
            origin: Vector3 {
                x: 6.0,
                y: 0.0,
                z: 0.0,
            },
        };

        assert!(!collider_1.is_colliding(&collider_2));

        let collider_2 = RectangleCollider {
            width: 5.0,
            height: 5.0,
            length: 5.0,
            origin: Vector3 {
                x: 5.0,
                y: 0.0,
                z: 0.0,
            },
        };

        assert!(collider_1.is_colliding(&collider_2));
    }

    #[test]
    fn test_rectangle_colliders_y() {
        let collider_1 = RectangleCollider {
            width: 5.0,
            height: 5.0,
            length: 5.0,
            origin: Vector3::zero(),
        };

        let collider_2 = RectangleCollider {
            width: 5.0,
            height: 5.0,
            length: 5.0,
            origin: Vector3 {
                x: 0.0,
                y: 4.0,
                z: 0.0,
            },
        };

        assert!(collider_1.is_colliding(&collider_2));

        let collider_2 = RectangleCollider {
            width: 5.0,
            height: 5.0,
            length: 5.0,
            origin: Vector3 {
                x: 0.0,
                y: 6.0,
                z: 0.0,
            },
        };

        assert!(!collider_1.is_colliding(&collider_2));

        let collider_2 = RectangleCollider {
            width: 5.0,
            height: 5.0,
            length: 5.0,
            origin: Vector3 {
                x: 0.0,
                y: 5.0,
                z: 0.0,
            },
        };

        assert!(collider_1.is_colliding(&collider_2));
    }

    #[test]
    fn test_rectangle_colliders_z() {
        let collider_1 = RectangleCollider {
            width: 5.0,
            height: 5.0,
            length: 5.0,
            origin: Vector3::zero(),
        };

        let collider_2 = RectangleCollider {
            width: 5.0,
            height: 5.0,
            length: 5.0,
            origin: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 4.0,
            },
        };

        assert!(collider_1.is_colliding(&collider_2));

        let collider_2 = RectangleCollider {
            width: 5.0,
            height: 5.0,
            length: 5.0,
            origin: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 6.0,
            },
        };

        assert!(!collider_1.is_colliding(&collider_2));

        let collider_2 = RectangleCollider {
            width: 5.0,
            height: 5.0,
            length: 5.0,
            origin: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 5.0,
            },
        };

        assert!(collider_1.is_colliding(&collider_2));
    }

    #[test]
    fn test_rectangle_rectangle_snap() {
        let mut collider_1 = RectangleCollider {
            width: 3.0,
            height: 6.0,
            length: 1.0,
            origin: Vector3 {
                x: 3.5,
                y: 4.0,
                z: 0.0,
            },
        };

        let collider_2 = RectangleCollider {
            width: 4.0,
            height: 4.0,
            length: 1.0,
            origin: Vector3 {
                x: 6.0,
                y: 7.0,
                z: 0.0,
            },
        };

        assert!(collider_1.is_colliding(&collider_2));

        collider_1.snap(&collider_2);

        assert_eq!(
            collider_1,
            RectangleCollider {
                width: 3.0,
                height: 6.0,
                length: 1.0,
                origin: Vector3 {
                    x: 2.5,
                    y: 2.0,
                    z: 0.0,
                }
            }
        );
    }

    #[test]
    fn test_rectangle_rectangle_snap_x() {
        let mut collider_1 = RectangleCollider {
            width: 3.0,
            height: 6.0,
            length: 1.0,
            origin: Vector3 {
                x: 3.5,
                y: 4.0,
                z: 0.0,
            },
        };

        let collider_2 = RectangleCollider {
            width: 4.0,
            height: 4.0,
            length: 1.0,
            origin: Vector3 {
                x: 6.0,
                y: 7.0,
                z: 0.0,
            },
        };

        assert!(collider_1.is_colliding(&collider_2));

        collider_1.snap_x(&collider_2);

        assert_eq!(
            collider_1,
            RectangleCollider {
                width: 3.0,
                height: 6.0,
                length: 1.0,
                origin: Vector3 {
                    x: 2.5,
                    y: 4.0,
                    z: 0.0,
                }
            }
        );
    }

    #[test]
    fn test_rectangle_rectangle_snap_y() {
        let mut collider_1 = RectangleCollider {
            width: 3.0,
            height: 6.0,
            length: 1.0,
            origin: Vector3 {
                x: 3.5,
                y: 4.0,
                z: 0.0,
            },
        };

        let collider_2 = RectangleCollider {
            width: 4.0,
            height: 4.0,
            length: 1.0,
            origin: Vector3 {
                x: 6.0,
                y: 7.0,
                z: 0.0,
            },
        };

        assert!(collider_1.is_colliding(&collider_2));

        collider_1.snap_y(&collider_2);

        assert_eq!(
            collider_1,
            RectangleCollider {
                width: 3.0,
                height: 6.0,
                length: 1.0,
                origin: Vector3 {
                    x: 3.5,
                    y: 2.0,
                    z: 0.0,
                }
            }
        );
    }

    #[test]
    fn test_rectangle_rectangle_snap_z() {
        let mut collider_1 = RectangleCollider {
            width: 3.0,
            height: 6.0,
            length: 1.0,
            origin: Vector3 {
                x: 3.5,
                y: 4.0,
                z: 0.0,
            },
        };

        let collider_2 = RectangleCollider {
            width: 4.0,
            height: 4.0,
            length: 1.0,
            origin: Vector3 {
                x: 6.0,
                y: 7.0,
                z: 0.0,
            },
        };

        assert!(collider_1.is_colliding(&collider_2));

        collider_1.snap_z(&collider_2);

        assert_eq!(
            collider_1,
            RectangleCollider {
                width: 3.0,
                height: 6.0,
                length: 1.0,
                origin: Vector3 {
                    x: 3.5,
                    y: 4.0,
                    z: 0.0,
                }
            }
        );
    }

    #[test]
    fn test_rectangle_plane_collision() {
        let rectangle_collider = RectangleCollider {
            width: 5.0,
            height: 5.0,
            length: 5.0,
            origin: Vector3 {
                x: 0.0,
                y: 2.5,
                z: 0.0,
            },
        };

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

        let rectangle_collider = RectangleCollider {
            width: 5.0,
            height: 5.0,
            length: 5.0,
            origin: Vector3 {
                x: 0.0,
                y: 6.0,
                z: 0.0,
            },
        };

        assert!(!rectangle_collider.is_colliding(&plane_collider));
    }
}
