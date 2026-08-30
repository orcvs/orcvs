use crate::grid::Grid;

const SPACE_BYTE: u8 = b' ';

#[derive(Clone, Debug)]
pub struct ExpressionMap {
    inner: Vec<Option<Range>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Range {
    start: usize,
    end: usize,
}

impl Range {
    fn new(start: usize, end: usize) -> Self {
        assert!(
            start <= end,
            "Expression range start must not exceed its end"
        );
        Self { start, end }
    }

    pub fn start(self) -> usize {
        self.start
    }

    pub fn end(self) -> usize {
        self.end
    }
}

impl ExpressionMap {
    ///
    /// Derives every Cell's Expression extent from one complete Source.
    /// Expressions are contiguous occupied Cells confined to one Grid row.
    ///
    pub fn build(grid: Grid, bytes: &[u8]) -> Self {
        assert_eq!(
            bytes.len(),
            grid.count(),
            "ExpressionMap Source length must match its Grid"
        );

        let mut inner = Vec::with_capacity(bytes.len());
        for (row_number, row) in bytes.chunks_exact(grid.cols()).enumerate() {
            inner.extend(row_extents(row_number * grid.cols(), row));
        }

        Self { inner }
    }

    ///
    /// Answers the Expression extent that would contain `idx` after replacing
    /// that Cell with `byte`, without cloning or scanning any other row.
    ///
    pub fn prospective_range(grid: Grid, bytes: &[u8], idx: usize, byte: u8) -> Option<Range> {
        assert_eq!(
            bytes.len(),
            grid.count(),
            "ExpressionMap Source length must match its Grid"
        );
        assert!(
            idx < bytes.len(),
            "prospective Cell must belong to the Source"
        );

        let cols = grid.cols();
        let row_start = (idx / cols) * cols;
        let mut row = bytes[row_start..row_start + cols].to_vec();
        let row_idx = idx - row_start;
        row[row_idx] = byte;

        row_extents(row_start, &row)[row_idx]
    }

    pub fn get(&self, idx: usize) -> Option<Range> {
        self.inner[idx]
    }
}

///
/// The one rule for Expression extent. A future Comment implementation belongs
/// here and must exclude the entire `#` suffix of a row from Expressions.
///
fn row_extents(row_start: usize, row: &[u8]) -> Vec<Option<Range>> {
    let mut extents = vec![None; row.len()];
    let mut local_start = 0;

    while local_start < row.len() {
        if row[local_start] == SPACE_BYTE {
            local_start += 1;
            continue;
        }

        let local_end = row[local_start..]
            .iter()
            .position(|&byte| byte == SPACE_BYTE)
            .map_or(row.len() - 1, |offset| local_start + offset - 1);
        let range = Range::new(row_start + local_start, row_start + local_end);
        extents[local_start..=local_end].fill(Some(range));
        local_start = local_end + 1;
    }

    extents
}

#[cfg(test)]
mod tests {
    use crate::{grid::Grid, source::expression_map::Range};

    use super::ExpressionMap;

    fn assert_range(map: &ExpressionMap, start: usize, end: usize) {
        for idx in start..=end {
            assert_eq!(map.get(idx), Some(Range::new(start, end)));
        }
    }

    #[test]
    fn build_leaves_empty_rows_without_expressions() {
        let map = ExpressionMap::build(Grid::new(5, 1), b"     ");

        for idx in 0..5 {
            assert_eq!(map.get(idx), None);
        }
    }

    #[test]
    fn build_maps_every_cell_in_one_run_to_its_inclusive_extent() {
        let map = ExpressionMap::build(Grid::new(5, 1), b" id1 ");

        assert_eq!(map.get(0), None);
        assert_range(&map, 1, 3);
        assert_eq!(map.get(4), None);
    }

    #[test]
    fn build_separates_multiple_runs_in_one_row() {
        let map = ExpressionMap::build(Grid::new(8, 1), b"id  .+  ");

        assert_range(&map, 0, 1);
        assert_eq!(map.get(2), None);
        assert_eq!(map.get(3), None);
        assert_range(&map, 4, 5);
        assert_eq!(map.get(6), None);
        assert_eq!(map.get(7), None);
    }

    #[test]
    fn build_keeps_edge_touching_runs_inside_their_rows() {
        let map = ExpressionMap::build(Grid::new(4, 2), b"  id.+  ");

        assert_range(&map, 2, 3);
        assert_range(&map, 4, 5);
        assert_ne!(map.get(3), map.get(4));
    }

    #[test]
    fn prospective_range_scans_only_the_edited_row() {
        let grid = Grid::new(5, 2);
        let bytes = b"id   .+   ";

        assert_eq!(
            ExpressionMap::prospective_range(grid, bytes, 2, b'1'),
            Some(Range::new(0, 2))
        );
        assert_eq!(
            ExpressionMap::prospective_range(grid, bytes, 7, b'0'),
            Some(Range::new(5, 7))
        );
    }

    #[test]
    #[should_panic(expected = "ExpressionMap Source length must match its Grid")]
    fn build_rejects_source_content_with_the_wrong_length() {
        let _ = ExpressionMap::build(Grid::new(5, 1), b"    ");
    }
}
