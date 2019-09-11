//! Rectangles.

use cgmath::{EuclideanSpace, Point2, Vector2, Zero};
use std::{f64, ops};
use swift_birb::protocol::{SBRect, SBVector2};

/// A rectangle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// Rectangle origin.
    pub origin: Point2<f64>,

    /// Rectangle size.
    pub size: Vector2<f64>,
}

impl Rect {
    /// Creates a new rectangle.
    pub fn new(origin: Point2<f64>, size: Vector2<f64>) -> Rect {
        Rect { origin, size }
    }

    /// Returns a zero-sized rectangle at the origin.
    pub fn zero() -> Rect {
        Rect {
            origin: Point2::new(0., 0.),
            size: Vector2::zero(),
        }
    }

    /// Returns the center point.
    pub fn center(&self) -> Point2<f64> {
        self.origin + self.size / 2.
    }

    /// Returns true if the point is inside the rectangle.
    pub fn contains(&self, point: Point2<f64>) -> bool {
        point.x >= self.origin.x
            && point.y >= self.origin.y
            && point.x < self.origin.x + self.size.x
            && point.y < self.origin.y + self.size.y
    }

    /// Returns true if the two rectangles intersect.
    pub fn intersects(&self, rect: Rect) -> bool {
        let own_opposite = self.origin + self.size;
        let rect_opposite = rect.origin + rect.size;

        self.origin.x < rect_opposite.x
            && self.origin.y < rect_opposite.y
            && rect.origin.x < own_opposite.x
            && rect.origin.y < own_opposite.y
    }

    /// Returns the intersection rectangle.
    pub fn intersect(&self, rect: Rect) -> Option<Rect> {
        if !self.intersects(rect) {
            return None;
        }

        let min_x = self.origin.x.max(rect.origin.x);
        let min_y = self.origin.y.max(rect.origin.y);
        let max_x = (self.origin.x + self.size.x).min(rect.origin.x + rect.size.x);
        let max_y = (self.origin.y + self.size.y).min(rect.origin.y + rect.size.y);

        Some(Rect {
            origin: (min_x, min_y).into(),
            size: (max_x - min_x, max_y - min_y).into(),
        })
    }

    /// Returns a new rectangle inset by the specified amount.
    pub fn inset(&self, horiz: f64, vert: f64) -> Rect {
        Rect {
            origin: (self.origin.x + horiz, self.origin.y + vert).into(),
            size: (self.size.x - 2. * horiz, self.size.y - 2. * vert).into(),
        }
    }

    /// Returns a new rectangle with the given origin.
    pub fn with_origin(&self, origin: Point2<f64>) -> Rect {
        Rect {
            origin,
            size: self.size,
        }
    }

    /// Returns a new rectangle with the given size added to the current size.
    pub fn with_added_size(self, size: Vector2<f64>) -> Rect {
        Rect {
            origin: self.origin,
            size: self.size + size,
        }
    }
}

impl ops::Add<Point2<f64>> for Rect {
    type Output = Rect;
    fn add(self, point: Point2<f64>) -> Rect {
        Rect {
            origin: self.origin + point.to_vec(),
            size: self.size,
        }
    }
}

impl Into<SBRect> for Rect {
    fn into(self) -> SBRect {
        SBRect {
            origin: SBVector2 {
                x: self.origin.x,
                y: self.origin.y,
            },
            size: SBVector2 {
                x: self.size.x,
                y: self.size.y,
            },
        }
    }
}
