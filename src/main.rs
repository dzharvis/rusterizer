mod la;
mod tga;
mod wf;
use std::{mem::{self}, ops::Mul};

use la::Matrix;
use la::Vec3f;
use wf::Wavefront;

use crate::la::barycentric;

trait Shader {
    fn vertex(&mut self, face: usize, vertex: usize) -> Vec3f;
    fn fragment(&mut self, bar: &Vec3f);
}

struct BasicShader<'a> {
    light_dir: Vec3f,
    lookat_m: Matrix,
    lookat_mi: Matrix,
    model: &'a Wavefront,
    out_texture: &'a mut tga::Image,
    mod_texture: &'a tga::Image,
    norm_texture: &'a tga::Image,
    z_buffer: &'a mut tga::Image,

    varying_uv: Matrix, // 3x2
    varying_xy: Matrix, // 3x3
}

impl Shader for BasicShader<'_> {
    fn vertex(&mut self, face: usize, vertex: usize) -> Vec3f {
        let (v, t) = self.model.faces.get(face).unwrap();
        let v = self.model.vertices.get(v[vertex] as usize).unwrap();
        let t = self.model.texture_coord.get(t[vertex] as usize).unwrap();

        for i in 0..2 {
            self.varying_uv.0[i][vertex] = t[i];
        }

        let ss = to_screen_space(
            &persp(10.0, &look_at(&self.lookat_m, v)),
            self.out_texture.width,
            self.out_texture.height,
        );
        self.varying_xy.0[0][vertex] = ss.0;
        self.varying_xy.0[1][vertex] = ss.1;
        self.varying_xy.0[2][vertex] = ss.2;
        ss
    }

    fn fragment(&mut self, bar: &Vec3f) {
        if bar.0 < 0.0 || bar.1 < 0.0 || bar.2 < 0.0 {
            return;
        }
        let bar_mtrx = bar.into();
        let xyz = self.varying_xy.mul(&bar_mtrx);
        let x = xyz.0[0][0].round() as i32;
        let y = xyz.0[1][0].round() as i32;
        let z = xyz.0[2][0].round() as u8;
        if z < self.z_buffer.pixel_at(x, y).0
            || x < 0
            || x >= self.out_texture.width
            || y < 0
            || y >= self.out_texture.height
        {
            return;
        }

        let tp = self.varying_uv.mul(&bar_mtrx);
        let (u, v) = (tp.0[0][0], tp.0[1][0]);

        let txt = self.mod_texture.pixel_at(
            (u * self.mod_texture.width as f32).round() as i32,
            (v * self.mod_texture.height as f32).round() as i32,
        );
        let normal = self.norm_texture.pixel_at(
            (u * self.mod_texture.width as f32).round() as i32,
            (v * self.mod_texture.height as f32).round() as i32,
        );
        let normal_vec = Vec3f(((normal.2 as f32 / 255.0) * 2.) - 1., ((normal.1 as f32 / 255.0) * 2.) - 1., ((normal.0 as f32 / 255.0) * 2.) - 1.).normalize();
        let normal_vec: Vec3f = self.lookat_mi.mul(&(normal_vec).embed(4, 0.0)).into();
        let normal_vec = normal_vec.normalize();
        
        let light = normal_vec.mul(&self.light_dir).max(0.0);
        let reflected = normal_vec.mulf(normal_vec.mul(&self.light_dir) * 2.0).sub(&self.light_dir).normalize();
        let light_spec = reflected.2.max(0.0).powf(23.0); // cam on z

        self.out_texture.set_pixel(x, y, txt.highlight(light_spec*0.9 + light));
        self.z_buffer.set_pixel(x, y, tga::Color(z, z, z))
        // }
    }
}

fn persp(c: f32, v1: &Vec3f) -> Vec3f {
    Vec3f(v1.0/(1.0-v1.2/c), v1.1/(1.0-v1.2/c), v1.2/(1.0-v1.2/c))
}

fn get_look_at(p: &Vec3f) -> Matrix {
    let up = Vec3f(0.0, 1.0, 0.0);
    let c = Vec3f(0.0, 0.0, 0.0);

    let z = p.sub(&c).normalize();
    let x = up.cross(&z).normalize();
    let y = z.cross(&x).normalize();

    let minv = Matrix(vec![
        vec![x.0, x.1, x.2, 0.0],
        vec![y.0, y.1, y.2, 0.0],
        vec![z.0, z.1, z.2, 0.0],
        vec![0.0, 0.0, 0.0, 1.0],
    ]); //.transpose();

    let tr = Matrix(vec![
        vec![1.0, 0.0, 0.0, -c.0],
        vec![0.0, 1.0, 0.0, -c.1],
        vec![0.0, 0.0, 1.0, -c.2],
        vec![0.0, 0.0, 1.0, 1.0],
    ]);

    let mv = minv.mul(&tr); // 4x4
    return mv;
}

fn look_at(m: &Matrix, v: &Vec3f) -> Vec3f {
    m.mul(&v.embed(4, 0.0)).into()
}

fn to_screen_space(v: &Vec3f, width: i32, height: i32) -> Vec3f {
    let x0 = (v.0 + 1.) * (width - 1) as f32 / 2.;
    let y0 = (v.1 + 1.) * (height - 1) as f32 / 2.;
    Vec3f(x0, y0, ((v.2 + 1.) / 2.) * 255.0)
}

fn main() {
    let width: i32 = 1000;
    let height: i32 = 1000;
    let mut out_texture = tga::Image::new(width, height);
    let mut z_buffer = tga::Image::new(width, height);

    let model = Wavefront::parse("african_head.obj".to_string());
    let model_texture = tga::Image::from_file("textr23.tga".to_string());
    let model_normals = tga::Image::from_file("nm.tga".to_string());
    
    let campos = Vec3f(0.5, 0.5, 1.0);
    let lookat = get_look_at(&campos);
    let lookat_i = lookat.inverse().transpose();
    let light_dir: Vec3f = look_at(&lookat, &Vec3f(01.0, -0.0, 0.5).normalize());

    // println!("{:?}", lookat.mul(&lookat_i));
    let mut shader = BasicShader {
        light_dir: light_dir.normalize(),
        lookat_m: lookat,
        lookat_mi: lookat_i,
        model: &model,
        mod_texture: &model_texture,
        out_texture: &mut out_texture,
        norm_texture: &model_normals,
        z_buffer: &mut z_buffer,
        varying_uv: Matrix::zeroed(3, 2),
        varying_xy: Matrix::zeroed(3, 3),
    };

    for f in 0..model.faces.len() {
        let mut vertices = [Vec3f::zeroed(), Vec3f::zeroed(), Vec3f::zeroed()];
        for v in 0..3 {
            vertices[v] = shader.vertex(f, v);
        }
        triangle(&vertices[0], &vertices[1], &vertices[2], &mut shader);
    }

    out_texture.apply_gamma(1.5);
    out_texture.write_to_tga("african_head.tga").unwrap();
    z_buffer.write_to_tga("zbuff.tga").unwrap();
}

fn triangle(v1: &Vec3f, v2: &Vec3f, v3: &Vec3f, sh: &mut dyn Shader) {
    let z = Vec3f(v2.0, v2.1, v2.2)
        .sub(&Vec3f(v1.0, v1.1, v1.2))
        .cross(&Vec3f(v3.0, v3.1, v3.2).sub(&Vec3f(v1.0, v1.1, v1.2)));

    if z.2 < 0.0 {
        return;
    }

    let x0 = vec![v1.0, v2.0, v3.0]
        .iter()
        .fold(&v1.0, |xmin, x| if xmin > x { x } else { xmin })
        .round() as i32;
    let y0 = vec![v1.1, v2.1, v3.1]
        .iter()
        .fold(&v1.1, |ymin, y| if ymin > y { y } else { ymin })
        .round() as i32;
    let x1 = vec![v1.0, v2.0, v3.0]
        .iter()
        .fold(&v1.0, |xmax, x| if xmax < x { x } else { xmax })
        .round() as i32;
    let y1 = vec![v1.1, v2.1, v3.1]
        .iter()
        .fold(&v1.1, |ymax, y| if ymax < y { y } else { ymax })
        .round() as i32;

    for y in y0..=y1 {
        for x in x0..=x1 {
            let bc = barycentric(&v1, &v2, &v3, (x as f32, y as f32));
            sh.fragment(&bc);
        }
    }
}

fn line(
    mut x0: i32,
    mut y0: i32,
    mut x1: i32,
    mut y1: i32,
    img: &mut tga::Image,
    color: tga::Color,
) {
    let dx = if x1 > x0 { x1 - x0 } else { x0 - x1 };
    let dy = if y1 > y0 { y1 - y0 } else { y0 - y1 };

    if dx > dy {
        if x1 < x0 {
            mem::swap(&mut x1, &mut x0);
            mem::swap(&mut y1, &mut y0);
        }
        for x in x0..=x1 {
            let t = ((x - x0) as f32) / ((x1 - x0) as f32);
            let y = (y0 as f32) * (1f32 - t) + (y1 as f32) * t;
            img.set_pixel(x as i32, y.round() as i32, color.clone());
        }
    } else {
        if y1 < y0 {
            mem::swap(&mut x1, &mut x0);
            mem::swap(&mut y1, &mut y0);
        }
        for y in y0..=y1 {
            let t = ((y - y0) as f32) / ((y1 - y0) as f32);
            let x = (x0 as f32) * (1f32 - t) + (x1 as f32) * t;
            img.set_pixel(x.round() as i32, y as i32, color.clone());
        }
    }
}
