/// 2D point represented by x and y coordinates
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    /// Creates a new point
    pub fn new(x: f32, y: f32) -> Point {
        Point { x, y }
    }

    /// Returns square of the distance between `self` and `other` point
    ///
    /// # Examples
    /// ```
    /// # use triangulation::Point;
    /// let a = Point::new(10.0, 10.0);
    /// let b = Point::new(10.0, 110.0);
    /// assert!((a.distance_sq(b) - 10000.0) < 1e-6);
    /// ```
    #[inline]
    pub fn distance_sq(self, other: Point) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    /// Returns true if points are approximately equal
    ///
    /// # Examples
    ///
    /// ```
    /// # use triangulation::Point;
    /// let a = Point::new(10.0, 10.0);
    /// let b = Point::new(10.0, 10.0000001);
    /// assert!(a.approx_eq(b))
    /// ```
    #[inline]
    pub fn approx_eq(self, other: Point) -> bool {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx.abs() <= std::f32::EPSILON && dy.abs() <= std::f32::EPSILON
    }
}

impl Into<(i32, i32)> for Point {
    fn into(self) -> (i32, i32) {
        (self.x as i32, self.y as i32)
    }
}

impl Into<(f32, f32)> for Point {
    fn into(self) -> (f32, f32) {
        (self.x, self.y)
    }
}

/// A triangle made of 3 points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Triangle(pub Point, pub Point, pub Point);

impl Triangle {
    #[inline]
    fn circumcircle_delta(self) -> (f32, f32) {
        let p = Point {
            x: self.1.x - self.0.x,
            y: self.1.y - self.0.y,
        };

        let q = Point {
            x: self.2.x - self.0.x,
            y: self.2.y - self.0.y,
        };

        let p2 = p.x * p.x + p.y * p.y;
        let q2 = q.x * q.x + q.y * q.y;
        let d = 2.0 * (p.x * q.y - p.y * q.x);

        if d == 0.0 {
            return (std::f32::INFINITY, std::f32::INFINITY);
        }

        let dx = (q.y * p2 - p.y * q2) / d;
        let dy = (p.x * q2 - q.x * p2) / d;

        (dx, dy)
    }

    /// Returns square of the circumcircle radius.
    ///
    /// # Examples
    /// ```
    /// # use triangulation::{Triangle, Point};
    /// let t = Triangle(
    ///     Point::new(10.0, 10.0),
    ///     Point::new(10.0, 110.0),
    ///     Point::new(110.0, 10.0)
    /// );
    /// assert!((t.circumradius_sq() - 5000.0) < 1e-6);
    /// ```
    #[inline]
    pub fn circumradius_sq(self) -> f32 {
        let (x, y) = self.circumcircle_delta();
        x * x + y * y
    }

    /// Returns the circumcenter.
    ///
    /// # Examples
    /// ```
    /// # use triangulation::{Triangle, Point};
    /// let t = Triangle(
    ///     Point::new(10.0, 10.0),
    ///     Point::new(10.0, 110.0),
    ///     Point::new(110.0, 10.0)
    /// );
    /// assert!(t.circumcenter().approx_eq(Point::new(60.0, 60.0)));
    /// ```
    #[inline]
    pub fn circumcenter(self) -> Point {
        let (x, y) = self.circumcircle_delta();

        Point {
            x: x + self.0.x,
            y: y + self.0.y,
        }
    }

    /// Returns the cross product of vectors 1--0 and 1--2
    ///
    /// # Examples
    /// ```
    /// # use triangulation::{Triangle, Point};
    ///
    /// let t = Triangle(
    ///     Point::new(10.0, 10.0),
    ///     Point::new(10.0, 110.0),
    ///     Point::new(110.0, 10.0)
    /// );
    /// assert!(t.orientation() > 0.0);
    /// ```
    #[inline]
    pub fn orientation(self) -> f32 {
        let v21x = self.0.x - self.1.x;
        let v21y = self.0.y - self.1.y;
        let v23x = self.2.x - self.1.x;
        let v23y = self.2.y - self.1.y;
        v21x * v23y - v21y * v23x
    }

    /// Returns true if the triangle is right-handed (conter-clockwise order).
    #[inline]
    pub fn is_right_handed(self) -> bool {
        self.orientation() > 0.0
    }

    /// Returns true if the triangle is left-handed (clockwise order).
    #[inline]
    pub fn is_left_handed(self) -> bool {
        self.orientation() < 0.0
    }

    /// Returns true if the given point lies inside the circumcircle of the triangle.
    ///
    /// # Examples
    /// ```
    /// # use triangulation::{Triangle, Point};
    ///
    /// let t = Triangle(
    ///     Point::new(10.0, 10.0),
    ///     Point::new(10.0, 110.0),
    ///     Point::new(110.0, 10.0)
    /// );
    /// assert!(t.in_circumcircle(Point::new(30.0, 30.0)));
    /// assert!(!t.in_circumcircle(Point::new(5.0, 5.0)));
    /// ```
    #[inline]
    pub fn in_circumcircle(self, point: Point) -> bool {
        let dx = self.0.x - point.x;
        let dy = self.0.y - point.y;
        let ex = self.1.x - point.x;
        let ey = self.1.y - point.y;
        let fx = self.2.x - point.x;
        let fy = self.2.y - point.y;

        let ap = dx * dx + dy * dy;
        let bp = ex * ex + ey * ey;
        let cp = fx * fx + fy * fy;

        dx * (ey * cp - bp * fy) - dy * (ex * cp - bp * fx) + ap * (ex * fy - ey * fx) < 0.0
    }
}

/// Monotonically increases with the real angle, returns vales in range [0; 1]
///
/// # Examples
/// ```
/// # use triangulation::geom::pseudo_angle;
/// let a = pseudo_angle(1.0, 1.0);  // 45 degrees
/// let b = pseudo_angle(2.0, 1.0);  // 26 degrees
/// assert!(a > b);
/// ```
pub fn pseudo_angle(dx: f32, dy: f32) -> f32 {
    let p = dx / (dx.abs() + dy.abs());

    if dy > 0.0 {
        (3.0 - p) / 4.0
    } else {
        (1.0 + p) / 4.0
    }
}
