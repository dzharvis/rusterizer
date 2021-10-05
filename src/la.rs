#[derive(Clone, Debug)]
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
        Matrix(v)
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
}

impl Into<Matrix<1, 3>> for &Vec3f {
    fn into(self) -> Matrix<1, 3> {
        Matrix([[self.0], [self.1], [self.2]])
    }
}

#[derive(Clone, Debug)]
pub struct Matrix<const X: usize, const Y: usize>(pub [[f32; X]; Y]);

impl<const T: usize> Into<Vec3f> for Matrix<1, T> {
    fn into(self) -> Vec3f {
        assert!(self.0[0].len() == 1);
        return Vec3f(self.0[0][0], self.0[1][0], self.0[2][0]);
    }
}

impl<const X: usize, const Y: usize> Matrix<X, Y> {
    pub fn zeroed() -> Self {
        Matrix([[0.0f32; X]; Y])
    }

    pub fn inverse(&self) -> Self
    where [(); X*2]: Sized  {
        assert!(self.0.len() == self.0[0].len());
        let n = self.0.len();
        let mut aug = {
            let mut r: Matrix<{X*2}, Y> = Matrix::zeroed();
            for y in 0..n {
                for x in 0..n {
                    r.0[y][x] = self.0[y][x];
                }
            }
            for y in 0..n {
                for x in 0..n {
                    r.0[y][n + x] = if x == y { 1.0 } else { 0.0 };
                }
            }
            r
        };
        for y in 0..n {
            if aug.0[y][y] == 0.0f32 {
                panic!("it's a bad idea to divide by zero");
            }
            for x in 0..n {
                if x != y {
                    let r = aug.0[x][y] / aug.0[y][y];
                    for k in 0..n * 2 {
                        aug.0[x][k] = aug.0[x][k] - r * aug.0[y][k];
                    }
                }
            }
        }

        for y in 0..n {
            for x in n..n * 2 {
                aug.0[y][x] = aug.0[y][x] / aug.0[y][y];
            }
        }

        let mut res: Matrix<X, Y> = Matrix::zeroed();
        for y in 0..n {
            for x in n..n * 2 {
                res.0[y][x - n] = aug.0[y][x];
            }
        }

        return res;
    }

    pub fn transpose(&self) -> Matrix<Y, X> {
        let mut res = Matrix::zeroed();
        for x in 0..self.0[0].len() {
            for y in 0..self.0.len() {
                res.0[x][y] = self.0[y][x];
            }
        }
        return res;
    }

    pub fn mul<const XX: usize, const YY: usize>(&self, matrix: &Matrix<XX, YY>) -> Matrix<XX, Y> {
        assert!(self.0[0].len() == matrix.0.len());

        let mut res = Matrix::zeroed();
        // let tm = matrix.transpose(); // transposing doesn't give any speed improvement :/
        // looks like compilers are smart enough for this optimization
        // also probably isn't worse it in case of 3x3 matrices
        for y in 0..self.0.len() {
            for x in 0..matrix.0[0].len() {
                for p in 0..self.0[0].len() {
                    res.0[y][x] += self.0[y][p] * matrix.0[p][x];
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
