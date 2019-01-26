#[cfg(feature = "rayon")]
use rayon::prelude::*;

const STACK_CAPACITY: usize = 100;

/// Option<usize>, where None is represented by -1
///
/// Takes 8 bytes instead of 16.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OptionIndex(usize);

impl OptionIndex {
    /// Returns `Some(idx)` value
    pub fn some(idx: usize) -> OptionIndex {
        debug_assert!(idx < std::usize::MAX);
        OptionIndex(idx)
    }

    /// Returns None value
    pub fn none() -> OptionIndex {
        OptionIndex(std::usize::MAX)
    }

    /// Returns true if it is a `Some` value
    pub fn is_some(self) -> bool {
        self != OptionIndex::none()
    }

    /// Returns true if it is a `None` value
    pub fn is_none(self) -> bool {
        self == OptionIndex::none()
    }

    /// Returns the associated `Option` value
    pub fn get(self) -> Option<usize> {
        if self.is_some() {
            Some(self.0)
        } else {
            None
        }
    }
}

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
    pub fn distance_sq(self, other: Point) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    /// Returns true if points are approximately equal
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

#[derive(Clone, Copy, Debug, PartialEq)]
struct Triangle(pub Point, pub Point, pub Point);

impl Triangle {
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

    fn circumradius_sq(self) -> f32 {
        let (x, y) = self.circumcircle_delta();
        x * x + y * y
    }

    fn circumcenter(self) -> Point {
        let (x, y) = self.circumcircle_delta();

        Point {
            x: x + self.0.x,
            y: y + self.0.y,
        }
    }

    /// Returns the cross product of vectors 1--0 and 1--2
    fn orientation(self) -> f32 {
        let v21x = self.0.x - self.1.x;
        let v21y = self.0.y - self.1.y;
        let v23x = self.2.x - self.1.x;
        let v23y = self.2.y - self.1.y;
        v21x * v23y - v21y * v23x
    }

    fn is_right_handed(self) -> bool {
        self.orientation() > 0.0
    }

    fn is_left_handed(self) -> bool {
        self.orientation() < 0.0
    }

    fn in_circumcircle(self, point: Point) -> bool {
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
fn pseudo_angle(dx: f32, dy: f32) -> f32 {
    let p = dx / (dx.abs() + dy.abs());

    if dy > 0.0 {
        (3.0 - p) / 4.0
    } else {
        (1.0 + p) / 4.0
    }
}

/// Maps angle between `point` and `center` to index in the hash table
fn angular_hash(point: Point, center: Point, size: usize) -> usize {
    let angle = pseudo_angle(point.x - center.x, point.y - center.y);
    (angle * size as f32) as usize % size
}

/// Counter-clockwise convex hull
struct Hull {
    /// Maps point index to next point index
    next: Vec<usize>,

    /// Maps point index to previous point index
    prev: Vec<usize>,

    /// Radial hash table
    hash_table: Vec<OptionIndex>,

    /// Boundary triangles
    triangles: Vec<OptionIndex>,

    /// Center point for calculating radial hash
    center: Point,

    /// Starting point index
    start: usize,
}

impl Hull {
    fn new(seed: [usize; 3], points: &[Point]) -> Hull {
        let capacity = points.len();
        let table_size = (capacity as f32).sqrt().ceil() as usize;

        let center = Triangle(points[seed[0]], points[seed[1]], points[seed[2]]).circumcenter();

        let mut hull = Hull {
            next: vec![0; capacity],
            prev: vec![0; capacity],
            hash_table: vec![OptionIndex::none(); table_size],
            triangles: vec![OptionIndex::none(); capacity],
            start: seed[0],
            center,
        };

        hull.next[seed[0]] = seed[1];
        hull.next[seed[1]] = seed[2];
        hull.next[seed[2]] = seed[0];

        hull.prev[seed[0]] = seed[2];
        hull.prev[seed[1]] = seed[0];
        hull.prev[seed[2]] = seed[1];

        hull.triangles[seed[0]] = OptionIndex::some(0);
        hull.triangles[seed[1]] = OptionIndex::some(1);
        hull.triangles[seed[2]] = OptionIndex::some(2);

        hull.add_hash(seed[0], points[seed[0]]);
        hull.add_hash(seed[1], points[seed[1]]);
        hull.add_hash(seed[2], points[seed[2]]);

        hull
    }

    /// Adds a new point in the hash table
    fn add_hash(&mut self, index: usize, point: Point) {
        let table_size = self.hash_table.len();
        self.hash_table[angular_hash(point, self.center, table_size)] = OptionIndex::some(index);
    }

    /// Returns the first convex hull edge visible from the point and a boolean
    /// indicating whether the previous edge may be visible too
    fn find_visible_edge(&self, point: Point, points: &[Point]) -> Option<(usize, bool)> {
        let table_size = self.hash_table.len();
        let hash = angular_hash(point, self.center, table_size);

        let mut start = OptionIndex::none();

        // basically linear probing hash table
        for i in 0..table_size {
            start = self.hash_table[(hash + i) % table_size];

            // if e == self.next[e] then it is an empty hash table entry; skip it
            if start.get().filter(|&e| e != self.next[e]).is_some() {
                break;
            }
        }

        // now `start` is a point near enough to the target
        // let's go forward to find a visible edge

        let start = self.prev[start.get()?];
        let mut edge = start;

        loop {
            let next = self.next[edge];
            let tri = Triangle(point, points[edge], points[next]);

            if tri.is_left_handed() {
                // edge is visible, breakin' outta hell
                break;
            }

            edge = next;
            if edge == start {
                // avoiding the endless loop
                return None;
            }
        }

        // if edge == start then we made 0 iterations, so we can't say for sure
        // that there are no visible edges preceding the start one

        Some((edge, edge == start))
    }

    fn add_point(&mut self, index: usize, triangulation: &mut Delaunay, points: &[Point]) {
        let point = points[index];

        let (mut start, should_walk_back) = match self.find_visible_edge(point, points) {
            Some(v) => v,
            None => return,
        };

        let mut end = self.next[start];

        let t = triangulation.add_triangle(
            [start, index, end],
            [
                OptionIndex::none(),
                OptionIndex::none(),
                self.triangles[start],
            ],
        );

        self.triangles[index] = OptionIndex::some(triangulation.legalize(t + 2, points, self));
        self.triangles[start] = OptionIndex::some(t);

        loop {
            let next = self.next[end];
            let tri = Triangle(point, points[next], points[end]);
            if !tri.is_right_handed() {
                break;
            }

            let t = triangulation.add_triangle(
                [end, index, next],
                [
                    self.triangles[index],
                    OptionIndex::none(),
                    self.triangles[end],
                ],
            );

            self.triangles[index] = OptionIndex::some(triangulation.legalize(t + 2, points, self));
            self.next[end] = end;
            end = next;
        }

        if should_walk_back {
            loop {
                let prev = self.prev[start];
                let tri = Triangle(point, points[start], points[prev]);
                if !tri.is_right_handed() {
                    break;
                }

                let t = triangulation.add_triangle(
                    [prev, index, start],
                    [
                        OptionIndex::none(),
                        self.triangles[start],
                        self.triangles[prev],
                    ],
                );

                triangulation.legalize(t + 2, points, self);

                self.triangles[prev] = OptionIndex::some(t);
                self.next[start] = start;
                start = prev;
            }
        }

        self.start = start;
        self.next[start] = index;
        self.next[index] = end;

        self.prev[end] = index;
        self.prev[index] = start;

        self.add_hash(index, point);
        self.add_hash(start, points[start]);
    }
}

/// Calculates the median point (arithmetic mean of the coordinates)
fn find_center(points: &[Point]) -> Point {
    let (x_sum, y_sum) = points
        .iter()
        .fold((0.0, 0.0), |(x, y), point| (x + point.x, y + point.y));

    Point::new(x_sum / points.len() as f32, y_sum / points.len() as f32)
}

fn find_seed_triangle(points: &[Point]) -> Option<(Triangle, [usize; 3])> {
    let center = find_center(&points);

    #[cfg(feature = "rayon")]
    let iter = points.par_iter();

    #[cfg(not(feature = "rayon"))]
    let iter = points.iter();

    let (seed_idx, seed) = iter.clone().cloned().enumerate().min_by(|(_, a), (_, b)| {
        a.distance_sq(center)
            .partial_cmp(&b.distance_sq(center))
            .unwrap()
    })?;

    let (nearest_idx, nearest, _) = iter
        .clone()
        .cloned()
        .enumerate()
        .filter(|&(i, _)| i != seed_idx)
        .map(|(i, p)| (i, p, p.distance_sq(seed)))
        .filter(|(_, _, d)| d.abs() > std::f32::EPSILON)
        .min_by(|(_, _, a), (_, _, b)| a.partial_cmp(&b).unwrap())?;

    let (third_idx, third) = iter
        .cloned()
        .enumerate()
        .filter(|&(i, _)| i != seed_idx && i != nearest_idx)
        .min_by(|&(_, a), &(_, b)| {
            let t0 = Triangle(seed, nearest, a);
            let t1 = Triangle(seed, nearest, b);

            t0.circumradius_sq()
                .partial_cmp(&t1.circumradius_sq())
                .unwrap()
        })?;

    let tri = Triangle(seed, nearest, third);

    if tri.is_right_handed() {
        Some((tri, [seed_idx, nearest_idx, third_idx]))
    } else {
        let tri = Triangle(seed, third, nearest);
        Some((tri, [seed_idx, third_idx, nearest_idx]))
    }
}

/// Delaunay triangulation represented by DCEL (doubly connected edge list)
pub struct Delaunay {
    /// Maps edge id to start point id
    pub triangles: Vec<usize>,

    /// Maps edge id to the opposite edge id in the adjacent triangle, if it exists
    pub halfedges: Vec<OptionIndex>,

    stack: Vec<usize>,
}

impl Delaunay {
    /// Creates delaunay triangulation of given points, if it exists
    ///
    /// Delaunay triangulation does not exist if and only if all points lie on the same line
    /// or there are less than three points.
    pub fn new(points: &[Point]) -> Option<Delaunay> {
        let (seed, seed_indices) = find_seed_triangle(points)?;
        let seed_circumcenter = seed.circumcenter();

        let mut indices = (0..points.len())
            .filter(|&i| i != seed_indices[0] && i != seed_indices[1] && i != seed_indices[2])
            .collect::<Vec<_>>();

        let cmp = |&a: &usize, &b: &usize| {
            points[a]
                .distance_sq(seed_circumcenter)
                .partial_cmp(&points[b].distance_sq(seed_circumcenter))
                .unwrap()
        };

        #[cfg(feature = "rayon")]
        indices.par_sort_by(cmp);

        #[cfg(not(feature = "rayon"))]
        indices.sort_by(cmp);

        let mut hull = Hull::new(seed_indices, points);

        let max_triangles = 2 * points.len() - 3 - 2;
        let mut triangulation = Delaunay::with_capacity(max_triangles);

        triangulation.add_triangle(seed_indices, [OptionIndex::none(); 3]);

        let mut prev_point: Option<Point> = None;

        for &i in &indices {
            let point = points[i];

            if let Some(p) = prev_point {
                if p.approx_eq(point) {
                    continue;
                }
            }

            hull.add_point(i, &mut triangulation, points);
            prev_point = Some(point);
        }

        triangulation.stack.shrink_to_fit();
        Some(triangulation)
    }

    fn with_capacity(cap: usize) -> Delaunay {
        Delaunay {
            triangles: Vec::with_capacity(cap * 3),
            halfedges: vec![OptionIndex::none(); cap * 3],
            stack: Vec::with_capacity(STACK_CAPACITY),
        }
    }

    fn add_triangle(&mut self, vertices: [usize; 3], halfedges: [OptionIndex; 3]) -> usize {
        let t = self.triangles.len();

        self.triangles.push(vertices[0]);
        self.triangles.push(vertices[1]);
        self.triangles.push(vertices[2]);

        for (i, &halfedge) in halfedges.iter().enumerate() {
            if let Some(e) = halfedge.get() {
                self.halfedges[t + i] = OptionIndex::some(e);
                self.halfedges[e] = OptionIndex::some(t + i);
            }
        }

        t
    }

    fn legalize(&mut self, index: usize, points: &[Point], hull: &mut Hull) -> usize {
        self.stack.push(index);

        let mut ar = 0;

        while let Some(a) = self.stack.pop() {
            let a0 = a - a % 3;
            ar = a0 + (a + 2) % 3;

            let b = match self.halfedges[a].get() {
                Some(v) => v,
                None => continue,
            };

            let b0 = b - b % 3;
            let al = a0 + (a + 1) % 3;
            let bl = b0 + (b + 2) % 3;

            let p0 = self.triangles[ar];
            let pr = self.triangles[a];
            let pl = self.triangles[al];
            let p1 = self.triangles[bl];

            let illegal = Triangle(points[p0], points[pr], points[pl]).in_circumcircle(points[p1]);

            if !illegal {
                continue;
            }

            self.triangles[a] = p1;
            self.triangles[b] = p0;

            let hbl = self.halfedges[bl];

            self.halfedges[a] = hbl;

            if let Some(e) = hbl.get() {
                self.halfedges[e] = OptionIndex::some(a);
            } else {
                let mut edge = hull.start;

                loop {
                    if hull.triangles[edge] == OptionIndex::some(bl) {
                        hull.triangles[edge] = OptionIndex::some(a);
                        break;
                    }

                    edge = hull.next[edge];

                    if edge == hull.start {
                        break;
                    }
                }
            }

            self.halfedges[b] = self.halfedges[ar];
            if let Some(e) = self.halfedges[ar].get() {
                self.halfedges[e] = OptionIndex::some(b);
            }

            self.halfedges[ar] = OptionIndex::some(bl);
            self.halfedges[bl] = OptionIndex::some(ar);

            let br = b0 + (b + 1) % 3;

            if self.stack.len() >= STACK_CAPACITY - 1 {
                continue;
            }

            self.stack.push(br);
            self.stack.push(a);
        }

        ar
    }
}
