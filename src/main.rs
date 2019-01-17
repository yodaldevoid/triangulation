use ordered_float::NotNan;
use rayon::prelude::*;

use serde_derive::Serialize;

type Scalar = NotNan<f32>;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Point {
    x: Scalar,
    y: Scalar,
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
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
pub struct Triangle(Point, Point, Point);

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
            y: y + self.1.y,
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
}

fn add_points(points: &[Point], triangles: &mut Vec<Triangle>) {
    let mut hull = vec![triangles[0].0, triangles[0].1, triangles[0].2];

    for &new_point in points {
        let visible = hull
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, point)| (i, Triangle(point, hull[(i + 1) % hull.len()], new_point)))
            .filter(|(_, triangle)| !triangle.is_right_handed())
            .map(|(i, triangle)| {
                triangles.push(triangle);
                i
            })
            .collect::<Vec<_>>();

        if visible.len() == 0 {
            // that's bad
            // running away from problems
            continue;
        }

        let initial_len = hull.len();
        let mut new_point_idx = visible[0] + 1;

        for (i, &edge) in visible.iter().enumerate().rev() {
            let prev_idx = if i == 0 {
                visible.len() - 1
            } else {
                i - 1
            };

            let prev = visible[prev_idx];

            if (prev + 1) % initial_len == edge {
                hull.remove(edge);
                new_point_idx = edge;
            }
        }

        hull.insert(new_point_idx, new_point);
    }
}

pub fn triangulate(mut points: Vec<Point>) -> Vec<Triangle> {
    let seed = points.pop().unwrap();

    points.par_sort_unstable_by_key(|p| -p.distance_sq(&seed));

    let nearest = points.pop().unwrap();

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

    let mut triangles = vec![triangle];

    add_points(points.as_slice(), &mut triangles);

    triangles
}

fn main() {
    use rand::Rng;

    let mut points = vec![];
    let mut rng = rand::thread_rng();

    for _ in 0..10000 {
        let x = rng.gen_range(0.0, 500.0);
        let y = rng.gen_range(0.0, 500.0);
        points.push(Point::new(x, y));
    }

    let t = std::time::Instant::now();
    let tris = triangulate(points);
    eprintln!("elapsed {:?}", t.elapsed());
    println!("{}", serde_json::to_string(&tris).unwrap());
}
