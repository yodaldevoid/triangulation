use core::ops::{Add, Index, IndexMut, Sub};

use crate::{OptionIndex, Point, Triangle};

/// Doubly connected edge list (a.k.a. half-edge data structure) of triangles
#[derive(Debug, Clone)]
pub struct TrianglesDCEL {
    /// Maps edge id to start point id
    pub vertices: Vec<PointIndex>,

    /// Maps edge id to the opposite edge id in the adjacent triangle, if it exists
    pub halfedges: Vec<OptionIndex<EdgeIndex>>,

    // lazily initialized
    points_to_triangles: Option<Vec<EdgeIndex>>,
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
            .map(move |t| self.triangle(t.into(), points))
    }

    /// Adds a new triangle from given point ids to the DCEL and returns its `id`.
    /// Triangles `id + 1` and `id + 2` will reference to the same triangle
    /// viewed from different points.
    ///
    /// Make sure points is ordered in counter-clockwise order.
    #[inline]
    pub fn add_triangle(&mut self, points: [PointIndex; 3]) -> EdgeIndex {
        let t = self.vertices.len();
        self.vertices.extend_from_slice(&points);
        t.into()
    }

    #[inline]
    pub fn triangle_edges(&self, t: EdgeIndex) -> [EdgeIndex; 3] {
        let a = t;
        let b = self.next_edge(a);
        let c = self.next_edge(b);
        [a, b, c]
    }

    /// Returns point ids of the given triangle.
    ///
    /// # Examples
    /// ```
    /// # use triangulation::dcel::TrianglesDCEL;
    /// let mut dcel = TrianglesDCEL::with_capacity(3);
    /// let t = dcel.add_triangle([0.into(), 1.into(), 2.into()]);
    /// assert_eq!(dcel.triangle_points(t), [0.into(), 1.into(), 2.into()]);
    /// assert_eq!(dcel.triangle_points(t + 1), [1.into(), 2.into(), 0.into()]);
    /// assert_eq!(dcel.triangle_points(t + 2), [2.into(), 0.into(), 1.into()]);
    /// ```
    #[inline]
    pub fn triangle_points(&self, t: EdgeIndex) -> [PointIndex; 3] {
        let [a, b, c] = self.triangle_edges(t);

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
    /// let t = dcel.add_triangle([0.into(), 1.into(), 2.into()]);
    /// assert_eq!(dcel.triangle(t, points), Triangle(points[0], points[1], points[2]));
    /// ```
    #[inline]
    pub fn triangle(&self, t: EdgeIndex, points: &[Point]) -> Triangle {
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
    /// let t = dcel.add_triangle([0.into(), 1.into(), 2.into()]).into();
    /// assert_eq!(dcel.triangle_first_edge(t), t);
    /// assert_eq!(dcel.triangle_first_edge(t + 1), t);
    /// assert_eq!(dcel.triangle_first_edge(t + 2), t);
    /// ```
    #[inline]
    pub fn triangle_first_edge(&self, t: EdgeIndex) -> EdgeIndex {
        (t.0 - t.0 % 3).into()
    }

    /// Returns the edge next to the specified one (counter-clockwise order).
    ///
    /// # Examples
    /// ```
    /// # use triangulation::dcel::TrianglesDCEL;
    /// let mut dcel = TrianglesDCEL::with_capacity(3);
    /// assert_eq!(dcel.next_edge(0.into()), 1.into());
    /// assert_eq!(dcel.next_edge(1.into()), 2.into());
    /// assert_eq!(dcel.next_edge(2.into()), 0.into());
    /// ```
    #[inline]
    pub fn next_edge(&self, edge: EdgeIndex) -> EdgeIndex {
        if edge.0 % 3 == 2 {
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
    /// assert_eq!(dcel.prev_edge(0.into()), 2.into());
    /// assert_eq!(dcel.prev_edge(1.into()), 0.into());
    /// assert_eq!(dcel.prev_edge(2.into()), 1.into());
    /// ```
    #[inline]
    pub fn prev_edge(&self, edge: EdgeIndex) -> EdgeIndex {
        if edge.0 % 3 == 0 {
            edge + 2
        } else {
            edge - 1
        }
    }

    /// Returns the twin edge id, if it exists.
    #[inline]
    pub fn twin(&self, edge: EdgeIndex) -> Option<EdgeIndex> {
        self.halfedges[edge].get()
    }

    #[inline]
    pub fn edge_endpoint(&self, edge: EdgeIndex) -> PointIndex {
        self.vertices[self.next_edge(edge)]
    }

    /// Mark two given edges as twins.
    ///
    /// # Examples
    /// ```
    /// # use triangulation::dcel::TrianglesDCEL;
    /// let mut dcel = TrianglesDCEL::with_capacity(6);
    /// let a = dcel.add_triangle([0.into(), 1.into(), 2.into()]);
    /// let b = dcel.add_triangle([2.into(), 1.into(), 3.into()]).into();
    /// dcel.link(a + 1, b);
    /// assert_eq!(dcel.twin(a + 1), Some(b));
    /// assert_eq!(dcel.twin(b), Some(a + 1));
    /// ```
    #[inline]
    pub fn link(&mut self, a: EdgeIndex, b: EdgeIndex) {
        self.halfedges[a] = OptionIndex::some(b);
        self.halfedges[b] = OptionIndex::some(a);
    }

    /// Removes twin of the given edge.
    #[inline]
    pub fn unlink(&mut self, a: EdgeIndex) {
        self.halfedges[a] = OptionIndex::none();
    }

    /// If `b` is `Some` works like [`link`](TrianglesDCEL::link),
    /// otherwise removes the twin of `a`.
    #[inline]
    pub fn link_option(&mut self, a: EdgeIndex, b: Option<EdgeIndex>) {
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
    pub fn triangles_around_point<'a>(&'a self, p: PointIndex) -> TrianglesAroundPoint<'a> {
        let start = self
            .points_to_triangles
            .as_ref()
            .expect("initialize point-to-triangle map calling init_revmap")[p.0];

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

        let mut map = vec![0.into(); self.vertices.len()];

        for (t, &p) in self.vertices.iter().enumerate() {
            map[p.0] = t.into();
        }

        self.points_to_triangles = Some(map);
    }
}

/// Iterator of triangles around a certain point in DCEL
#[derive(Debug, Clone)]
pub struct TrianglesAroundPoint<'a> {
    dcel: &'a TrianglesDCEL,
    start: EdgeIndex,
    current: Option<EdgeIndex>,
    backward: bool,
}

impl<'a> Iterator for TrianglesAroundPoint<'a> {
    type Item = EdgeIndex;

    fn next(&mut self) -> Option<EdgeIndex> {
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

        let around = dcel.triangles_around_point(0.into()).collect::<Vec<_>>();

        assert_eq!(around.len(), count);
        assert_eq!(around.iter().collect::<HashSet<_>>().len(), count); // no duplicates

        for &p in &around {
            assert_eq!(dcel.vertices[p], 0.into());
        }
    }

    #[test]
    fn around_hull_vertex() {
        let count = 10;
        let mut dcel = circular(count);
        assert_eq!(dcel.num_triangles(), count);

        dcel.init_revmap();

        let around = dcel.triangles_around_point(PointIndex(1)).collect::<Vec<_>>();

        assert_eq!(around.len(), 2);
        assert_eq!(around.iter().collect::<HashSet<_>>().len(), 2); // no duplicates

        for &p in &around {
            assert_eq!(dcel.vertices[p], 1.into());
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Hash)]
pub struct EdgeIndex(usize);

impl EdgeIndex {
    pub fn as_usize(&self) -> usize {
        self.0
    }
}

impl From<usize> for EdgeIndex {
    fn from(idx: usize) -> Self {
        EdgeIndex(idx)
    }
}

impl From<EdgeIndex> for usize  {
    fn from(idx: EdgeIndex) -> Self {
        idx.0
    }
}

// For indexing into vertex records
impl Index<EdgeIndex> for [PointIndex] {
    type Output = PointIndex;

    fn index(&self, idx: EdgeIndex) -> &Self::Output {
        self.get(idx.0).unwrap()
    }
}

impl IndexMut<EdgeIndex> for [PointIndex] {
    fn index_mut(&mut self, idx: EdgeIndex) -> &mut Self::Output {
        self.get_mut(idx.0).unwrap()
    }
}

impl Index<EdgeIndex> for Vec<PointIndex> {
    type Output = PointIndex;

    fn index(&self, idx: EdgeIndex) -> &Self::Output {
        self.get(idx.0).unwrap()
    }
}

impl IndexMut<EdgeIndex> for Vec<PointIndex> {
    fn index_mut(&mut self, idx: EdgeIndex) -> &mut Self::Output {
        self.get_mut(idx.0).unwrap()
    }
}

// For indexing into halfedge records
impl Index<EdgeIndex> for [OptionIndex<EdgeIndex>] {
    type Output = OptionIndex<EdgeIndex>;

    fn index(&self, idx: EdgeIndex) -> &Self::Output {
        self.get(idx.0).unwrap()
    }
}

impl IndexMut<EdgeIndex> for [OptionIndex<EdgeIndex>] {
    fn index_mut(&mut self, idx: EdgeIndex) -> &mut Self::Output {
        self.get_mut(idx.0).unwrap()
    }
}

impl Index<EdgeIndex> for Vec<OptionIndex<EdgeIndex>> {
    type Output = OptionIndex<EdgeIndex>;

    fn index(&self, idx: EdgeIndex) -> &Self::Output {
        self.get(idx.0).unwrap()
    }
}

impl IndexMut<EdgeIndex> for Vec<OptionIndex<EdgeIndex>> {
    fn index_mut(&mut self, idx: EdgeIndex) -> &mut Self::Output {
        self.get_mut(idx.0).unwrap()
    }
}

impl Add<usize> for EdgeIndex {
    type Output = EdgeIndex;

    fn add(self, rhs: usize) -> Self::Output {
        EdgeIndex(self.0 + rhs)
    }
}

impl Sub<usize> for EdgeIndex {
    type Output = EdgeIndex;

    fn sub(self, rhs: usize) -> Self::Output {
        EdgeIndex(self.0 - rhs)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Hash)]
pub struct PointIndex(usize);

impl PointIndex {
    pub fn as_usize(&self) -> usize {
        self.0
    }
}

impl From<usize> for PointIndex {
    fn from(idx: usize) -> Self {
        PointIndex(idx)
    }
}

impl From<PointIndex> for usize  {
    fn from(idx: PointIndex) -> Self {
        idx.0
    }
}

impl Index<PointIndex> for [Point] {
    type Output = Point;

    fn index(&self, idx: PointIndex) -> &Self::Output {
        self.get(idx.0).unwrap()
    }
}

impl IndexMut<PointIndex> for [Point] {
    fn index_mut(&mut self, idx: PointIndex) -> &mut Self::Output {
        self.get_mut(idx.0).unwrap()
    }
}

impl Index<PointIndex> for Vec<Point> {
    type Output = Point;

    fn index(&self, idx: PointIndex) -> &Self::Output {
        self.get(idx.0).unwrap()
    }
}

impl IndexMut<PointIndex> for Vec<Point> {
    fn index_mut(&mut self, idx: PointIndex) -> &mut Self::Output {
        self.get_mut(idx.0).unwrap()
    }
}

impl Add<usize> for PointIndex {
    type Output = PointIndex;

    fn add(self, rhs: usize) -> Self::Output {
        PointIndex(self.0 + rhs)
    }
}

impl Sub<usize> for PointIndex {
    type Output = PointIndex;

    fn sub(self, rhs: usize) -> Self::Output {
        PointIndex(self.0 - rhs)
    }
}
