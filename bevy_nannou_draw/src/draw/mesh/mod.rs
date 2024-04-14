//! Items related to the custom mesh type used by the `Draw` API.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use std::ops::{Deref, DerefMut};
use bevy::render::render_asset::RenderAssetUsages;

pub mod builder;

pub use self::builder::MeshBuilder;

pub trait MeshExt {
    fn init() -> Mesh;
    fn init_with_topology(topology: PrimitiveTopology) -> Mesh;
    fn clear(&mut self);
    fn points(&self) -> &[[f32; 3]];
    fn points_mut(&mut self) -> &mut Vec<[f32; 3]>;
    fn colors(&self) -> &[[f32; 4]];
    fn colors_mut(&mut self) -> &mut Vec<[f32; 4]>;
    fn tex_coords(&self) -> &[[f32; 2]];
    fn tex_coords_mut(&mut self) -> &mut Vec<[f32; 2]>;
    fn normals(&self) -> &[[f32; 3]];
    fn normals_mut(&mut self) -> &mut Vec<[f32; 3]>;
    fn get_index(&self, index: usize) -> u32;
    fn count_indices(&self) -> usize;
    fn push_index(&mut self, index: u32);
}

impl MeshExt for Mesh {
    fn init() -> Mesh {
        Self::init_with_topology(PrimitiveTopology::TriangleList)
    }

    fn init_with_topology(topology: PrimitiveTopology) -> Mesh {
        let mesh = Mesh::new(topology, RenderAssetUsages::default());
        mesh.with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<[f32; 3]>::new())
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, Vec::<[f32; 4]>::new())
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, Vec::<[f32; 2]>::new())
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, Vec::<[f32; 3]>::new())
            .with_inserted_indices(Indices::U32(vec![]))
    }

    fn clear(&mut self) {
        for (_, attribute) in self.attributes_mut().into_iter() {
            match attribute {
                VertexAttributeValues::Float32x2(attr) => attr.clear(),
                VertexAttributeValues::Float32x3(attr) => attr.clear(),
                VertexAttributeValues::Float32x4(attr) => attr.clear(),
                _ => panic!("Unsupported attribute type"),
            }
        }

        self.indices_mut().map(|indices| {
            match indices {
                Indices::U32(indices) => indices.clear(),
                _ => panic!("Unsupported indices type"),
            }
        });
    }

    fn points(&self) -> &[[f32; 3]] {
        let points = self
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("Mesh must have ATTRIBUTE_POSITION attribute");

        match points {
            VertexAttributeValues::Float32x3(points) => points,
            _ => panic!("Mesh ATTRIBUTE_POSITION attribute must be of type Float32x3"),
        }
    }

    fn points_mut(&mut self) -> &mut Vec<[f32; 3]> {
        let points = self
            .attribute_mut(Mesh::ATTRIBUTE_POSITION)
            .expect("Mesh must have ATTRIBUTE_POSITION attribute");

        match points {
            VertexAttributeValues::Float32x3(points) => points,
            _ => panic!("Mesh ATTRIBUTE_POSITION attribute must be of type Float32x3"),
        }
    }

    fn colors(&self) -> &[[f32; 4]] {
        let colors = self
            .attribute(Mesh::ATTRIBUTE_COLOR)
            .expect("Mesh must have ATTRIBUTE_COLOR attribute");

        match colors {
            VertexAttributeValues::Float32x4(colors) => colors,
            _ => panic!("Mesh ATTRIBUTE_COLOR attribute must be of type Float32x4"),
        }
    }

    fn colors_mut(&mut self) -> &mut Vec<[f32; 4]> {
        let colors = self
            .attribute_mut(Mesh::ATTRIBUTE_COLOR)
            .expect("Mesh must have ATTRIBUTE_COLOR attribute");

        match colors {
            VertexAttributeValues::Float32x4(colors) => colors,
            _ => panic!("Mesh ATTRIBUTE_COLOR attribute must be of type Float32x4"),
        }
    }

    fn tex_coords(&self) -> &[[f32; 2]] {
        let tex_coords = self
            .attribute(Mesh::ATTRIBUTE_UV_0)
            .expect("Mesh must have ATTRIBUTE_UV_0 attribute");

        match tex_coords {
            VertexAttributeValues::Float32x2(tex_coords) => tex_coords,
            _ => panic!("Mesh ATTRIBUTE_UV_0 attribute must be of type Float32x2"),
        }
    }

    fn tex_coords_mut(&mut self) -> &mut Vec<[f32; 2]> {
        let tex_coords = self
            .attribute_mut(Mesh::ATTRIBUTE_UV_0)
            .expect("Mesh must have ATTRIBUTE_UV_0 attribute");

        match tex_coords {
            VertexAttributeValues::Float32x2(tex_coords) => tex_coords,
            _ => panic!("Mesh ATTRIBUTE_UV_0 attribute must be of type Float32x2"),
        }
    }

    fn normals(&self) -> &[[f32; 3]] {
        let normals = self
            .attribute(Mesh::ATTRIBUTE_NORMAL)
            .expect("Mesh must have ATTRIBUTE_NORMAL attribute");

        match normals {
            VertexAttributeValues::Float32x3(normals) => normals,
            _ => panic!("Mesh ATTRIBUTE_NORMAL attribute must be of type Float32x3"),
        }
    }

    fn normals_mut(&mut self) -> &mut Vec<[f32; 3]> {
        let normals = self
            .attribute_mut(Mesh::ATTRIBUTE_NORMAL)
            .expect("Mesh must have ATTRIBUTE_NORMAL attribute");

        match normals {
            VertexAttributeValues::Float32x3(normals) => normals,
            _ => panic!("Mesh ATTRIBUTE_NORMAL attribute must be of type Float32x3"),
        }
    }

    fn get_index(&self, index: usize) -> u32 {
        match self.indices() {
            Some(Indices::U32(indices)) => indices[index],
            _ => panic!("Mesh must have U32 indices"),
        }
    }

    fn count_indices(&self) -> usize {
        match self.indices() {
            Some(Indices::U32(indices)) => indices.len(),
            _ => panic!("Mesh must have U32 indices"),
        }
    }

    fn push_index(&mut self, index: u32) {
        match self.indices_mut() {
            Some(Indices::U32(indices)) => indices.push(index),
            _ => panic!("Mesh must have U32 indices"),
        }
    }
}
