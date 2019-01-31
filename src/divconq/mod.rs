use std::ops::Range;

use crate::geom::{Point, Triangle};
use crate::OptionIndex;

struct Half {
    triangles: Vec<usize>,
    halfedges: Vec<OptionIndex>,
    bottom_most: usize,
    offset: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Side {
    Left,
    Right,
}

impl Half {
    pub fn new(range: Range<usize>, side: Side, points: &[Point]) -> Half {
        let len = range.end - range.start;

        if len == 2 {
            Half::new_single_edge(range.start, side, points)
        } else if len == 3 {
            Half::new_single_tri(range.start, side, points)
        } else {
            panic!()
        }
    }

    fn new_single_edge(offset: usize, side: Side, points: &[Point]) -> Half {
        let bottom_most = (0..2)
            .min_by(|a, b| {
                let a = points[a + offset];
                let b = points[b + offset];

                if side == Side::Left {
                    b.y.partial_cmp(&a.y)
                        .unwrap()
                        .then(b.x.partial_cmp(&a.x).unwrap())
                } else {
                    b.y.partial_cmp(&a.y)
                        .unwrap()
                        .then(a.x.partial_cmp(&b.x).unwrap())
                }
            })
            .unwrap();

        Half {
            triangles: vec![1, 0],
            halfedges: vec![OptionIndex::none(); 2],
            bottom_most,
            offset,
        }
    }

    fn new_single_tri(offset: usize, side: Side, points: &[Point]) -> Half {
        let mut bottom_most = (0..3)
            .min_by(|a, b| {
                let a = points[a + offset];
                let b = points[b + offset];

                if side == Side::Left {
                    b.y.partial_cmp(&a.y)
                        .unwrap()
                        .then(a.x.partial_cmp(&b.x).unwrap())
                } else {
                    b.y.partial_cmp(&a.y)
                        .unwrap()
                        .then(b.x.partial_cmp(&a.x).unwrap())
                }
            })
            .unwrap();

        let tri = Triangle(points[offset], points[offset + 1], points[offset + 2]);

        let triangles = if tri.is_right_handed() {
            vec![0, 1, 2]
        } else {
            bottom_most = match bottom_most {
                2 => 1,
                1 => 2,
                a => a,
            };

            vec![0, 2, 1]
        };

        Half {
            triangles,
            halfedges: vec![OptionIndex::none(); 3],
            bottom_most,
            offset,
        }
    }

    fn next_edge(&self, edge: usize) -> usize {
        if edge % 3 == 2 {
            edge - 2
        } else {
            edge + 1
        }
    }

    fn prev_edge(&self, edge: usize) -> usize {
        if edge % 3 == 0 {
            edge + 2
        } else {
            edge - 1
        }
    }

    fn point(&self, edge: usize, points: &[Point]) -> Point {
        points[self.offset + self.triangles[edge]]
    }

    fn find_base_lr(&self, other: &Half, points: &[Point]) -> (usize, usize) {
        let left_is_lower =
            self.point(self.bottom_most, points).y > other.point(other.bottom_most, points).y;

        let (victim, culprit) = if left_is_lower {
            (self, other)
        } else {
            (other, self)
        };

        let start = culprit.bottom_most;
        let start_pt = culprit.point(start, points);
        let mut end = victim.bottom_most;

        loop {
            let next_in_tri = victim.next_edge(end);

            let next = victim.halfedges[next_in_tri]
                .get()
                .map(|e| victim.next_edge(e))
                .unwrap_or(next_in_tri);

            let tri = Triangle(
                victim.point(end, points),
                start_pt,
                victim.point(next, points),
            );

            if left_is_lower && tri.is_right_handed() || !left_is_lower && tri.is_left_handed() {
                break;
            }

            end = next;
        }

        if left_is_lower {
            (end, start)
        } else {
            (start, end)
        }
    }

    fn candidates(&self, side: Side, edge: usize) -> Candidates<'_> {
        Candidates {
            half: self,
            current: Some(edge),
            side,
            last: false,
        }
    }

    fn triangle_first_edge(&self, t: usize) -> usize {
        t - t % 3
    }

    fn delete_triangle(&mut self, side: Side, t: usize, base: &mut usize) -> bool {
        let t = self.triangle_first_edge(t);
        let base_t = self.triangle_first_edge(*base);

        let mut base_valid = true;

        if base_t == t {
            let next = if side == Side::Right {
                self.halfedges[*base].get().map(|v| self.next_edge(v))
            } else {
                self.halfedges[self.prev_edge(*base)].get()
            };

            if let Some(next) = next {
                *base = next;

                if self.bottom_most == base_t {
                    self.bottom_most = next;
                }
            } else {
                base_valid = false;
            }
        }

        for i in 0..3 {
            self.triangles[t + i] = 0;

            if let Some(h) = self.halfedges[t + i].get() {
                self.halfedges[t + i] = OptionIndex::none();
                self.halfedges[h] = OptionIndex::none();
            }
        }

        base_valid
    }

    fn select_candidate(
        &mut self,
        side: Side,
        mut base: usize,
        end: Point,
        points: &[Point],
    ) -> Option<usize> {
        let base_pt = self.point(base, points);

        loop {
            let mut candidates = self.candidates(side, base);
            let curr = candidates.next()?;
            let next = candidates.next();

            let mut tri = Triangle(self.point(curr, points), base_pt, end);

            if side == Side::Left && tri.is_left_handed()
                || side == Side::Right && tri.is_right_handed()
            {
                // do you love me?
                return None;
            }

            if !tri.is_right_handed() {
                std::mem::swap(&mut tri.1, &mut tri.2);
            }

            if let Some(next) = next {
                if tri.in_circumcircle(self.point(next, points)) {
                    if !self.delete_triangle(side, curr, &mut base) {
                        return Some(next);
                    }

                    continue;
                }
            }

            return Some(curr);
        }
    }

    pub fn merge(mut self, other: Half, points: &[Point]) -> Half {
        let base = self.find_base_lr(&other, points);
        self
    }
}

struct Candidates<'a> {
    half: &'a Half,
    current: Option<usize>,
    side: Side,
    last: bool,
}

impl<'a> Iterator for Candidates<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.last {
            let current = self.current;
            self.current = None;
            return current;
        }

        let current = self.current?;

        if self.side == Side::Right {
            let candidate = self.half.prev_edge(current);

            self.current = self.half.halfedges[current]
                .get()
                .map(|v| self.half.next_edge(v));

            if self.current.is_none() {
                self.current = Some(self.half.next_edge(current));
                self.last = true;
            }

            Some(candidate)
        } else {
            let candidate = self.half.next_edge(current);
            self.current = self.half.halfedges[self.half.prev_edge(current)].get();

            if self.current.is_none() {
                self.current = Some(self.half.prev_edge(current));
                self.last = true;
            }

            Some(candidate)
        }
    }
}

impl<'a> std::iter::FusedIterator for Candidates<'a> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bottom_most() {
        let points = vec![
            Point::new(60.0, 40.0),
            Point::new(80.0, 10.0),
            Point::new(100.0, 40.0),
        ];

        let l = Half::new(0..3, Side::Left, &points);
        assert!(points[0].approx_eq(l.point(l.bottom_most, &points)));

        let r = Half::new(0..3, Side::Right, &points);
        assert!(points[2].approx_eq(r.point(r.bottom_most, &points)));
    }

    #[test]
    fn bottom_most_couple() {
        let points = vec![Point::new(50.0, 50.0), Point::new(100.0, 50.0)];

        let l = Half::new(0..2, Side::Left, &points);
        assert!(points[0].approx_eq(l.point(l.bottom_most, &points)));

        let r = Half::new(0..2, Side::Right, &points);
        assert!(points[1].approx_eq(r.point(r.bottom_most, &points)));
    }

    #[test]
    fn base_lr() {
        let points = vec![
            Point::new(0.0, 100.0),
            Point::new(20.0, 50.0),
            Point::new(40.0, 80.0),
            Point::new(60.0, 40.0),
            Point::new(80.0, 10.0),
            Point::new(100.0, 40.0),
        ];

        let l = Half::new(0..3, Side::Left, &points);
        let r = Half::new(3..6, Side::Right, &points);

        let (b0, b1) = l.find_base_lr(&r, &points);
        let (p0, p1) = (l.point(b0, &points), r.point(b1, &points));

        assert!(p0.approx_eq(points[2]));
        assert!(p1.approx_eq(points[5]));
    }

    #[test]
    fn base_lr_victim() {
        let points = vec![
            Point::new(0.0, 40.0),
            Point::new(20.0, 10.0),
            Point::new(40.0, 40.0),
            Point::new(60.0, 100.0),
            Point::new(80.0, 50.0),
            Point::new(100.0, 80.0),
        ];

        let l = Half::new(0..3, Side::Left, &points);
        let r = Half::new(3..6, Side::Right, &points);

        let (b0, b1) = l.find_base_lr(&r, &points);
        let (p0, p1) = (l.point(b0, &points), r.point(b1, &points));

        assert!(p0.approx_eq(points[0]));
        assert!(p1.approx_eq(points[3]));
    }

    #[test]
    fn candidates() {
        let _points = vec![
            Point::new(0.0, 0.0),
            Point::new(60.0, 0.0),
            Point::new(30.0, 30.0),
            Point::new(60.0, 60.0),
            Point::new(60.0, 90.0),
        ];

        let s = |v| OptionIndex::some(v);
        let n = OptionIndex::none();

        let half = Half {
            triangles: vec![0, 2, 1, 1, 2, 3, 3, 2, 4, 0, 4, 2],
            halfedges: vec![s(11), s(3), n, s(1), s(6), n, s(4), s(10), n, n, s(7), s(0)],
            offset: 0,
            bottom_most: 10,
        };

        let mut candidates = half.candidates(Side::Right, 10);
        assert_eq!(candidates.next(), Some(9));
        assert_eq!(candidates.next(), Some(7));
        assert_eq!(candidates.next(), Some(6));
        assert_eq!(candidates.next(), None);

        let mut candidates = half.candidates(Side::Left, 8);
        assert_eq!(candidates.next(), Some(6));
        assert_eq!(candidates.next(), Some(11));
        assert_eq!(candidates.next(), Some(9));
        assert_eq!(candidates.next(), None)
    }

    #[test]
    fn valid_candidate() {
        let s = |v| OptionIndex::some(v);
        let n = OptionIndex::none();

        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(60.0, 0.0),
            Point::new(30.0, 30.0),
            Point::new(60.0, 60.0),
            Point::new(60.0, 90.0),
        ];

        let mut half = Half {
            triangles: vec![0, 2, 1, 1, 2, 3, 3, 2, 4, 0, 4, 2],
            halfedges: vec![s(11), s(3), n, s(1), s(6), n, s(4), s(10), n, n, s(7), s(0)],
            offset: 0,
            bottom_most: 10,
        };

        let c = half.select_candidate(Side::Right, 10, Point::new(-30.0, 90.0), &points);
        assert!(half.point(c.unwrap(), &points).approx_eq(Point::new(30.0, 30.0)));
    }
}
