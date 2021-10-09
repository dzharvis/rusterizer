use std::fs;

use crate::{
    la::Vec3f,
    tga::{Color, Image},
};

pub struct Model {
    pub model: Wavefront,
    pub normal_map: Image,
    pub texture: Image,
}

impl Model {
    pub fn new(wf: Wavefront, nm: Image, txt: Image) -> Self {
        Model {
            model: wf,
            normal_map: nm,
            texture: txt,
        }
    }

    pub fn num_faces(&self) -> usize {
        self.model.faces.len()
    }

    pub fn vertex(&self, iface: usize, nvert: usize) -> Vec3f {
        let (vertices, _) = self.model.faces.get(iface).unwrap();
        self.model.vertices[vertices[nvert] as usize]
    }

    pub fn texture_coords(&self, iface: usize, nvert: usize) -> [f32; 2] {
        let (_, texture) = self.model.faces.get(iface).unwrap();
        self.model.texture_coord[texture[nvert] as usize]
    }

    pub fn texture(&self, u: f32, v: f32) -> Color {
        self.texture.pixel_at(
            (u * self.texture.width as f32).round() as i32,
            (v * self.texture.height as f32).round() as i32,
        )
    }

    pub fn normal(&self, u: f32, v: f32) -> Vec3f {
        let normal = self.normal_map.pixel_at(
            (u * self.normal_map.width as f32).round() as i32,
            (v * self.normal_map.height as f32).round() as i32,
        );
        Vec3f(
            ((normal.2 as f32 / 255.0) * 2.) - 1.,
            ((normal.1 as f32 / 255.0) * 2.) - 1.,
            ((normal.0 as f32 / 255.0) * 2.) - 1.,
        )
        .normalize()
    }
}

#[derive(Clone, Debug)]
pub struct Wavefront {
    pub vertices: Vec<Vec3f>,
    pub texture_coord: Vec<[f32; 2]>,
    pub normals: Vec<Vec3f>,
    pub faces: Vec<([i32; 3], [i32; 3])>,
}

impl Wavefront {
    pub fn new(
        vertices: Vec<Vec3f>,
        faces: Vec<([i32; 3], [i32; 3])>,
        normals: Vec<Vec3f>,
        texture_coord: Vec<[f32; 2]>,
    ) -> Self {
        Wavefront {
            vertices,
            texture_coord,
            normals,
            faces,
        }
    }

    pub fn parse_file(file: String) -> Self {
        let contents = fs::read_to_string(file).expect("Something went wrong reading the file");
        Wavefront::parse_string(contents)
    }

    pub fn parse_string(contents: String) -> Self {
        // let contents = fs::read_to_string(file).expect("Something went wrong reading the file");
        let lines = contents.lines();
        let mut vertices: Vec<Vec3f> = Vec::new();
        let mut normals: Vec<Vec3f> = Vec::new();
        let mut tc: Vec<[f32; 2]> = Vec::new();
        let mut faces: Vec<([i32; 3], [i32; 3])> = Vec::new();
        for l in lines {
            let lc = l.trim();
            if lc.starts_with("#") || l.is_empty() {
                continue;
            }
            if lc.starts_with("v ") {
                let mut items = lc.split_ascii_whitespace();
                items.next().unwrap(); // v
                vertices.push(Vec3f(
                    items.next().unwrap().parse().unwrap(),
                    items.next().unwrap().parse().unwrap(),
                    items.next().unwrap().parse().unwrap(),
                ))
            }
            if lc.starts_with("vn ") {
                let mut items = lc.split_ascii_whitespace();
                items.next().unwrap(); // vn
                normals.push(Vec3f(
                    items.next().unwrap().parse().unwrap(),
                    items.next().unwrap().parse().unwrap(),
                    items.next().unwrap().parse().unwrap(),
                ))
            }
            if lc.starts_with("vt ") {
                let mut items = lc.split_ascii_whitespace();
                items.next().unwrap(); // vt
                tc.push([
                    items.next().unwrap().parse().unwrap(),
                    items.next().unwrap().parse().unwrap(),
                ])
            }
            if lc.starts_with("f ") {
                let mut items = lc.split_ascii_whitespace();
                items.next().unwrap(); // f
                let mut f1 = items.next().unwrap().split("/");
                let mut f2 = items.next().unwrap().split("/");
                let mut f3 = items.next().unwrap().split("/");

                faces.push((
                    [
                        f1.next().unwrap().parse::<i32>().unwrap() - 1,
                        f2.next().unwrap().parse::<i32>().unwrap() - 1,
                        f3.next().unwrap().parse::<i32>().unwrap() - 1,
                    ],
                    [
                        f1.next().unwrap().parse::<i32>().unwrap() - 1,
                        f2.next().unwrap().parse::<i32>().unwrap() - 1,
                        f3.next().unwrap().parse::<i32>().unwrap() - 1,
                    ],
                ))
            }
        }

        return Wavefront::new(vertices, faces, normals, tc);
    }
}
