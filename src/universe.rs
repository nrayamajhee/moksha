use fixedbitset::FixedBitSet;
use js_sys::Math;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pattern {
    GLIDER,
    GUN,
    PULSAR,
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: FixedBitSet,
}

#[wasm_bindgen]
impl Universe {
    pub fn new(width: u32, height: u32) -> Universe {
        let cells = Self::empty_cells((width * height) as usize);
        Universe {
            width,
            height,
            cells,
        }
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

    pub fn tick(&mut self) {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                let next_cell = match (cell, live_neighbors) {
                    (true, x) if x < 2 => false,
                    (true, 2) | (true, 3) => true,
                    (true, x) if x > 3 => false,
                    (false, 3) => true,
                    (otherwise, _) => otherwise,
                };

                next.set(idx, next_cell);
            }
        }
        self.cells = next;
    }

    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    pub fn is_alive(&self, row: u32, column: u32) -> bool {
        let indx = self.get_index(row, column);
        self.cells[indx]
    }

    pub fn birth_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells.set(idx, true);
    }

    pub fn kill_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells.set(idx, false);
    }

    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        if self.is_alive(row, column) {
            self.kill_cell(row, column);
        } else {
            self.birth_cell(row, column);
        }
    }

    fn random_cells(size: usize) -> FixedBitSet {
        let mut cells = FixedBitSet::with_capacity(size);
        for i in 0..size {
            cells.set(i, if Math::random() < 0.5 { true } else { false });
        }
        cells
    }

    fn empty_cells(size: usize) -> FixedBitSet {
        let mut cells = FixedBitSet::with_capacity(size);
        for i in 0..size {
            cells.set(i, false);
        }
        cells
    }

    pub fn randomize_cells(&mut self) {
        self.cells = Self::random_cells((self.width * self.height) as usize);
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
        self.clear();
    }

    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        self.clear();
    }

    pub fn cells(&self) -> *const u32 {
        self.cells.as_slice().as_ptr()
    }
    pub fn insert(&mut self, pattern: Pattern, c: u32, r: u32) {
        match pattern {
            Pattern::GLIDER => {
                self.set_cells(&[
                    (r, c + 1),
                    (r + 1, c + 2),
                    (r + 2, c),
                    (r + 2, c + 1),
                    (r + 2, c + 2),
                ]);
            },
            Pattern::GUN => {
                self.set_cells(&[
                    (r + 1, c),
                    (r, c - 1),
                    (r + 1, c - 1),
                    (r + 2, c - 1),
                    (r - 1, c - 2),
                    (r + 3, c - 2),
                    (r + 1, c - 3),

                    (r - 2, c - 4),
                    (r + 4, c - 4),
                    (r - 2, c - 5),
                    (r + 4, c - 5),
                    (r - 1, c - 6),
                    (r + 3, c - 6),
                    (r, c - 7),
                    (r + 1, c - 7),
                    (r + 2, c - 7),

                    (r, c - 16),
                    (r, c - 17),
                    (r + 1, c - 16),
                    (r + 1, c - 17),

                    (r, c + 3),
                    (r - 1, c + 3),
                    (r - 2, c + 3),
                    (r, c + 4),
                    (r - 1, c + 4),
                    (r - 2, c + 4),
                    (r + 1, c + 5),
                    (r - 3, c + 5),
                    (r - 3, c + 7),
                    (r - 4, c + 7),
                    (r + 1, c + 7),
                    (r + 2, c + 7),

                    (r - 1, c + 17),
                    (r - 2, c + 17),
                    (r - 1, c + 18),
                    (r - 2, c + 18),
                ]);
            },
            Pattern::PULSAR => {
                self.set_cells(&[
                    (r - 2, c + 1),
                    (r - 3, c + 1),
                    (r - 4, c + 1),
                    (r - 2, c - 1),
                    (r - 3, c - 1),
                    (r - 4, c - 1),

                    (r - 1, c + 2),
                    (r - 1, c + 3),
                    (r - 1, c + 4),
                    (r + 1, c + 2),
                    (r + 1, c + 3),
                    (r + 1, c + 4),

                    (r + 2, c - 1),
                    (r + 3, c - 1),
                    (r + 4, c - 1),
                    (r + 2, c + 1),
                    (r + 3, c + 1),
                    (r + 4, c + 1),

                    (r - 1, c - 2),
                    (r - 1, c - 3),
                    (r - 1, c - 4),
                    (r + 1, c - 2),
                    (r + 1, c - 3),
                    (r + 1, c - 4),

                    (r - 6, c - 2),
                    (r - 6, c - 3),
                    (r - 6, c - 4),
                    (r - 6, c + 2),
                    (r - 6, c + 3),
                    (r - 6, c + 4),

                    (r + 6, c - 2),
                    (r + 6, c - 3),
                    (r + 6, c - 4),
                    (r + 6, c + 2),
                    (r + 6, c + 3),
                    (r + 6, c + 4),

                    (r - 2, c + 6),
                    (r - 3, c + 6),
                    (r - 4, c + 6),
                    (r + 2, c + 6),
                    (r + 3, c + 6),
                    (r + 4, c + 6),

                    (r - 2, c - 6),
                    (r - 3, c - 6),
                    (r - 4, c - 6),
                    (r + 2, c - 6),
                    (r + 3, c - 6),
                    (r + 4, c - 6),
                ]);
            }
        }
    }
}

// can't be exported with wasm_bindgen
impl Universe {
    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells.set(idx, true);
        }
    }
    pub fn get_cells(&self) -> &[u32] {
        self.cells.as_slice()
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
