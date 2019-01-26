use crate::{OptionIndex, Point, Triangle};

/// Doubly connected edge list (a.k.a. half-edge data structure) of triangles
#[derive(Debug, Clone)]
pub struct TrianglesDCEL {
    vertices: Vec<usize>,
    halfedges: Vec<OptionIndex>,
}

impl TrianglesDCEL {
    /// Constructs a new DCEL with specified capacity.
    ///
    /// The DCEL will be able to hold at most `cap` edges.
    pub fn with_capacity(cap: usize) -> TrianglesDCEL {
        TrianglesDCEL {
            vertices: Vec::with_capacity(cap),
            halfedges: vec![OptionIndex::none(); cap],
        }
    }

    /// Adds a new triangle from given point ids to the DCEL and returns its `id`.
    /// Triangles `id + 1` and `id + 2` will reference to the same triangle
    /// viewed from different points.
    ///
    /// Make sure points is ordered in counter-clockwise order.
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
    pub fn prev_edge(&self, edge: usize) -> usize {
        if edge % 3 == 0 {
            edge + 2
        } else {
            edge - 1
        }
    }

    /// Returns the twin edge id, if it exists.
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
    pub fn link(&mut self, a: usize, b: usize) {
        self.halfedges[a] = OptionIndex::some(b);
        self.halfedges[b] = OptionIndex::some(a);
    }
}
