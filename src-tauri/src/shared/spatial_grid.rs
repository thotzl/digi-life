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

    pub fn get_nearby_creatures(&self, x: f32, y: f32, range: f32) -> Vec<u32> {
        let start_col = ((x - range) / self.cell_size).floor() as usize;
        let end_col = ((x + range) / self.cell_size).floor() as usize;
        let start_row = ((y - range) / self.cell_size).floor() as usize;
        let end_row = ((y + range) / self.cell_size).floor() as usize;

        let start_col = start_col.clamp(0, self.cols.saturating_sub(1));
        let end_col = end_col.clamp(0, self.cols.saturating_sub(1));
        let start_row = start_row.clamp(0, self.rows.saturating_sub(1));
        let end_row = end_row.clamp(0, self.rows.saturating_sub(1));

        let mut result = Vec::new();
        for c in start_col..=end_col {
            for r in start_row..=end_row {
                let idx = c * self.rows + r;
                if idx < self.creature_buckets.len() {
                    result.extend(&self.creature_buckets[idx]);
                }
            }
        }
        result
    }

    pub fn get_nearby_food(&self, x: f32, y: f32, range: f32) -> Vec<u32> {
        let start_col = ((x - range) / self.cell_size).floor() as usize;
        let end_col = ((x + range) / self.cell_size).floor() as usize;
        let start_row = ((y - range) / self.cell_size).floor() as usize;
        let end_row = ((y + range) / self.cell_size).floor() as usize;

        let start_col = start_col.clamp(0, self.cols.saturating_sub(1));
        let end_col = end_col.clamp(0, self.cols.saturating_sub(1));
        let start_row = start_row.clamp(0, self.rows.saturating_sub(1));
        let end_row = end_row.clamp(0, self.rows.saturating_sub(1));

        let mut result = Vec::new();
        for c in start_col..=end_col {
            for r in start_row..=end_row {
                let idx = c * self.rows + r;
                if idx < self.food_buckets.len() {
                    result.extend(&self.food_buckets[idx]);
                }
            }
        }
        result
    }
}
