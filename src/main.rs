mod tga;
mod wf;
mod la;
use std::{convert::TryInto, mem::{self, swap}};

use wf::Wavefront;
use la::Vec3f;
use la::Matrix;

fn persp(c: f32, v1: &Vec3f) -> Vec3f {
    Vec3f(v1.0/(1.0-v1.2/c), v1.1/(1.0-v1.2/c), v1.2/(1.0-v1.2/c))
}

fn get_look_at() -> Matrix {
    let p = Vec3f(-0.2, -0.2, 0.5);
    let up = Vec3f(0.0, 1.0, 0.0);
    let c = Vec3f(0.0, 0.0, 0.0);

    let z = p.sub(&c).normalize();
    let x = up.cross(&z).normalize();
    let y = z.cross(&x).normalize();

    let minv = Matrix(vec![
        vec![x.0, x.1, x.2],
        vec![y.0, y.1, y.2],
        vec![z.0, z.1, z.2],
    ]).transpose();

    // let minv = Matrix(vec![
    //     vec![1.0, 0.0, 0.0],
    //     vec![0.0, 1.0, 0.0],
    //     vec![0.0, 0.0, 1.0],
    // ]).transpose();

    let tr = Matrix(vec![
        vec![1.0, 0.0, 0.0, 0.0],
        vec![0.0, 1.0, 0.0, 0.0],
        vec![0.0, 0.0, 1.0, -0.5],
    ]);
    
    let mv = minv.mul(&tr);
    // let into = mv.mul(&v.into()).into();
    // println!("{:?}", into);
    return mv;
}

fn look_at(m: &Matrix, v: &Vec3f) -> Vec3f {
    let vm = Matrix(vec![
        vec![v.0],
        vec![v.1],
        vec![v.2],
        vec![1.0]
    ]);

    m.mul(&vm).into()
}

fn find_t(a: f32, b: f32, m: f32) -> f32 {
    (m - a) / (b - a)
}

fn interpolate(a: f32, b: f32, t: f32) -> f32 {
    a * (1f32 - t) + b * t
}

fn main() {

    let width: i32 = 2000;
    let height: i32 = 2000;
    let mut zbuffer = vec![-1.0f32; (width * height).try_into().unwrap()];
    let mut img = tga::Image::new(width, height);

    let wf = Wavefront::parse("african_head.obj".to_string());
    let texture = tga::Image::from_file("textr23.tga".to_string());
    let nm = tga::Image::from_file("nm.tga".to_string());
    texture.write_to_tga("sc.tga").unwrap();

    let to_screen_space = |(x, y): (f32, f32)| {
        let x0 = (x + 1.) * (width - 1) as f32 / 2.;
        let y0 = (y + 1.) * (height - 1) as f32 / 2.;
        (x0.round() as i32, y0.round() as i32)
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
                        mut img: &mut tga::Image| {

        let v1 = look_at(&lookat, &v1);
        let v2 = look_at(&lookat, &v2);
        let v3 = look_at(&lookat, &v3);

        let v1 = persp(10.0, &v1);
        let v2 = persp(10.0, &v2);
        let v3 = persp(10.0, &v3);

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

        let (mid_vertex, mid_tx, mid_nv) = vec![(&v1, tx1, nv1), (&v2, tx2, nv2), (&v3, tx3, nv3)]
            .iter()
            .find(|(Vec3f(_, y, _), _, _)| *y > y0 && *y < y1)
            .unwrap_or(&(&v1, tx1, nv1))
            .clone();
        let (high_vertex, high_tx, high_nv) = vec![(&v1, tx1, nv1), (&v2, tx2, nv2), (&v3, tx3, nv3)]
            .iter()
            .find(|(Vec3f(x, y, _), _, _)| *y < mid_vertex.1)
            .unwrap_or(&(&v2, tx2, nv2))
            .clone();
        let (low_vertex, low_tx, low_nv) = vec![(&v1, tx1, nv1), (&v2, tx2, nv2), (&v3, tx3, nv3)]
            .iter()
            .find(|(Vec3f(x, y, _), _, _)| *y > mid_vertex.1)
            .unwrap_or(&(&v3, tx3, nv3))
            .clone();

        let (xs0, ys0) = to_screen_space((x0, y0));
        let (xs1, ys1) = to_screen_space((x1, y1));
        let (mxs1, mys1) = to_screen_space((mid_vertex.0, mid_vertex.1));
        let (hxs1, hys1) = to_screen_space((high_vertex.0, high_vertex.1));
        let (lxs1, lys1) = to_screen_space((low_vertex.0, low_vertex.1));

        let z = Vec3f(v2.0, v2.1, v2.2).sub(&Vec3f(v1.0, v1.1, v1.2)).cross(&Vec3f(v3.0, v3.1, v3.2).sub(&Vec3f(v1.0, v1.1, v1.2)));
        // let get_angle = get_angle(&v1, &v2, &v3);
        // if z.2 <= 0.0 {
        //     return;
        // }
        // let color = (get_angle * 255.0).round() as u8;

        for y in mys1..ys1 {
            let t1 = find_t(mys1 as f32, lys1 as f32, y as f32);
            let t2 = find_t(hys1 as f32, lys1 as f32, y as f32);

            let xl1 = interpolate(mxs1 as f32, lxs1 as f32, t1);
            let xl2 = interpolate(hxs1 as f32, lxs1 as f32, t2);

            let zl1 = interpolate(mid_vertex.2, low_vertex.2, t1);
            let zl2 = interpolate(high_vertex.2, low_vertex.2, t2);

            let tx1x = interpolate(mid_tx.0, low_tx.0, t1);
            let tx1y = interpolate(mid_tx.1, low_tx.1, t1);
            let tx2x = interpolate(high_tx.0, low_tx.0, t2);
            let tx2y = interpolate(high_tx.1, low_tx.1, t2);

            let nv1x = interpolate(mid_nv.0, low_nv.0, t1);
            let nv1y = interpolate(mid_nv.1, low_nv.1, t1);
            let nv1z = interpolate(mid_nv.2, low_nv.2, t1);
            let nv2x = interpolate(high_nv.0, low_nv.0, t2);
            let nv2y = interpolate(high_nv.1, low_nv.1, t2);
            let nv2z = interpolate(high_nv.2, low_nv.2, t2);

            draw_pixels(
                xl1, xl2,
                zl1, zl2,
                y,
                width,
                &mut zbuffer,
                &mut img,
                &texture,
                &nm,
                tx1x, tx1y, tx2x, tx2y, 
                nv1x, nv1y, nv1z, nv2x, nv2y, nv2z,
                z.2
            );
        }

        for y in (ys0..=mys1).rev() {
            let t1 = find_t(hys1 as f32, mys1 as f32, y as f32);
            let t2 = find_t(hys1 as f32, lys1 as f32, y as f32);
            let xl1 = interpolate(hxs1 as f32, mxs1 as f32, t1);
            let xl2 = interpolate(hxs1 as f32, lxs1 as f32, t2);
            let zl1 = interpolate(high_vertex.2, mid_vertex.2, t1);
            let zl2 = interpolate(high_vertex.2, low_vertex.2, t2);

            let tx1x = interpolate(high_tx.0, mid_tx.0, t1);
            let tx1y = interpolate(high_tx.1, mid_tx.1, t1);
            let tx2x = interpolate(high_tx.0, low_tx.0, t2);
            let tx2y = interpolate(high_tx.1, low_tx.1, t2);

            let nv1x = interpolate(high_nv.0, mid_nv.0, t1);
            let nv1y = interpolate(high_nv.1, mid_nv.1, t1);
            let nv1z = interpolate(high_nv.2, mid_nv.2, t1);
            let nv2x = interpolate(high_nv.0, low_nv.0, t2);
            let nv2y = interpolate(high_nv.1, low_nv.1, t2);
            let nv2z = interpolate(high_nv.2, low_nv.2, t2);

            draw_pixels(
                xl1,
                xl2,
                zl1,
                zl2,
                y,
                width,
                &mut zbuffer,
                &mut img,
                &texture,
                &nm,
                tx1x,
                tx1y,
                tx2x,
                tx2y,
                nv1x,
                nv1y,
                nv1z,
                nv2x,
                nv2y,
                nv2z,
                z.2
            );
        }
    };

    let mut draw = |(v1x, v1y, _): &(f32, f32, f32),
                    (v2x, v2y, _): &(f32, f32, f32),
                    mut img: &mut tga::Image| {
        let (x0, y0) = to_screen_space((*v1x, *v1y));
        let (x1, y1) = to_screen_space((*v2x, *v2y));
        line(x0, y0, x1, y1, &mut img, tga::Color(200, 200, 200));
    };

    for ((v1, v2, v3), (tc1, tc2, tc3)) in &wf.faces {
        let nv1 = wf.normals.get(*v1 as usize).unwrap();
        let nv2 = wf.normals.get(*v2 as usize).unwrap();
        let nv3 = wf.normals.get(*v3 as usize).unwrap();

        let v1 = wf.vertices.get(*v1 as usize).unwrap();
        let v2 = wf.vertices.get(*v2 as usize).unwrap();
        let v3 = wf.vertices.get(*v3 as usize).unwrap();

        let t1 = wf.texture_coord.get(*tc1 as usize).unwrap();
        let t2 = wf.texture_coord.get(*tc2 as usize).unwrap();
        let t3 = wf.texture_coord.get(*tc3 as usize).unwrap();


        triangle(v1, v2, v3, t1, t2, t3, nv1, nv2, nv3, &mut img);
        // draw(v1, v2, &mut img);
        // draw(v2, v3, &mut img);
        // draw(v3, v1, &mut img);
    }

    img.apply_gamma(1.5);
    img.write_to_tga("african_head.tga").unwrap();

    // println!("{:?}", wf);
}

fn draw_pixels(
    mut xl1: f32,
    mut xl2: f32,
    zl1: f32,
    zl2: f32,
    y: i32,
    width: i32,
    zbuffer: &mut Vec<f32>,
    img: &mut tga::Image,
    txt:  &tga::Image,
    nm: &tga::Image,
    mut tx1x: f32,
    mut tx1y: f32,
    mut tx2x: f32,
    mut tx2y: f32,
    mut nv1x: f32,
    mut nv1y: f32,
    mut nv1z: f32,
    mut nv2x: f32,
    mut nv2y: f32,
    mut nv2z: f32,
    get_angle: f32
) {

    if xl1 > xl2 {
        swap(&mut xl1, &mut xl2);
        swap(&mut tx1x, &mut tx2x);
        swap(&mut tx1y, &mut tx2y);
        swap(&mut nv1x, &mut nv2x);
        swap(&mut nv1y, &mut nv2y);
        swap(&mut nv1z, &mut nv2z);
    }
    for x in xl1.round() as i32..=xl2.round() as i32 {
        // if xl1 > xl2 {
        //     swap(&mut xl1, &mut xl2);
        // }
        if x >= xl1.round() as i32 && x <= xl2.round() as i32 {
            let tz = find_t(xl1, xl2, x as f32);
            let zb = interpolate(zl1, zl2, tz);
            let txx = interpolate(tx1x, tx2x, tz);
            let txy = interpolate(tx1y, tx2y, tz);
            let nvx = interpolate(nv1x, nv2x, tz);
            let nvy = interpolate(nv1y, nv2y, tz);
            let nvz = interpolate(nv1z, nv2z, tz);
            let nvz = nvz/(nvx*nvx + nvy*nvy + nvz*nvz).sqrt();
            let txxp = (txx*txt.width as f32).round() as i32;
            let txyp = (txy*txt.height as f32).round() as i32;
            let normal_vec = nm.pixel_at(txxp, txyp);
            let c = txt.pixel_at(txxp, txyp).highlight(normal_vec.2 as f32 / 255.0);
            // let c = Color(200, 200, 200).highlight(nvz);
            let shift: usize = ((y as i32) * width + x as i32) as usize;
            if x >= 0 && x < img.width && y >= 0 && y < img.height && zbuffer.get(shift).unwrap() < &zb {
                img.set_pixel(
                    x.try_into().unwrap(),
                    y.try_into().unwrap(),
                    c,
                );
                let s = zbuffer.get_mut(shift).unwrap();
                *s = zb;
            }
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