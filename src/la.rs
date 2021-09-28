#[derive(Clone, Debug)]
pub struct Vec3f(pub f32, pub f32, pub f32);

impl Vec3f {
    pub fn zeroed() -> Self {
        Vec3f(0.0, 0.0, 0.0)
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
}

impl Into<Matrix> for &Vec3f {
    fn into(self) -> Matrix {
        Matrix(vec![
            vec![self.0], 
            vec![self.1], 
            vec![self.2]
        ])
    }
}

#[derive(Clone, Debug)]
pub struct Matrix(pub Vec<Vec<f32>>);

impl Into<Vec3f> for Matrix {
    fn into(self) -> Vec3f {
        assert!(self.0.len() == 3 && self.0[0].len() == 1);
        Vec3f(self.0[0][0], self.0[1][0], self.0[2][0])
    }
}


impl Matrix {
    pub fn zeroed(x: usize, y: usize) -> Self {
        Matrix(vec![vec![0.0f32; x]; y])
    }

    // pub fn random(x: usize, y: usize) -> Self {
    //     let offset = Instant::now().checked_sub(Duration::new(1, 0)).unwrap();
    //     fn get_row(x: usize, offset: Instant) -> Vec<f32> {
    //         (0..x)
    //         .map(|_| (Instant::now().duration_since(offset).as_nanos() % 17) as f32)
    //         .collect()
    //     }
    //     Matrix((0..y)
    //     .map(|_| get_row(x, offset))
    //     .collect())
    // }

    pub fn transpose(&self) -> Self {
        let mut res = Matrix::zeroed(self.0.len(), self.0[0].len());
        for x in 0..self.0[0].len() {
            for y in 0..self.0.len() {
                res.0[x][y] = self.0[y][x];
            }
        }
        return res;
    }

    pub fn mul(&self, matrix: &Matrix) -> Self {
        assert!(self.0[0].len() == matrix.0.len());
        let mut res = Matrix::zeroed(matrix.0[0].len(), self.0.len());
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
    fn test_matrix() {
        // assert_eq!(
        //     0.0,
        //     get_angle(&(0.0, 0.0, 0.0), &(0.0, 0.0, 0.0), &(0.0, 0.0, 0.0))
        // );
        let mut m1 = Matrix::zeroed(3, 3);
        m1.0 = vec![
            vec![1.0, 1.1, 1.2],
        ];
        let mut m2 = Matrix::zeroed(3, 3);
        m2.0 = vec![
            vec![1.0],
            vec![2.0],
            vec![3.0],
        ];

        // let v = Vec3f(1.0, 2.0, 3.0);

        println!("{:?}", m1.mul(&m2));
        // println!("{:?}", m2.mul(v.into()));
        // println!("{:?}", m1.transpose());
    }
}
