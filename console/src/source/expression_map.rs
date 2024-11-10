use std::{
    ops::Deref,
    sync::{Arc, RwLock},
};

#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct ExpressionMap {
    inner: Vec<Option<ExpressionRange>>,
}

type Range = std::ops::Range<usize>;

#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct ExpressionRange(Arc<RwLock<Range>>);

impl ExpressionRange {
    fn new(start: usize, end: usize) -> Self {
        ExpressionRange(Arc::new(RwLock::new(Range { start, end })))
    }

    fn set_end(&self, end: usize) {
        let mut lock = self.write().unwrap();
        lock.end = end;
    }

    fn end(&self) -> usize {
        let lock = self.read().unwrap();
        lock.end
    }

    fn set_start(&self, start: usize) {
        let mut lock = self.write().unwrap();
        lock.start = start;
    }

    fn start(&self) -> usize {
        let lock = self.read().unwrap();
        lock.start
    }

    fn range(&self) -> Range {
        let lock = self.read().unwrap();
        lock.clone()
    }

    fn append(&self, idx: usize) {
        self.set_end(idx);
    }

    fn prepend(&self, idx: usize) {
        self.set_start(idx);
    }
}

impl Deref for ExpressionRange {
    type Target = Arc<RwLock<Range>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ExpressionMap {
    pub fn new(count: usize) -> Self {
        Self {
            inner: vec![None; count],
        }
    }

    pub fn get(&self, idx: usize) -> Option<Range> {
        if let Some(exp) = &self.inner[idx] {
            // let exp = *exp.borrow();
            Some(exp.range())
        } else {
            None
        }
    }

    fn set_exp(&mut self, idx: usize, exp: &ExpressionRange) {
        self.inner[idx] = Some(exp.clone());
    }

    fn set_from(&mut self, start: usize, end: usize, exp: &ExpressionRange) {
        for i in start..(end + 1) {
            self.inner[i] = Some(exp.clone());
        }
    }

    // fn set_exp(&mut self, idx: usize, exp: Rc<RefCell<ExpressionIndex>>) {
    //     self.inner[idx] = Some(exp);
    // }

    fn remove(&mut self, idx: usize) {
        self.inner[idx] = None;
    }

    fn join_exp(&mut self, idx: usize, lft_exp: &ExpressionRange, rgt_exp: &ExpressionRange) {
        let idx_exp = &self.inner[idx];
        if idx_exp.is_none() {
            // In some cases lft and rgt may refer to the same expression
            // We cannot have multiple mutable borrows in the same scope
            // So we split the borrows into separate scopes

            // Right must be first as we want to preserve the end idx
            // The left expression will be modified
            let end = rgt_exp.end();
            {
                lft_exp.set_end(end);
            }
            // Iterate the map until the rgt end and set the lft expression
            self.set_from(idx, end, lft_exp);
        }
    }

    fn split_exp(
        &mut self,
        lft_idx: usize,
        lft_exp: &ExpressionRange,
        rgt_idx: usize,
        rgt_exp: &ExpressionRange,
    ) {
        {
            // lft and rgt often refer to the same expression
            // We cannot have multiple mutable borrows in the same scope
            // So we split the borrows into separate scopes
            // Right must be first as we want to capture the end value before modifying the left value
            // let rgt = rgt_exp.borrow();
            let rgt_end = rgt_exp.end();
            let exp = ExpressionRange::new(rgt_idx, rgt_end);
            self.set_from(rgt_idx, rgt_end, &exp);
        }
        // Update A-1 (Left)
        // Expression now ends at A-1 (Left)
        // self.end_exp(lft_idx, &lft_exp);
        lft_exp.set_end(lft_idx);
    }

    pub fn set(&mut self, idx: usize) {
        self._set(idx, true)
    }

    pub fn unset(&mut self, idx: usize) {
        self._set(idx, false);
    }

    fn _set(&mut self, idx: usize, glyph: bool) {
        let (lft_idx, lft_exp) = if idx > 0 {
            let idx = idx - 1;
            let exp = self.inner[idx].clone();
            (idx, exp)
        } else {
            // Index 0 has no left expression
            (0, None)
        };

        let (rgt_idx, rgt_exp) = if idx < self.inner.len() - 1 {
            let idx = idx + 1;
            let exp = self.inner[idx].clone();
            (idx, exp)
        } else {
            // Last index  has no right expression
            (self.inner.len(), None)
        };

        if !glyph {
            self.remove(idx);
        }

        match (lft_exp, rgt_exp) {
            (Some(lft_exp), Some(ref mut rgt_exp)) => {
                if glyph {
                    self.join_exp(idx, &lft_exp, &rgt_exp);
                } else {
                    self.split_exp(lft_idx, &lft_exp, rgt_idx, &rgt_exp);
                }
            }
            (Some(lft_exp), None) => {
                if glyph {
                    lft_exp.append(idx);
                    self.set_exp(idx, &lft_exp);
                } else {
                    lft_exp.set_end(lft_idx);
                }
            }
            (None, Some(rgt_exp)) => {
                if glyph {
                    rgt_exp.prepend(idx);
                    self.set_exp(idx, &rgt_exp);
                } else {
                    rgt_exp.set_start(rgt_idx);
                }
            }
            (None, None) => {
                if glyph {
                    let exp = ExpressionRange::new(idx, idx);
                    self.set_exp(idx, &exp);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{source::ExpressionMap, test::trace};

    #[test]
    fn test_expression_join() {
        trace();

        let mut map = ExpressionMap::new(10);

        map.set(0);
        map.set(1);
        map.set(3);

        let exp = map.get(0).unwrap();
        assert_eq!(exp, 0..1);

        let exp = map.get(1).unwrap();
        assert_eq!(exp, 0..1);

        let exp = map.get(3).unwrap();
        assert_eq!(exp, 3..3);

        map.set(2);

        let exp = map.get(2).unwrap();
        assert_eq!(exp, 0..3);

        let exp = map.get(3).unwrap();
        assert_eq!(exp, 0..3);
    }

    #[test]
    fn test_expression_split() {
        trace();

        let mut map = ExpressionMap::new(10);

        // {++AABB}
        map.set(0);
        map.set(1);
        map.set(2);
        map.set(3);
        map.set(4);
        map.set(5);

        let exp = map.get(0).unwrap();
        assert_eq!(exp, 0..5);

        // {++ ABB}
        map.unset(2);

        let exp = map.get(0).unwrap();
        assert_eq!(exp, 0..1);
    }

    #[test]
    fn test_expression_prepend() {
        trace();

        let mut map = ExpressionMap::new(10);

        // {     IDAA}
        map.set(6);
        map.set(7);
        map.set(8);
        map.set(9);

        let exp = map.get(6).unwrap();
        assert_eq!(exp, 6..9);

        // {    XIDAA}
        map.set(5);

        let exp = map.get(5).unwrap();
        assert_eq!(exp, 5..9);
    }

    #[test]
    fn test_expression_insert() {
        trace();

        let mut map = ExpressionMap::new(10);

        // {IDAA       }
        map.set(0);
        map.set(1);
        map.set(2);
        map.set(3);

        let exp = map.get(0).unwrap();
        assert_eq!(exp, 0..3);
        // {ID0A       }
        map.set(2);

        let exp = map.get(0).unwrap();
        assert_eq!(exp, 0..3);

        let exp = map.get(7).is_none();
        assert_eq!(exp, true);
    }

    #[test]
    fn test_expression_delete_last() {
        trace();

        let mut map = ExpressionMap::new(10);

        map.set(8);
        map.set(9);

        let exp = map.get(9).unwrap();
        assert_eq!(exp, 8..9);

        // {          A}
        map.unset(8);

        let exp = map.get(9).unwrap();
        assert_eq!(exp, 9..9);

        // {          A}
        map.unset(9);

        let exp = map.get(8).is_none();
        assert_eq!(exp, true);

        let exp = map.get(9).is_none();
        assert_eq!(exp, true);
    }
}
