use std::fs;

use crate::la::Vec3f;

#[derive(Clone, Debug)]
pub struct Wavefront {
    pub vertices: Vec<Vec3f>,
    pub texture_coord: Vec<[f32; 2]>,
    pub normals: Vec<Vec3f>,
    pub faces: Vec<([i32; 3], [i32; 3])>,
}

impl Wavefront {
    pub fn new(vertices: Vec<Vec3f>, faces: Vec<([i32; 3], [i32; 3])>, normals: Vec<Vec3f>, texture_coord: Vec<[f32; 2]>) -> Self {
        Wavefront { vertices, texture_coord, normals, faces, }
    }

    pub fn parse(file: String) -> Self {
        let contents = fs::read_to_string(file).expect("Something went wrong reading the file");
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
                    ]
                ))
            }
        }

        return Wavefront::new(vertices, faces, normals, tc);
    }
}
