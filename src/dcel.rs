use crate::{OptionIndex, Point, Triangle};

/// Doubly connected edge list (a.k.a. half-edge data structure) of triangles
#[derive(Debug, Clone)]
pub struct TrianglesDCEL {
    /// Maps edge id to start point id
    pub vertices: Vec<usize>,

    /// Maps edge id to the opposite edge id in the adjacent triangle, if it exists
    pub halfedges: Vec<OptionIndex>,

    // lazily initialized
    points_to_triangles: Option<Vec<usize>>,
}

impl TrianglesDCEL {
    /// Constructs a new DCEL with specified capacity.
    ///
    /// The DCEL will be able to hold at most `cap` triangles.
    pub fn with_capacity(cap: usize) -> TrianglesDCEL {
        TrianglesDCEL {
            vertices: Vec::with_capacity(3 * cap),
            halfedges: vec![OptionIndex::none(); 3 * cap],
            points_to_triangles: None,
        }
    }

    /// Returns the number of triangles in the triangulation
    pub fn num_triangles(&self) -> usize {
        self.vertices.len() / 3
    }

    /// Returns the iterator over all triangles in the triangulation
    pub fn triangles<'a, 'b: 'a>(
        &'a self,
        points: &'b [Point],
    ) -> impl Iterator<Item = Triangle> + 'a {
        (0..self.vertices.len())
            .step_by(3)
            .map(move |t| self.triangle(t, points))
    }

    /// Adds a new triangle from given point ids to the DCEL and returns its `id`.
    /// Triangles `id + 1` and `id + 2` will reference to the same triangle
    /// viewed from different points.
    ///
    /// Make sure points is ordered in counter-clockwise order.
    #[inline]
    pub fn add_triangle(&mut self, points: [usize; 3]) -> usize {
        let t = self.vertices.len();
        self.vertices.extend_from_slice(&points);
        t
    }

    /// Returns point ids of the given triangle.
    ///
    /// # Examples
    /// ```
    /// # use triangulation::dcel::TrianglesDCEL;
    /// let mut dcel = TrianglesDCEL::with_capacity(3);
    /// let t = dcel.add_triangle([0, 1, 2]);
    /// assert_eq!(dcel.triangle_points(t), [0, 1, 2]);
    /// assert_eq!(dcel.triangle_points(t + 1), [1, 2, 0]);
    /// assert_eq!(dcel.triangle_points(t + 2), [2, 0, 1]);
    /// ```
    #[inline]
    pub fn triangle_points(&self, t: usize) -> [usize; 3] {
        let a = t;
        let b = self.next_edge(a);
        let c = self.next_edge(b);

        [self.vertices[a], self.vertices[b], self.vertices[c]]
    }

    /// Returns the actual triangle associated with the given id.
    ///
    /// # Examples
    /// ```
    /// # use triangulation::{Point, Triangle, dcel::TrianglesDCEL};
    /// let points = &[Point::new(10.0, 10.0), Point::new(10.0, 100.0), Point::new(100.0, 10.0)];
    ///
    /// let mut dcel = TrianglesDCEL::with_capacity(3);
    /// let t = dcel.add_triangle([0, 1, 2]);
    /// assert_eq!(dcel.triangle(t, points), Triangle(points[0], points[1], points[2]));
    /// ```
    #[inline]
    pub fn triangle(&self, t: usize, points: &[Point]) -> Triangle {
        let [a, b, c] = self.triangle_points(t);
        Triangle(points[a], points[b], points[c])
    }

    /// Returns id of the first triangle edge (e.g. the value returned from
    /// [`add_triangle`](TrianglesDCEL::add_triangle)).
    ///
    /// # Examples
    /// ```
    /// # use triangulation::dcel::TrianglesDCEL;
    /// let mut dcel = TrianglesDCEL::with_capacity(3);
    /// let t = dcel.add_triangle([0, 1, 2]);
    /// assert_eq!(dcel.triangle_first_edge(t), t);
    /// assert_eq!(dcel.triangle_first_edge(t + 1), t);
    /// assert_eq!(dcel.triangle_first_edge(t + 2), t);
    /// ```
    #[inline]
    pub fn triangle_first_edge(&self, t: usize) -> usize {
        t - t % 3
    }

    /// Returns the edge next to the specified one (counter-clockwise order).
    ///
    /// # Examples
    /// ```
    /// # use triangulation::dcel::TrianglesDCEL;
    /// let mut dcel = TrianglesDCEL::with_capacity(3);
    /// assert_eq!(dcel.next_edge(0), 1);
    /// assert_eq!(dcel.next_edge(1), 2);
    /// assert_eq!(dcel.next_edge(2), 0);
    /// ```
    #[inline]
    pub fn next_edge(&self, edge: usize) -> usize {
        if edge % 3 == 2 {
            edge - 2
        } else {
            edge + 1
        }
    }

    /// Returns the edge next previous for the specified one (counter-clockwise order).
    ///
    /// # Examples
    /// ```
    /// # use triangulation::dcel::TrianglesDCEL;
    /// let mut dcel = TrianglesDCEL::with_capacity(3);
    /// assert_eq!(dcel.prev_edge(0), 2);
    /// assert_eq!(dcel.prev_edge(1), 0);
    /// assert_eq!(dcel.prev_edge(2), 1);
    /// ```
    #[inline]
    pub fn prev_edge(&self, edge: usize) -> usize {
        if edge % 3 == 0 {
            edge + 2
        } else {
            edge - 1
        }
    }

    /// Returns the twin edge id, if it exists.
    #[inline]
    pub fn twin(&self, edge: usize) -> Option<usize> {
        self.halfedges[edge].get()
    }

    /// Mark two given edges as twins.
    ///
    /// # Examples
    /// ```
    /// # use triangulation::dcel::TrianglesDCEL;
    /// let mut dcel = TrianglesDCEL::with_capacity(6);
    /// let a = dcel.add_triangle([0, 1, 2]);
    /// let b = dcel.add_triangle([2, 1, 3]);
    /// dcel.link(a + 1, b);
    /// assert_eq!(dcel.twin(a + 1), Some(b));
    /// assert_eq!(dcel.twin(b), Some(a + 1));
    /// ```
    #[inline]
    pub fn link(&mut self, a: usize, b: usize) {
        self.halfedges[a] = OptionIndex::some(b);
        self.halfedges[b] = OptionIndex::some(a);
    }

    /// Removes twin of the given edge.
    #[inline]
    pub fn unlink(&mut self, a: usize) {
        self.halfedges[a] = OptionIndex::none();
    }

    /// If `b` is `Some` works like [`link`](TrianglesDCEL::link),
    /// otherwise removes the twin of `a`.
    #[inline]
    pub fn link_option(&mut self, a: usize, b: Option<usize>) {
        if let Some(b) = b {
            self.link(a, b);
        } else {
            self.unlink(a);
        }
    }

    /// Returns the iterator of triangles around the given point.
    ///
    /// [`init_revmap`](TrianglesDCEL::init_revmap) must be called beforehand
    /// to initialize the point-to-triangle map.
    pub fn triangles_around_point<'a>(&'a self, p: usize) -> TrianglesAroundPoint<'a> {
        let start = self
            .points_to_triangles
            .as_ref()
            .expect("initialize point-to-triangle map calling init_revmap")[p];

        TrianglesAroundPoint {
            dcel: self,
            start,
            current: Some(start),
            backward: false,
        }
    }

    /// Initializes the point-to-triangle map.
    pub fn init_revmap(&mut self) {
        if self.points_to_triangles.is_some() {
            return;
        }

        let mut map = vec![0; self.vertices.len()];

        for (t, &p) in self.vertices.iter().enumerate() {
            map[p] = t;
        }

        self.points_to_triangles = Some(map);
    }
}

/// Iterator of triangles around a certain point in DCEL
#[derive(Debug, Clone)]
pub struct TrianglesAroundPoint<'a> {
    dcel: &'a TrianglesDCEL,
    start: usize,
    current: Option<usize>,
    backward: bool,
}

impl<'a> Iterator for TrianglesAroundPoint<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        let result = self.current?;

        if self.backward {
            self.current = self.dcel.twin(self.dcel.prev_edge(result));

            if self.current == Some(self.start) {
                self.current = None;
            }
        } else {
            self.current = self.dcel.twin(result).map(|t| self.dcel.next_edge(t));

            if self.current.is_none() {
                self.current = self.dcel.twin(self.dcel.prev_edge(self.start));
                self.backward = true;
            }

            if self.current == Some(self.start) {
                self.current = None;
            }
        }

        Some(result)
    }
}

impl<'a> std::iter::FusedIterator for TrianglesAroundPoint<'a> {}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::Delaunay;

    fn circular(count: usize) -> TrianglesDCEL {
        let mut points = Vec::with_capacity(count + 1);

        points.push(Point::new(100.0, 100.0));

        for i in 0..count {
            let angle = i as f32 / count as f32 * 2.0 * std::f32::consts::PI;
            let (sin, cos) = angle.sin_cos();
            points.push(Point::new(cos * 100.0 + 100.0, sin * 100.0 + 100.0));
        }

        let t = Delaunay::new(&points).unwrap();
        t.dcel
    }

    #[test]
    fn around_center() {
        let count = 10;
        let mut dcel = circular(count);
        assert_eq!(dcel.num_triangles(), count);

        dcel.init_revmap();

        let around = dcel.triangles_around_point(0).collect::<Vec<_>>();

        assert_eq!(around.len(), count);
        assert_eq!(around.iter().collect::<HashSet<_>>().len(), count); // no duplicates

        for &p in &around {
            assert_eq!(dcel.vertices[p], 0);
        }
    }

    #[test]
    fn around_hull_vertex() {
        let count = 10;
        let mut dcel = circular(count);
        assert_eq!(dcel.num_triangles(), count);

        dcel.init_revmap();

        let around = dcel.triangles_around_point(1).collect::<Vec<_>>();

        assert_eq!(around.len(), 2);
        assert_eq!(around.iter().collect::<HashSet<_>>().len(), 2); // no duplicates

        for &p in &around {
            assert_eq!(dcel.vertices[p], 1);
        }
    }
}
