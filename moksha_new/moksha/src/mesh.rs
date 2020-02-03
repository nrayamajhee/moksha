use crate::storage::Component;
use genmesh::{
    generators::{IndexedPolygon, SharedVertex},
    EmitTriangles, Triangulate, Vertex,
};
use nalgebra::{Isometry3, Vector3};

#[derive(Debug)]
pub struct Color {
    r: u16,
    g: u16,
    b: u16,
    a: f32,
}

impl Color {
    pub fn white() -> Self {
        Self::gray(1.0)
    }
    pub fn gray(scale: f64) -> Self {
        assert!(scale <= 1.0 && scale >= 0.0, "Scale should be in between 0.0 and 1.0");
        let val = (u16::max_value() as f64 * scale) as u16;
        Self {
            r: val,
            g: val,
            b: val,
            a: 1.0,
        }
    }
}

#[derive(Debug)]
pub struct Geometry {
    pub vertices: Vec<f32>,
    pub normals: Vec<f32>,
    pub indices: Vec<usize>,
}

#[derive(Debug)]
pub struct Material {
    pub color: Option<Color>,
}

#[derive(Debug)]
pub struct Mesh {
    pub geometry: Geometry,
    pub material: Material,
}

impl Component for Mesh {}

impl Geometry {
    pub fn from_genmesh<T, P>(primitive: &T) -> Self
    where
        P: EmitTriangles<Vertex = usize>,
        T: SharedVertex<Vertex> + IndexedPolygon<P>,
    {
        Self::generate(primitive, true)
    }
    pub fn from_genmesh_no_normals<T, P>(primitive: &T) -> Self
    where
        P: EmitTriangles<Vertex = usize>,
        T: SharedVertex<Vertex> + IndexedPolygon<P>,
    {
        Self::generate(primitive, false)
    }
    fn generate<T, P>(primitive: &T, add_normals: bool) -> Self
    where
        P: EmitTriangles<Vertex = usize>,
        T: SharedVertex<Vertex> + IndexedPolygon<P>,
    {
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut indices = Vec::new();
        for Vertex { pos, normal } in primitive.shared_vertex_iter() {
            vertices.push(pos.x);
            vertices.push(pos.y);
            vertices.push(pos.z);
            if add_normals {
                normals.push(normal.x);
                normals.push(normal.y);
                normals.push(normal.z);
            }
        }
        for t in primitive.indexed_polygon_iter().triangulate() {
            indices.push(t.x);
            indices.push(t.y);
            indices.push(t.z);
        }
        Self {
            vertices,
            indices,
            normals,
        }
    }
}

#[derive(Debug)]
pub struct Transform {
    pub isometry: Isometry3<f32>,
    pub scale: Vector3<f32>,
}

impl Component for Transform {}

impl Transform {
    pub fn identity() -> Self {
        Self::from(Isometry3::identity())
    }
}

impl From<Isometry3<f32>> for Transform {
    fn from(isometry: Isometry3<f32>) -> Self {
        Self {
            isometry,
            scale: Vector3::new(1., 1., 1.),
        }
    }
}
