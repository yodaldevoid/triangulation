use fnv::FnvHashMap;
use ordered_float::NotNan;
use rayon::prelude::*;
use serde_derive::Serialize;

type Scalar = NotNan<f32>;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Point {
    pub x: Scalar,
    pub y: Scalar,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Point {
        Point {
            x: NotNan::new(x).unwrap(),
            y: NotNan::new(y).unwrap(),
        }
    }

    pub fn distance_sq(&self, other: &Point) -> Scalar {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    pub fn sort(a: &mut Point, b: &mut Point) {
        if a.y < b.y {
            return;
        }

        if a.y == b.y && a.x < a.y {
            return;
        }

        std::mem::swap(a, b);
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
pub struct Circumcircle {
    pub radius_sq: Scalar,
    pub center: Point,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
pub struct Triangle(pub Point, pub Point, pub Point);

impl Triangle {
    fn circumcircle_xy(&self) -> (Scalar, Scalar) {
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
        let d = Scalar::new(2.0).unwrap() * (p.x * q.y - p.y * q.x);

        if d == Scalar::new(0.0).unwrap() {
            let inf = Scalar::new(std::f32::INFINITY).unwrap();
            return (inf, inf);
        }

        let x = (q.y * p2 - p.y * q2) / d;
        let y = (p.x * q2 - q.x * p2) / d;

        (x, y)
    }

    pub fn circumradius_sq(&self) -> Scalar {
        let (x, y) = self.circumcircle_xy();
        x * x + y * y
    }

    pub fn circumcenter(&self) -> Point {
        let (x, y) = self.circumcircle_xy();

        Point {
            x: x + self.0.x,
            y: y + self.0.y,
        }
    }

    pub fn circumcircle(&self) -> Circumcircle {
        let (x, y) = self.circumcircle_xy();

        Circumcircle {
            radius_sq: x * x + y * y,
            center: Point {
                x: x + self.0.x,
                y: y + self.0.y,
            },
        }
    }

    pub fn is_right_handed(&self) -> bool {
        let v21x = self.0.x - self.1.x;
        let v21y = self.0.y - self.1.y;
        let v23x = self.2.x - self.1.x;
        let v23y = self.2.y - self.1.y;
        v21x * v23y - v21y * v23x > Scalar::new(0.0).unwrap()
    }

    pub fn make_right_handed(&mut self) {
        if !self.is_right_handed() {
            std::mem::swap(&mut self.1, &mut self.2);
        }
    }

    pub fn is_zero_area(&self) -> bool {
        let v21x = self.0.x - self.1.x;
        let v21y = self.0.y - self.1.y;
        let v23x = self.2.x - self.1.x;
        let v23y = self.2.y - self.1.y;

        v21x * v23y - v21y * v23x == Scalar::new(0.0).unwrap()
    }
}

pub struct ConvexHull {
    points: Vec<Point>,
}

impl ConvexHull {
    pub fn new(a: Point, b: Point, c: Point) -> ConvexHull {
        let points = vec![a, b, c];
        ConvexHull { points }
    }

    pub fn add_point<F>(&mut self, new_point: Point, mut add_triangle: F)
    where
        F: FnMut(Triangle),
    {
        let visible = self
            .points
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, p)| {
                let t = Triangle(p, self.points[(i + 1) % self.points.len()], new_point);
                (i, t)
            })
            .filter(|(_, t)| !t.is_right_handed() && !t.is_zero_area());

        let mut start = None;
        let mut jump_start = None;
        let mut jump_end = None;
        let mut end = None;

        for (i, triangle) in visible {
            add_triangle(triangle);

            if start.is_none() {
                start = Some(i);
            }

            if let Some(v) = end {
                if v + 1 != i {
                    jump_start = Some(v + 1);
                    jump_end = Some(i);
                }
            }

            end = Some(i);
        }

        match (start, jump_start, end) {
            (Some(start), None, Some(end)) if start != end => {
                self.points.drain(start + 1..=end);
                self.points.insert(start + 1, new_point);
            }

            (Some(start), _, Some(end)) if start == end => {
                self.points.insert(start + 1, new_point);
            }

            (_, Some(jump_start), _) => {
                let jump_end = jump_end.unwrap();
                self.points.drain(jump_end + 1..);
                self.points.drain(..jump_start);
                self.points.insert(0, new_point);
            }

            _ => {}
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
pub struct Neighbours(Option<usize>, Option<usize>, Option<usize>);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
pub struct SharedEdge(usize, Option<usize>);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
pub struct MetaTriangle {
    triangle: Triangle,
    circumcircle: Circumcircle,
    neighbours: Neighbours,
}

impl MetaTriangle {
    pub fn new(triangle: Triangle) -> MetaTriangle {
        MetaTriangle {
            triangle,
            circumcircle: triangle.circumcircle(),
            neighbours: Neighbours(None, None, None),
        }
    }

    pub fn neighbour(&self, a: Point, b: Point) -> Option<usize> {
        if self.triangle.0 != a && self.triangle.0 != b {
            self.neighbours.0
        } else if self.triangle.1 != a && self.triangle.1 != b {
            self.neighbours.1
        } else {
            self.neighbours.2
        }
    }

    pub fn neighbour_mut(&mut self, a: Point, b: Point) -> &mut Option<usize> {
        if self.triangle.0 != a && self.triangle.0 != b {
            &mut self.neighbours.0
        } else if self.triangle.1 != a && self.triangle.1 != b {
            &mut self.neighbours.1
        } else {
            &mut self.neighbours.2
        }
    }

    pub fn against_edge(&self, a: Point, b: Point) -> Point {
        if self.triangle.0 != a && self.triangle.0 != b {
            self.triangle.0
        } else if self.triangle.1 != a && self.triangle.1 != b {
            self.triangle.1
        } else {
            self.triangle.2
        }
    }
}

pub fn check_and_flip(
    a_idx: usize,
    triangles: &mut Vec<MetaTriangle>,
    edge_table: &mut FnvHashMap<(Point, Point), SharedEdge>,
) {
    let a = triangles[a_idx];

    let mut check_edge = |b_idx, edge: (Point, Point)| {
        let b: MetaTriangle = triangles[b_idx];

        let opposite_a = a.against_edge(edge.0, edge.1);
        let opposite_b = b.against_edge(edge.0, edge.1);

        if a.circumcircle.center.distance_sq(&opposite_b) >= a.circumcircle.radius_sq {
            return;
        }

        let triangle = Triangle(edge.0, opposite_a, opposite_b);
        triangles[a_idx] = MetaTriangle {
            triangle,
            circumcircle: triangle.circumcircle(),
            neighbours: Neighbours(
                Some(b_idx),
                b.neighbour(edge.0, opposite_b),
                a.neighbour(edge.0, opposite_a),
            ),
        };

        let mut neighbour_edge = (edge.0, opposite_b);
        Point::sort(&mut neighbour_edge.0, &mut neighbour_edge.1);

        if let Some(neighbour) = b.neighbour(edge.0, opposite_b) {
            *triangles[neighbour].neighbour_mut(edge.0, opposite_b) = Some(a_idx);
            edge_table.insert(neighbour_edge, SharedEdge(a_idx, Some(neighbour)));
        } else {
            edge_table.insert(neighbour_edge, SharedEdge(a_idx, None));
        }

        let triangle = Triangle(edge.1, opposite_b, opposite_a);
        triangles[b_idx] = MetaTriangle {
            triangle,
            circumcircle: triangle.circumcircle(),
            neighbours: Neighbours(
                Some(a_idx),
                a.neighbour(edge.1, opposite_a),
                b.neighbour(edge.1, opposite_b),
            ),
        };

        let mut neighbour_edge = (edge.1, opposite_a);
        Point::sort(&mut neighbour_edge.0, &mut neighbour_edge.1);

        if let Some(neighbour) = a.neighbour(edge.1, opposite_a) {
            *triangles[neighbour].neighbour_mut(edge.1, opposite_a) = Some(b_idx);
            edge_table.insert(neighbour_edge, SharedEdge(b_idx, Some(neighbour)));
        } else {
            edge_table.insert(neighbour_edge, SharedEdge(b_idx, None));
        }

        check_and_flip(a_idx, triangles, edge_table);
        check_and_flip(b_idx, triangles, edge_table);
    };

    a.neighbours
        .0
        .map(|b_idx| check_edge(b_idx, (a.triangle.1, a.triangle.2)));
    a.neighbours
        .1
        .map(|b_idx| check_edge(b_idx, (a.triangle.0, a.triangle.2)));
    a.neighbours
        .2
        .map(|b_idx| check_edge(b_idx, (a.triangle.0, a.triangle.1)));
}

pub fn add_triangle(
    triangle: Triangle,
    triangles: &mut Vec<MetaTriangle>,
    edge_table: &mut FnvHashMap<(Point, Point), SharedEdge>,
) {
    let mut mt = MetaTriangle::new(triangle);

    triangles.push(mt);
    let index = triangles.len() - 1;

    let mut add_edge = |mut a, mut b| {
        Point::sort(&mut a, &mut b);

        edge_table
            .entry((a, b))
            .and_modify(|SharedEdge(old, new)| {
                *triangles[*old].neighbour_mut(a, b) = Some(index);
                *mt.neighbour_mut(a, b) = Some(*old);
                *new = Some(index)
            })
            .or_insert_with(|| SharedEdge(index, None));
    };

    add_edge(triangle.0, triangle.1);
    add_edge(triangle.1, triangle.2);
    add_edge(triangle.2, triangle.0);

    triangles[index] = mt;

    check_and_flip(index, triangles, edge_table);
}

pub fn triangulate(mut points: Vec<Point>) -> Vec<Triangle> {
    let seed = points.pop().unwrap();

    let (i, &nearest) = points
        .par_iter()
        .enumerate()
        .min_by_key(|(_, &p)| p.distance_sq(&seed))
        .unwrap();

    points.remove(i);

    let (i, &best_third) = points
        .par_iter()
        .enumerate()
        .min_by_key(|(_, &p)| Triangle(p, seed, nearest).circumradius_sq())
        .unwrap();

    points.remove(i);

    let mut triangle = Triangle(seed, nearest, best_third);
    triangle.make_right_handed();
    let circumcenter = triangle.circumcenter();

    points.par_sort_unstable_by_key(|p| p.distance_sq(&circumcenter));

    let mut triangles = vec![];
    let mut edge_table = Default::default();
    let mut hull = ConvexHull::new(triangle.0, triangle.1, triangle.2);

    add_triangle(triangle, &mut triangles, &mut edge_table);

    for point in points {
        hull.add_point(point, |triangle| {
            add_triangle(triangle, &mut triangles, &mut edge_table);
        });
    }

    triangles.iter().map(|mt| mt.triangle).collect()
}

fn main() {
    use rand::Rng;

    let mut points = vec![];
    let mut rng = rand::thread_rng();

    for _ in 0..1000 {
        let x = rng.gen_range(0.0, 50000.0);
        let y = rng.gen_range(0.0, 50000.0);
        points.push(Point::new(x, y));
    }

    let t = std::time::Instant::now();
    let tris = triangulate(points);
    eprintln!("elapsed {:?}", t.elapsed());
    println!("{}", serde_json::to_string(&tris).unwrap());
}
