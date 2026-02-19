#[derive(Clone, Debug)]
pub struct BitMatrix {
    pub size_i: usize,
    pub size_j: usize,
    el: Box<[bool]>,
}

#[derive(Clone, Debug)]
pub struct BitVector {
    pub size: usize,
    el: Box<[bool]>,
}

#[derive(Clone, Debug)]
pub struct NfaVector {
    pub size: usize,
    el: Box<[Option<usize>]>,
}

impl BitMatrix {
    fn index(&self, i: usize, j: usize) -> usize {
        self.size_j * i + j
    }

    pub fn new(sx: usize, sy: usize) -> BitMatrix {
        BitMatrix {
            size_i: sx,
            size_j: sy,
            el: vec![false; sx * sy].into_boxed_slice(),
        }
    }

    fn index_iter(&self) -> impl Iterator<Item = (usize, usize)> + use<> {
        let sy = self.size_j;
        (0..self.size_i).flat_map(move |i| (0..sy).map(move |j| (i, j)))
    }

    pub fn enumerate_iter(
        &self,
    ) -> impl Iterator<Item = ((usize, usize), &bool)> {
        self.index_iter().zip(self.el.iter())
    }

    pub fn enumerate_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = ((usize, usize), &mut bool)> {
        self.index_iter().zip(self.el.iter_mut())
    }

    pub fn reset(&mut self) {
        self.enumerate_iter_mut().for_each(|(_, v)| *v = false)
    }

    pub fn set(&mut self, i: usize, j: usize, value: bool) {
        assert!(i < self.size_i);
        assert!(j < self.size_j);
        self.el[self.index(i, j)] = value;
    }

    pub fn get(&self, i: usize, j: usize) -> bool {
        assert!(i < self.size_i);
        assert!(j < self.size_j);
        self.el[self.index(i, j)]
    }

    pub fn add(a: &BitMatrix, b: &BitMatrix, c: &mut BitMatrix) {
        assert_eq!(a.size_i, b.size_i);
        assert_eq!(a.size_j, b.size_j);
        assert_eq!(a.size_i, c.size_i);
        assert_eq!(a.size_j, c.size_j);
        c.enumerate_iter_mut()
            .for_each(|((i, j), value)| *value = a.get(i, j) || b.get(i, j));
    }

    pub fn mult(a: &BitMatrix, b: &BitMatrix, c: &mut BitMatrix) {
        assert_eq!(a.size_i, b.size_j);
        assert_eq!(c.size_i, b.size_i);
        assert_eq!(c.size_j, a.size_j);
        let n = a.size_i;
        c.enumerate_iter_mut().for_each(|((i, j), value)| {
            for k in 0..n {
                if a.get(i, k) && b.get(k, j) {
                    *value = true;
                    return;
                }
            }
            *value = false;
        });
    }
}

impl BitVector {
    pub fn new(size: usize) -> BitVector {
        BitVector {
            size,
            el: vec![false; size].into_boxed_slice(),
        }
    }

    pub fn enumerate_iter(&self) -> impl Iterator<Item = (usize, &bool)> {
        (0..self.size).zip(self.el.iter())
    }

    pub fn enumerate_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (usize, &mut bool)> {
        (0..self.size).zip(self.el.iter_mut())
    }

    pub fn reset(&mut self) {
        self.enumerate_iter_mut().for_each(|(_, v)| *v = false);
    }

    pub fn set(&mut self, i: usize, value: bool) {
        assert!(i < self.size);
        self.el[i] = value;
    }

    pub fn get(&self, i: usize) -> bool {
        assert!(i < self.size);
        self.el[i]
    }

    pub fn add(a: &BitVector, b: &BitVector, c: &mut BitVector) {
        assert_eq!(a.size, b.size);
        assert_eq!(a.size, c.size);
        c.enumerate_iter_mut()
            .for_each(|(i, value)| *value = a.get(i) || b.get(i));
    }

    pub fn mult(a: &BitMatrix, b: &BitVector, c: &mut BitVector) {
        assert_eq!(a.size_i, b.size);
        assert_eq!(a.size_j, c.size);
        let n = a.size_i;
        c.enumerate_iter_mut().for_each(|(i, value)| {
            for k in 0..n {
                if a.get(i, k) && b.get(k) {
                    *value = true;
                    return;
                }
            }
            *value = false;
        })
    }

    pub fn dot(a: &BitVector, b: &BitVector) -> bool {
        assert_eq!(a.size, b.size);
        a.enumerate_iter().any(|(i, value)| *value && b.get(i))
    }
}

impl NfaVector {
    pub fn new(size: usize) -> NfaVector {
        NfaVector {
            size,
            el: vec![None; size].into_boxed_slice(),
        }
    }

    pub fn enumerate_iter(
        &self,
    ) -> impl Iterator<Item = (usize, &Option<usize>)> {
        (0..self.size).zip(self.el.iter())
    }

    pub fn enumerate_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (usize, &mut Option<usize>)> {
        (0..self.size).zip(self.el.iter_mut())
    }

    pub fn reset(&mut self) {
        self.enumerate_iter_mut().for_each(|(_, v)| *v = None);
    }

    pub fn set(&mut self, i: usize, value: Option<usize>) {
        assert!(i < self.size);
        self.el[i] = value;
    }

    pub fn get(&self, i: usize) -> Option<usize> {
        assert!(i < self.size);
        self.el[i]
    }

    pub fn mult(a: &BitMatrix, b: &NfaVector, c: &mut NfaVector) {
        assert_eq!(a.size_i, b.size);
        assert_eq!(a.size_j, c.size);
        let n = a.size_i;
        c.enumerate_iter_mut().for_each(|(i, old_value)| {
            let mut value = None;
            for k in 0..n {
                if a.get(i, k) {
                    value = min_some(value, b.get(k));
                }
            }
            *old_value = value;
        })
    }

    pub fn dot(a: &NfaVector, b: &BitVector) -> Option<usize> {
        assert_eq!(a.size, b.size);
        a.el.iter()
            .zip(b.el.iter())
            .map(|(a, b)| a.and_then(|a| b.then_some(a)))
            .fold(None, min_some)
    }
}

fn min_some(a: Option<usize>, b: Option<usize>) -> Option<usize> {
    match (a, b) {
        (None, None) => None,
        (Some(x), None) | (None, Some(x)) => Some(x),
        (Some(x), Some(y)) => Some(x.min(y)),
    }
}
