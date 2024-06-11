use super::cell::Cell;

pub struct Game {
    cells: Vec<Vec<Option<Cell>>>,
    gen_num: u32
}

impl Game {
    pub fn randomized_board(size_x: usize, size_y: usize) -> Self {
        let mut cells = vec![vec![None; size_x];size_y];
        for i in 0..size_x {
            for j in 0..size_y {
                let cell = Cell::new(i as u32, j as u32);
                cells[i][j] = Some(cell);
            }
        }
        
        return Game {
            cells,
            gen_num: 0
        }
    }
}
