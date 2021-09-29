mod tga;
mod wf;
mod la;
use std::{convert::TryInto, mem::{self}};

use wf::Wavefront;
use la::Vec3f;
use la::Matrix;

trait Shader {
    fn vertex(face: u32, vertex: u32);
    fn fragment();
}

fn persp(c: f32, v1: &Vec3f) -> Vec3f {
    Vec3f(v1.0/(1.0-v1.2/c), v1.1/(1.0-v1.2/c), v1.2/(1.0-v1.2/c))
}

fn get_look_at() -> Matrix {
    let p = Vec3f(0.5, 0.3, 1.0);
    let up = Vec3f(0.0, 1.0, 0.0);
    let c = Vec3f(0.0, 0.0, 0.0);

    let z = p.sub(&c).normalize();
    let x = up.cross(&z).normalize();
    let y = z.cross(&x).normalize();

    let minv = Matrix(vec![
        vec![x.0, x.1, x.2],
        vec![y.0, y.1, y.2],
        vec![z.0, z.1, z.2],
    ]);//.transpose();

    // let minv = Matrix(vec![
    //     vec![1.0, 0.0, 0.0],
    //     vec![0.0, 1.0, 0.0],
    //     vec![0.0, 0.0, 1.0],
    // ]).transpose();

    let tr = Matrix(vec![
        vec![1.0, 0.0, 0.0, c.0],
        vec![0.0, 1.0, 0.0, c.1],
        vec![0.0, 0.0, 1.0, c.2],
    ]);
    
    let mv = minv.mul(&tr);
    // let into = mv.mul(&v.into()).into();
    // println!("{:?}", into);
    return mv;
}

fn look_at(m: &Matrix, v: &Vec3f) -> Vec3f {
    m.mul(&v.embed(4)).into()
}

fn find_t(a: f32, b: f32, m: f32) -> f32 {
    (m - a) / (b - a)
}

fn interpolate(a: f32, b: f32, t: f32) -> f32 {
    a * (1f32 - t) + b * t
}

fn interpolatev(a: &Vec3f, b: &Vec3f, t: f32) -> Vec3f {
    Vec3f(interpolate(a.0, b.0, t), interpolate(a.1, b.1, t), interpolate(a.2, b.2, t))
}

fn barycentric(a: &Vec3f, b: &Vec3f, c: &Vec3f, p: (f32, f32)) -> Vec3f {
    let cross = Vec3f(c.0 - a.0, b.0 - a.0, a.0 - p.0).cross(&Vec3f(c.1 - a.1, b.1 - a.1, a.1 - p.1));
    Vec3f(1.0 - (cross.1 + cross.0)/cross.2, cross.1/cross.2, cross.0/cross.2,)
}

fn main() {
    let width: i32 = 2000;
    let height: i32 = 2000;
    let mut zbuffer = vec![-1.1f32; (width * height).try_into().unwrap()];
    let mut out_texture = tga::Image::new(width, height);

    let model = Wavefront::parse("african_head.obj".to_string());
    let model_texture = tga::Image::from_file("textr23.tga".to_string());
    let model_normals = tga::Image::from_file("nm.tga".to_string());

    let to_screen_space = |(x, y): (f32, f32)| {
        let x0 = (x + 1.) * (width - 1) as f32 / 2.;
        let y0 = (y + 1.) * (height - 1) as f32 / 2.;
        (x0.round() as i32, y0.round() as i32)
    };

    let to_screen_space_f = |v: &Vec3f| {
        let x0 = (v.0 + 1.) * (width - 1) as f32 / 2.;
        let y0 = (v.1 + 1.) * (height - 1) as f32 / 2.;
        Vec3f(x0, y0, v.2)
    };

    let lookat = get_look_at();

    let mut triangle = |v1: &Vec3f,
                        v2: &Vec3f,
                        v3: &Vec3f,
                        tx1: &(f32, f32),
                        tx2: &(f32, f32),
                        tx3: &(f32, f32),
                        nv1: &Vec3f,
                        nv2: &Vec3f,
                        nv3: &Vec3f,
                        texture: &mut tga::Image| {

        let v1 = look_at(&lookat, &v1);
        let v2 = look_at(&lookat, &v2);
        let v3 = look_at(&lookat, &v3);

        let v1 = persp(10.0, &v1);
        let v2 = persp(10.0, &v2);
        let v3 = persp(10.0, &v3);

        let z = Vec3f(v2.0, v2.1, v2.2).sub(&Vec3f(v1.0, v1.1, v1.2)).cross(&Vec3f(v3.0, v3.1, v3.2).sub(&Vec3f(v1.0, v1.1, v1.2)));

        if z.2 < 0.0 {
            return;
        }

        let x0 = *vec![v1.0, v2.0, v3.0]
            .iter()
            .fold(&v1.0, |xmin, x| if xmin > x { x } else { xmin });
        let y0 = *vec![v1.1, v2.1, v3.1]
            .iter()
            .fold(&v1.1, |ymin, y| if ymin > y { y } else { ymin });
        let x1 = *vec![v1.0, v2.0, v3.0]
            .iter()
            .fold(&v1.0, |xmax, x| if xmax < x { x } else { xmax });
        let y1 = *vec![v1.1, v2.1, v3.1]
            .iter()
            .fold(&v1.1, |ymax, y| if ymax < y { y } else { ymax });

        let (xs0, ys0) = to_screen_space((x0, y0));
        let (xs1, ys1) = to_screen_space((x1, y1));
        let v1s = to_screen_space_f(&v1);
        let v2s = to_screen_space_f(&v2);
        let v3s = to_screen_space_f(&v3);

        for y in ys0..ys1 {
            for x in xs0..xs1 {
                let bc = barycentric(&v1s, &v2s, &v3s, (x as f32, y as f32));
                if bc.0 < 0.0 || bc.1 < 0.0 || bc.2 < 0.0 {
                    continue
                }
                if x < 0 || x >= width || y < 0 || y >= height {
                    continue
                }
                let zbp = bc.mul(&Vec3f(v1.2, v2.2, v3.2));
                let shift: usize = (y * width + x) as usize;
                if zbp >= *zbuffer.get(shift).unwrap() {
                    let tcx =  bc.mul(&Vec3f(tx1.0, tx2.0, tx3.0)) * (model_texture.width as f32);
                    let tcy =  bc.mul(&Vec3f(tx1.1, tx2.1, tx3.1)) * (model_texture.height as f32);
                    let txt = model_texture.pixel_at(tcx.round() as i32, tcy.round() as i32);
                    let normal = model_normals.pixel_at(tcx.round() as i32, tcy.round() as i32);

                    texture.set_pixel(x, y, txt.highlight(normal.2 as f32 / 255.0));
                    zbuffer[shift] = zbp;
                }
            } 
        }
    };

    for ((v1, v2, v3), (tc1, tc2, tc3)) in &model.faces {
        let nv1 = model.normals.get(*v1 as usize).unwrap();
        let nv2 = model.normals.get(*v2 as usize).unwrap();
        let nv3 = model.normals.get(*v3 as usize).unwrap();

        let v1 = model.vertices.get(*v1 as usize).unwrap();
        let v2 = model.vertices.get(*v2 as usize).unwrap();
        let v3 = model.vertices.get(*v3 as usize).unwrap();

        let t1 = model.texture_coord.get(*tc1 as usize).unwrap();
        let t2 = model.texture_coord.get(*tc2 as usize).unwrap();
        let t3 = model.texture_coord.get(*tc3 as usize).unwrap();


        triangle(v1, v2, v3, t1, t2, t3, nv1, nv2, nv3, &mut out_texture);
    }

    out_texture.apply_gamma(1.5);
    out_texture.write_to_tga("african_head.tga").unwrap();

    // println!("{:?}", wf);
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
        // if y1 < y0 {
        //     mem::swap(&mut y1, &mut y0);
        // }
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