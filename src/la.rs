#[derive(Clone, Debug, Copy)]
pub struct Vec3f(pub f32, pub f32, pub f32);

impl Vec3f {
    pub fn zeroed() -> Self {
        Vec3f(0.0, 0.0, 0.0)
    }

    pub fn embed<const L: usize>(&self, i: f32) -> Matrix<1, L> {
        assert!(L > 3);
        let mut v = [[i]; L];
        v[0][0] = self.0;
        v[1][0] = self.1;
        v[2][0] = self.2;
        v
    }

    pub fn cross(&self, v: &Vec3f) -> Self {
        Vec3f(
            self.1 * v.2 - self.2 * v.1,
            self.2 * v.0 - self.0 * v.2,
            self.0 * v.1 - self.1 * v.0,
        )
    }

    pub fn normalize(&self) -> Self {
        let mag = (self.0 * self.0 + self.1 * self.1 + self.2 * self.2).sqrt();
        let mag = if mag == 0.0 { f32::MIN } else { mag };
        Vec3f(self.0 / mag, self.1 / mag, self.2 / mag)
    }

    pub fn sub(&self, v: &Vec3f) -> Self {
        Vec3f(self.0 - v.0, self.1 - v.1, self.2 - v.2)
    }

    pub fn add(&self, v: &Vec3f) -> Self {
        Vec3f(self.0 + v.0, self.1 + v.1, self.2 + v.2)
    }

    pub fn mul(&self, v: &Vec3f) -> f32 {
        self.0 * v.0 + self.1 * v.1 + self.2 * v.2
    }

    pub fn mulf(&self, v: f32) -> Vec3f {
        Vec3f(self.0 * v, self.1 * v, self.2 * v)
    }

    pub fn rotate(&self, x: f32, y: f32) -> Vec3f {
        let xm: Matrix<3, 3> = [
            [1.0, 0.0, 0.0],
            [0.0, x.cos(), -x.sin()],
            [0.0, x.sin(), x.cos()],
        ];

        let ym: Matrix<3, 3> = [
            [y.cos(), 0.0, y.sin()],
            [0.0, 1.0, 0.0],
            [-y.sin(), 0.0, y.cos()],
        ];

        ym.mul(&xm.mul(&self.into())).into()
    }
}

impl Into<Matrix<1, 3>> for &Vec3f {
    fn into(self) -> Matrix<1, 3> {
        [[self.0], [self.1], [self.2]]
    }
}

pub trait MatrixI<const X: usize, const Y: usize> {
    fn zeroed() -> Self;
    fn inverse(&self) -> Self where [(); X*2]: Sized;
    fn transpose(&self) -> Matrix<Y, X>;
    fn mul<const XX: usize, const YY: usize>(&self, matrix: &Matrix<XX, YY>) -> Matrix<XX, Y>;
}

pub type Matrix<const X: usize, const Y: usize> = [[f32; X]; Y];

impl<const T: usize> Into<Vec3f> for Matrix<1, T> {
    fn into(self) -> Vec3f {
        assert!(self[0].len() == 1);
        return Vec3f(self[0][0], self[1][0], self[2][0]);
    }
}

impl<const X: usize, const Y: usize> MatrixI<X, Y> for Matrix<X, Y> {
    fn zeroed() -> Self {
        [[0.0f32; X]; Y]
    }

    fn inverse(&self) -> Self
    where [(); X*2]: Sized  {
        assert!(self.len() == self[0].len());
        let n = self.len();
        let mut aug = {
            let mut r: Matrix<{X*2}, Y> = Matrix::zeroed();
            for y in 0..n {
                for x in 0..n {
                    r[y][x] = self[y][x];
                }
            }
            for y in 0..n {
                for x in 0..n {
                    r[y][n + x] = if x == y { 1.0 } else { 0.0 };
                }
            }
            r
        };
        for y in 0..n {
            if aug[y][y] == 0.0f32 {
                panic!("it's a bad idea to divide by zero");
            }
            for x in 0..n {
                if x != y {
                    let r = aug[x][y] / aug[y][y];
                    for k in 0..n * 2 {
                        aug[x][k] = aug[x][k] - r * aug[y][k];
                    }
                }
            }
        }

        for y in 0..n {
            for x in n..n * 2 {
                aug[y][x] = aug[y][x] / aug[y][y];
            }
        }

        let mut res: Matrix<X, Y> = Matrix::zeroed();
        for y in 0..n {
            for x in n..n * 2 {
                res[y][x - n] = aug[y][x];
            }
        }

        return res;
    }

    fn transpose(&self) -> Matrix<Y, X> {
        let mut res = Matrix::zeroed();
        for x in 0..self[0].len() {
            for y in 0..self.len() {
                res[x][y] = self[y][x];
            }
        }
        return res;
    }

    fn mul<const XX: usize, const YY: usize>(&self, matrix: &Matrix<XX, YY>) -> Matrix<XX, Y> {
        assert!(self[0].len() == matrix.len());

        let mut res = Matrix::zeroed();
        // let tm = matrix.transpose(); // transposing doesn't give any speed improvement :/
        // looks like compilers are smart enough for this optimization
        // also probably isn't worth it in case of 3x3 matrices
        for y in 0..self.len() {
            for x in 0..matrix[0].len() {
                for p in 0..self[0].len() {
                    res[y][x] += self[y][p] * matrix[p][x];
                }
            }
        }
        res
    }
}

pub fn find_t(a: f32, b: f32, m: f32) -> f32 {
    (m - a) / (b - a)
}

pub fn interpolate(a: f32, b: f32, t: f32) -> f32 {
    a * (1f32 - t) + b * t
}

pub fn interpolatev(a: &Vec3f, b: &Vec3f, t: f32) -> Vec3f {
    Vec3f(interpolate(a.0, b.0, t), interpolate(a.1, b.1, t), interpolate(a.2, b.2, t))
}

pub fn barycentric(a: &Vec3f, b: &Vec3f, c: &Vec3f, p: (f32, f32)) -> Vec3f {
    let cross = Vec3f(c.0 - a.0, b.0 - a.0, a.0 - p.0).cross(&Vec3f(c.1 - a.1, b.1 - a.1, a.1 - p.1));
    Vec3f(1.0 - (cross.1 + cross.0)/cross.2, cross.1/cross.2, cross.0/cross.2,)
}

pub fn persp(c: f32, v1: &Vec3f) -> Vec3f {
    Vec3f(
        v1.0 / (1.08 - v1.2 / c),
        v1.1 / (1.08 - v1.2 / c),
        v1.2 / (1.08 - v1.2 / c),
    )
}

pub fn get_look_at(p: &Vec3f, c: &Vec3f) -> Matrix<4, 4> {
    let up = Vec3f(0.0, 1.0, 0.0);

    let z = p.sub(&c).normalize();
    let x = up.cross(&z).normalize();
    let y = z.cross(&x).normalize();

    let minv = [
        [x.0, x.1, x.2, 0.0],
        [y.0, y.1, y.2, 0.0],
        [z.0, z.1, z.2, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];

    let tr = [
        [1.0, 0.0, 0.0, -c.0],
        [0.0, 1.0, 0.0, -c.1],
        [0.0, 0.0, 1.0, -c.2],
        [0.0, 0.0, 0.0, 1.0],
    ];

    let mv = minv.mul(&tr); // 4x4
    return mv;
}

pub fn look_at(m: &Matrix<4, 4>, v: &Vec3f) -> Vec3f {
    let r = m.mul(&v.embed::<4>(1.0));
    Vec3f(r[0][0] / r[3][0], r[1][0] / r[3][0], r[2][0] / r[3][0])
}

pub fn to_screen_space(v: &Vec3f, width: i32, height: i32) -> Vec3f {
    let x0 = (v.0 + 1.) * (width - 1) as f32 / 2.;
    let y0 = (v.1 + 1.) * (height - 1) as f32 / 2.;
    Vec3f(x0, y0, ((v.2 + 1.) / 2.) * 255.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_time() {
    //     let mut acc = 0.0;
    //     let matrices: Vec<(Matrix, Matrix)> = (0..100).map(|_| (Matrix::random(200, 200), Matrix::random(200, 200))).collect();
    //     let start = Instant::now();
    //     for (m1, m2) in matrices {
    //         let r = m2.mul(&m1);
    //         acc += r.0[0][0];
    //     }
    //     println!("{:?}, blackhole: {:?}", start.elapsed(), acc);
    // }

    #[test]
    fn test_matrix_inv() {
        // let m = Matrix(vec![
        //     vec![1.0, 2.0, 1.0, 0.0],
        //     vec![0.0, 2.0, 1.0, 3.0],
        //     vec![1.0, 2.0, 0.0, 1.0],
        //     vec![1.0, 2.0, 0.0, 3.0],
        // ]);
        // println!("{:?}", m.inverse());
    }

    #[test]
    fn test_matrix() {
        // assert_eq!(
        //     0.0,
        //     get_angle(&(0.0, 0.0, 0.0), &(0.0, 0.0, 0.0), &(0.0, 0.0, 0.0))
        // );
        // let mut m1 = Matrix::zeroed(3, 3);
        // m1.0 = vec![vec![1.0, 1.1, 1.2]];
        // let mut m2 = Matrix::zeroed(3, 3);
        // m2.0 = vec![vec![1.0], vec![2.0], vec![3.0]];

        // // let v = Vec3f(1.0, 2.0, 3.0);

        // println!("{:?}", m1.mul(&m2));
        // println!("{:?}", m2.mul(v.into()));
        // println!("{:?}", m1.transpose());
    }
}
