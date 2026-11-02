/// Which bucket set of the grid a query reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridLayer {
    Creatures,
    Food,
}

pub struct SpatialGrid {
    pub cell_size: f32,
    pub cols: usize,
    pub rows: usize,
    creature_buckets: Vec<Vec<u32>>,
    food_buckets: Vec<Vec<u32>>,
}

impl SpatialGrid {
    pub fn new(width: f32, height: f32, cell_size: f32) -> Self {
        let cols = (width / cell_size).ceil() as usize;
        let rows = (height / cell_size).ceil() as usize;
        let total_cells = cols * rows;

        Self {
            cell_size,
            cols,
            rows,
            creature_buckets: vec![Vec::new(); total_cells],
            food_buckets: vec![Vec::new(); total_cells],
        }
    }

    pub fn clear(&mut self) {
        for bucket in &mut self.creature_buckets {
            bucket.clear();
        }
        for bucket in &mut self.food_buckets {
            bucket.clear();
        }
    }

    fn get_cell_index(&self, x: f32, y: f32) -> usize {
        let c = ((x / self.cell_size).floor() as usize).clamp(0, self.cols.saturating_sub(1));
        let r = ((y / self.cell_size).floor() as usize).clamp(0, self.rows.saturating_sub(1));
        c * self.rows + r
    }

    pub fn insert_creature(&mut self, id: u32, x: f32, y: f32) {
        let idx = self.get_cell_index(x, y);
        if idx < self.creature_buckets.len() {
            self.creature_buckets[idx].push(id);
        }
    }

    pub fn insert_food(&mut self, id: u32, x: f32, y: f32) {
        let idx = self.get_cell_index(x, y);
        if idx < self.food_buckets.len() {
            self.food_buckets[idx].push(id);
        }
    }

    /// Calls `f` with every id of `layer` in the cells overlapping the square `[x ± range] × [y ± range]`, column by
    /// column, then row, then insertion order (ids outside the range circle included). No allocation.
    pub fn for_each_nearby(&self, layer: GridLayer, x: f32, y: f32, range: f32, mut f: impl FnMut(u32)) {
        let buckets = match layer {
            GridLayer::Creatures => &self.creature_buckets,
            GridLayer::Food => &self.food_buckets,
        };
        let start_col = (((x - range) / self.cell_size).floor() as usize).clamp(0, self.cols.saturating_sub(1));
        let end_col = (((x + range) / self.cell_size).floor() as usize).clamp(0, self.cols.saturating_sub(1));
        let start_row = (((y - range) / self.cell_size).floor() as usize).clamp(0, self.rows.saturating_sub(1));
        let end_row = (((y + range) / self.cell_size).floor() as usize).clamp(0, self.rows.saturating_sub(1));

        for c in start_col..=end_col {
            for r in start_row..=end_row {
                if let Some(bucket) = buckets.get(c * self.rows + r) {
                    bucket.iter().for_each(|&id| f(id));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_for_each_nearby_visits_overlapping_cells_per_layer() {
        let mut grid = SpatialGrid::new(400.0, 400.0, 100.0);
        grid.insert_creature(1, 50.0, 50.0);
        grid.insert_creature(2, 150.0, 50.0);
        grid.insert_creature(3, 350.0, 350.0);
        grid.insert_food(7, 60.0, 60.0);
        let collect = |layer, x, y, range| {
            let mut ids = Vec::new();
            grid.for_each_nearby(layer, x, y, range, |id| ids.push(id));
            ids
        };
        assert_eq!(collect(GridLayer::Creatures, 60.0, 60.0, 50.0), vec![1, 2]);
        assert_eq!(collect(GridLayer::Creatures, 60.0, 60.0, 10.0), vec![1]);
        assert_eq!(collect(GridLayer::Food, 60.0, 60.0, 10.0), vec![7]);
        assert_eq!(collect(GridLayer::Creatures, 390.0, 390.0, 5.0), vec![3]);
    }
}
