use js_sys::Math;
use wasm_bindgen::prelude::*;
use fixedbitset::FixedBitSet;

pub struct Universe {
    width: u32,
    height: u32,
    cells: FixedBitSet,
}

impl Universe {
    pub fn new(width: u32, height: u32) -> Universe {
        let cells = Self::random_cells((width * height) as usize);
        Universe {
            width,
            height,
            cells,
        }
    }

    pub fn randomize_cells(&mut self) {
        self.cells = Self::random_cells((self.width * self.height) as usize);
    }

    pub fn tick(&mut self) {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                // log!(
                //     "cell[{}, {}] is initially {:?} and has {} live neighbors",
                //     row,
                //     col,
                //     cell,
                //     live_neighbors
                // );

                let next_cell = match (cell, live_neighbors) {
                    (true, x) if x < 2 => false,
                    (true, 2) | (true, 3) => true,
                    (true, x) if x > 3 => false,
                    (false, 3) => true,
                    (otherwise, _) => otherwise,
                };

                // log!("  it becomes {:?}", next_cell);
                next.set(idx, next_cell);
            }
        }
        self.cells = next;
    }

    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells.set(idx, !self.cells[idx]);
    }

    pub fn birth_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells.set(idx, true);
    }

    pub fn kill_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells.set(idx, false);
    }

    pub fn clear(&mut self) {
        for i in 0..self.width * self.height {
            self.cells.set(i as usize, false);
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        self.reset();
    }

    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        self.reset();
    }

    pub fn reset(&mut self) {
        for i in 0..self.width() * self.height() {
            self.cells.set(i as usize, false);
        }
    }

    pub fn is_alive(&self, row: u32, column: u32) -> bool {
        let indx = self.get_index(row, column);
        self.cells[indx]
    }

}

impl Universe {
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.cells[idx] as u8;
            }
        }
        count
    }

    fn random_cells(size: usize) -> FixedBitSet {
        let mut cells = FixedBitSet::with_capacity(size);
        for i in 0..size {
            cells.set(i, if Math::random() < 0.5 { true } else { false });
        }
        cells
    }
}

use std::fmt;

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == 0 { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}
