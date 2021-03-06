use core::marker::PhantomData;

#[cfg(feature = "rayon")]
use rayon::prelude::*;

pub mod dcel;
pub mod geom;

pub use dcel::{EdgeIndex, PointIndex, TrianglesDCEL};
pub use geom::{Point, Triangle};

const STACK_CAPACITY: usize = 512;

/// Option<usize>, where None is represented by usize::MAX.
///
/// Takes 8 bytes instead of 16.
#[derive(Clone, Copy, Eq, Hash, Ord)]
pub struct OptionIndex<T: Into<usize> + From<usize>>(usize, PhantomData<T>);

impl<T: Into<usize> + From<usize>> OptionIndex<T> {
    /// Returns `Some(idx)` value
    #[inline]
    pub fn some(idx: T) -> OptionIndex<T> {
        let idx = idx.into();
        debug_assert!(idx < std::usize::MAX);
        OptionIndex(idx, PhantomData)
    }

    /// Returns None value
    #[inline]
    pub fn none() -> OptionIndex<T> {
        OptionIndex(std::usize::MAX, PhantomData)
    }

    /// Returns true if it is a `Some` value
    #[inline]
    pub fn is_some(&self) -> bool {
        self != &OptionIndex::none()
    }

    /// Returns true if it is a `None` value
    #[inline]
    pub fn is_none(&self) -> bool {
        self == &OptionIndex::none()
    }

    /// Returns the associated `Option` value
    #[inline]
    pub fn get(&self) -> Option<T> {
        if self.is_some() {
            Some(self.0.into())
        } else {
            None
        }
    }
}

impl<T: Into<usize> + From<usize>> PartialEq for OptionIndex<T> {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl<T: Into<usize> + From<usize>> PartialOrd for OptionIndex<T> {
    fn partial_cmp(&self, rhs: &Self) -> Option<core::cmp::Ordering> {
        self.0.partial_cmp(&rhs.0)
    }
}

impl<T: Into<usize> + From<usize> + std::fmt::Debug> std::fmt::Debug for OptionIndex<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.get().fmt(f)
    }
}

/// Maps angle between `point` and `center` to index in the hash table
fn angular_hash(point: Point, center: Point, size: usize) -> usize {
    let angle = geom::pseudo_angle(point.x - center.x, point.y - center.y);
    (angle * size as f32) as usize % size
}

/// Counter-clockwise convex hull
struct Hull {
    /// Maps point index to next point index
    next: Vec<PointIndex>,

    /// Maps point index to previous point index
    prev: Vec<PointIndex>,

    /// Radial hash table
    hash_table: Vec<OptionIndex<PointIndex>>,

    /// Boundary triangles
    triangles: Vec<OptionIndex<EdgeIndex>>,

    /// Center point for calculating radial hash
    center: Point,

    /// Starting point index
    start: PointIndex,
}

impl Hull {
    fn new(seed: [PointIndex; 3], points: &[Point]) -> Hull {
        let capacity = points.len();
        let table_size = (capacity as f32).sqrt().ceil() as usize;

        let center = Triangle(points[seed[0]], points[seed[1]], points[seed[2]]).circumcenter();

        let mut hull = Hull {
            next: vec![0.into(); capacity],
            prev: vec![0.into(); capacity],
            hash_table: vec![OptionIndex::none(); table_size],
            triangles: vec![OptionIndex::none(); capacity],
            start: seed[0],
            center,
        };

        hull.next[seed[0].as_usize()] = seed[1];
        hull.next[seed[1].as_usize()] = seed[2];
        hull.next[seed[2].as_usize()] = seed[0];

        hull.prev[seed[0].as_usize()] = seed[2];
        hull.prev[seed[1].as_usize()] = seed[0];
        hull.prev[seed[2].as_usize()] = seed[1];

        hull.triangles[seed[0].as_usize()] = OptionIndex::some(0.into());
        hull.triangles[seed[1].as_usize()] = OptionIndex::some(1.into());
        hull.triangles[seed[2].as_usize()] = OptionIndex::some(2.into());

        hull.add_hash(seed[0], points[seed[0]]);
        hull.add_hash(seed[1], points[seed[1]]);
        hull.add_hash(seed[2], points[seed[2]]);

        hull
    }

    /// Adds a new point in the hash table
    fn add_hash(&mut self, index: PointIndex, point: Point) {
        let table_size = self.hash_table.len();
        self.hash_table[angular_hash(point, self.center, table_size)] = OptionIndex::some(index);
    }

    /// Returns the index of the ending point of first convex hull edge visible
    /// from the point and a boolean indicating whether the previous edge may be
    /// visible too
    fn find_visible_edge(&self, point: Point, points: &[Point]) -> Option<(PointIndex, bool)> {
        let table_size = self.hash_table.len();
        let hash = angular_hash(point, self.center, table_size);

        let mut start = OptionIndex::none();

        // basically linear probing hash table
        for i in 0..table_size {
            start = self.hash_table[(hash + i) % table_size];

            // if e == self.next[e] then it is an empty hash table entry; skip it
            if start.get().filter(|&e| e != self.next[e.as_usize()]).is_some() {
                break;
            }
        }

        // now `start` is a point near enough to the target
        // let's go forward to find a visible edge

        let start = self.prev[start.get()?.as_usize()];
        let mut edge = start;

        loop {
            let next = self.next[edge.as_usize()];
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
}

/// Calculates the median point (arithmetic mean of the coordinates)
fn find_center(points: &[Point]) -> Point {
    let (x_sum, y_sum) = points
        .iter()
        .fold((0.0, 0.0), |(x, y), point| (x + point.x, y + point.y));

    Point::new(x_sum / points.len() as f32, y_sum / points.len() as f32)
}

fn find_seed_triangle(points: &[Point]) -> Option<(Triangle, [PointIndex; 3])> {
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
        Some((tri, [seed_idx.into(), nearest_idx.into(), third_idx.into()]))
    } else {
        let tri = Triangle(seed, third, nearest);
        Some((tri, [seed_idx.into(), third_idx.into(), nearest_idx.into()]))
    }
}

/// Delaunay triangulation
pub struct Delaunay {
    pub dcel: TrianglesDCEL,
    hull: Hull,
    stack: Vec<EdgeIndex>,
}

impl Delaunay {
    /// Triangulates a set of given points, if it is possible.
    pub fn new(points: &[Point]) -> Option<Delaunay> {
        let (seed, seed_indices) = find_seed_triangle(points)?;
        let seed_circumcenter = seed.circumcenter();

        let mut indices = (0..points.len())
            .map(|i| PointIndex::from(i))
            .filter(|&i| i != seed_indices[0] && i != seed_indices[1] && i != seed_indices[2])
            .collect::<Vec<_>>();

        let cmp = |&a: &PointIndex, &b: &PointIndex| {
            points[a]
                .distance_sq(seed_circumcenter)
                .partial_cmp(&points[b].distance_sq(seed_circumcenter))
                .unwrap()
        };

        #[cfg(feature = "rayon")]
        indices.par_sort_by(cmp);

        #[cfg(not(feature = "rayon"))]
        indices.sort_by(cmp);

        let max_triangles = 2 * points.len() - 3 - 2;

        let mut delaunay = Delaunay {
            dcel: TrianglesDCEL::with_capacity(max_triangles),
            hull: Hull::new(seed_indices, points),
            stack: Vec::with_capacity(STACK_CAPACITY),
        };

        delaunay.dcel.add_triangle(seed_indices);

        let mut prev_point: Option<Point> = None;

        for &i in &indices {
            let point = points[i];

            if let Some(p) = prev_point {
                if p.approx_eq(point) {
                    continue;
                }
            }

            delaunay.add_point(i, points);
            prev_point = Some(point);
        }

        Some(delaunay)
    }

    fn add_point(&mut self, index: PointIndex, points: &[Point]) {
        let point = points[index];

        let (mut start, should_walk_back) = match self.hull.find_visible_edge(point, points) {
            Some(v) => v,
            None => return,
        };

        let mut end = self.hull.next[start.as_usize()];

        let t = self.add_triangle(
            [start, index, end],
            [
                OptionIndex::none(),
                OptionIndex::none(),
                self.hull.triangles[start.as_usize()],
            ],
        );

        self.hull.triangles[index.as_usize()] = OptionIndex::some(self.legalize(t + 2, points));
        self.hull.triangles[start.as_usize()] = OptionIndex::some(t);

        loop {
            let next = self.hull.next[end.as_usize()];
            let tri = Triangle(point, points[next], points[end]);
            if !tri.is_right_handed() {
                break;
            }

            let t = self.add_triangle(
                [end, index, next],
                [
                    self.hull.triangles[index.as_usize()],
                    OptionIndex::none(),
                    self.hull.triangles[end.as_usize()],
                ],
            );

            self.hull.triangles[index.as_usize()] = OptionIndex::some(self.legalize(t + 2, points));
            self.hull.next[end.as_usize()] = end;
            end = next;
        }

        if should_walk_back {
            loop {
                let prev = self.hull.prev[start.as_usize()];
                let tri = Triangle(point, points[start], points[prev]);
                if !tri.is_right_handed() {
                    break;
                }

                let t = self.add_triangle(
                    [prev, index, start],
                    [
                        OptionIndex::none(),
                        self.hull.triangles[start.as_usize()],
                        self.hull.triangles[prev.as_usize()],
                    ],
                );

                self.legalize(t + 2, points);

                self.hull.triangles[prev.as_usize()] = OptionIndex::some(t);
                self.hull.next[start.as_usize()] = start;
                start = prev;
            }
        }

        self.hull.start = start;
        self.hull.next[start.as_usize()] = index;
        self.hull.next[index.as_usize()] = end;

        self.hull.prev[end.as_usize()] = index;
        self.hull.prev[index.as_usize()] = start;

        self.hull.add_hash(index, point);
        self.hull.add_hash(start, points[start]);
    }

    fn add_triangle(&mut self, vertices: [PointIndex; 3], halfedges: [OptionIndex<EdgeIndex>; 3]) -> EdgeIndex {
        let t = self.dcel.add_triangle(vertices);

        for (i, &halfedge) in halfedges.iter().enumerate() {
            if let Some(e) = halfedge.get() {
                self.dcel.link(t + i, e);
            }
        }

        t
    }

    fn legalize(&mut self, index: EdgeIndex, points: &[Point]) -> EdgeIndex {
        self.stack.push(index);

        let mut output = 0.into();

        while let Some(a) = self.stack.pop() {
            let ar = self.dcel.prev_edge(a);
            output = ar;

            let b = match self.dcel.twin(a) {
                Some(v) => v,
                None => continue,
            };

            let br = self.dcel.next_edge(b);
            let bl = self.dcel.prev_edge(b);

            /* if the pair of triangles doesn't satisfy the Delaunay condition
             * (p1 is inside the circumcircle of [p0, pl, pr]), flip them,
             * then do the same check/flip recursively for the new pair of triangles
             *
             *           pl                    pl
             *          /||\                  /  \
             *       al/ || \bl            al/    \a
             *        /  ||  \              /      \
             *       /  a||b  \    flip    /___ar___\
             *     p0\   ||   /p1   =>   p0\---bl---/p1
             *        \  ||  /              \      /
             *       ar\ || /br             b\    /br
             *          \||/                  \  /
             *           pr                    pr
             */

            let [p0, pr, pl] = self.dcel.triangle_points(ar);
            let p1 = self.dcel.triangle_points(bl)[0];

            let illegal = Triangle(points[p0], points[pr], points[pl]).in_circumcircle(points[p1]);

            if !illegal {
                continue;
            }

            self.dcel.vertices[a] = p1;
            self.dcel.vertices[b] = p0;

            let hbl = self.dcel.twin(bl);

            self.dcel.link_option(a, hbl);
            self.dcel.link_option(b, self.dcel.twin(ar));
            self.dcel.link(ar, bl);

            if hbl.is_none() {
                let mut edge: EdgeIndex = self.hull.start.as_usize().into();

                loop {
                    if self.hull.triangles[edge] == OptionIndex::some(bl) {
                        self.hull.triangles[edge] = OptionIndex::some(a);
                        break;
                    }

                    edge = self.hull.next[edge].as_usize().into();

                    if edge.as_usize() == self.hull.start.as_usize() || edge.as_usize() == self.hull.next[edge].as_usize() {
                        break;
                    }
                }
            }

            if self.stack.len() >= STACK_CAPACITY - 1 {
                continue;
            }

            self.stack.push(br);
            self.stack.push(a);
        }

        output
    }
}
