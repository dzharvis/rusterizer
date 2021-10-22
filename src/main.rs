#![feature(generic_const_exprs)]
#[cfg(not(feature = "local"))]
extern crate anyhow;
#[cfg(not(feature = "local"))]
extern crate yew;

mod la;
mod model;
mod shader;
mod tga;
#[cfg(not(feature = "local"))]
mod web;
#[cfg(feature = "local")]
use crate::{
    la::{get_look_at, look_at, Matrix, MatrixI, Vec3f},
    model::Model,
    shader::{triangle, BasicShader, Shader, ShaderConf},
    tga::Image,
};
#[cfg(not(feature = "local"))]
use web::web;

#[cfg(not(feature = "local"))]
fn main() {
    web();
}

#[cfg(feature = "local")]
fn main() {
    use model::{Model, Wavefront};
    use shader::LightShader;

    let width: i32 = 1000;
    let height: i32 = 1000;
    let mut out_texture = tga::Image::new(width, height);
    let mut z_buffer = tga::Image::new(width, height);
    let mut light_texture = tga::Image::new(width, height);

    let wavefront = Wavefront::parse_file("./res/african_head/model.obj".to_string());
    let model_texture = tga::Image::from_file("./res/african_head/texture.tga".to_string());
    let model_normals = tga::Image::from_file("./res/african_head/normals.tga".to_string());

    let model = Model::new(wavefront, model_normals, model_texture);

    let camvec = Vec3f(0.5, 0.5, 1.0);
    let cam_lookat = Vec3f(0.0, 0.0, 0.0);
    let lookat_m = get_look_at(&camvec, &cam_lookat);
    let lookat_mi = lookat_m.inverse().transpose();
    let light_dir: Vec3f = look_at(&lookat_m, &Vec3f(1.0, -0.0, 0.5).normalize()).normalize();

    // println!("{:?}", lookat.mul(&lookat_i));
    let mut shader = BasicShader {
        conf: ShaderConf::new(),
        light_dir,
        lookat_m,
        lookat_mi,
        model: &model,
        out_texture: &mut out_texture,
        z_buffer: &mut z_buffer,
        light_texture: &mut light_texture,
        varying_uv: Matrix::zeroed(),
        varying_xy: Matrix::zeroed(),
        vertices: [Vec3f::zeroed(); 3],
        normal_face_vec: None,
    };

    for f in 0..model.num_faces() {
        let mut vertices = [Vec3f::zeroed(), Vec3f::zeroed(), Vec3f::zeroed()];
        for v in 0..3 {
            vertices[v] = shader.vertex(f, v);
        }
        triangle(&vertices[0], &vertices[1], &vertices[2], &mut shader);
    }

    let light_model = Model::screen_texture_model(); 

    let mut occl_texture = Image::new(width, height);
    let mut light_shader = LightShader {
        conf: ShaderConf::new(),
        model: &light_model,
        out_texture: &mut out_texture,
        light_texture: &mut light_texture,
        z_buffer: &mut z_buffer,
        varying_uv: Matrix::zeroed(),
        varying_xy: Matrix::zeroed(),
        occl_texture: &mut occl_texture,
    };

    for f in 0..light_model.num_faces() {
        let mut vertices = [Vec3f::zeroed(), Vec3f::zeroed(), Vec3f::zeroed()];
        for v in 0..3 {
            vertices[v] = light_shader.vertex(f, v);
        }
        triangle(&vertices[0], &vertices[1], &vertices[2], &mut light_shader);
    }

    out_texture.apply_gamma(1.5);
    out_texture.write_to_tga("african_head.tga").unwrap();
    z_buffer.write_to_tga("zbuff.tga").unwrap();
    light_texture.write_to_tga("light.tga").unwrap();
    occl_texture.write_to_tga("occl.tga").unwrap();
}
