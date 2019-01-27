use wasm_bindgen::prelude::*;
use triangulation::{Point, Delaunay};

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn triangulate(p: &[f32]) -> Vec<u32> {
    let mut points = Vec::with_capacity(p.len() / 2);

    for i in (0..p.len()).step_by(2) {
        points.push(Point::new(p[i], p[i + 1]));
    }

    let t = Delaunay::new(&points).unwrap();
    t.dcel.vertices.iter().map(|&v| v as u32).collect()
}
