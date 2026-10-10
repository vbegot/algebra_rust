use std::ops::{Index, IndexMut};

pub struct Matrix {
    data: Vec<f32>,
    n_rows: usize,
    n_cols: usize,
}

const OOB_MSG: &str = "Matrix index out of bound";

impl Matrix {
    pub fn new(n_rows: usize, n_cols: usize) -> Self {
        let data = vec![0.0; n_rows * n_cols];
        Matrix {
            data,
            n_rows,
            n_cols,
        }
    }

    fn idx(&self, i: usize, j: usize) -> Option<usize> {
        if i >= self.n_rows || j >= self.n_cols {
            return None;
        }
        Some(i * self.n_cols + j)
    }

    pub fn slice(&self, i: usize) -> &[f32] {
        assert!(i < self.n_rows, "{OOB_MSG}");
        let i0 = i * self.n_cols;
        let i1 = i0 + self.n_cols;
        &self.data[i0..i1]
    }

    pub fn get(&self, i: usize, j: usize) -> Option<&f32> {
        self.data.get(self.idx(i, j)?)
    }

    pub fn set(&mut self, i: usize, j: usize, x: f32) {
        let idx = self.idx(i, j).expect(OOB_MSG);
        self.data[idx] = x;
    }
}

impl Index<(usize, usize)> for Matrix {
    type Output = f32;

    fn index(&self, (i, j): (usize, usize)) -> &Self::Output {
        let idx = self.idx(i, j).expect(OOB_MSG);
        &self.data[idx]
    }
}

impl IndexMut<(usize, usize)> for Matrix {
    fn index_mut(&mut self, (i, j): (usize, usize)) -> &mut Self::Output {
        let idx = self.idx(i, j).expect(OOB_MSG);
        &mut self.data[idx]
    }
}
