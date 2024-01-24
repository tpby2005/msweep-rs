use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor},
    terminal,
};
use rand::Rng;
use std::io::{stdout, Write};

struct Board {
    board: Vec<Vec<Cell>>,
    cursor_x: usize,
    cursor_y: usize,
    game_over: bool,
}

#[derive(Clone)]
struct Cell {
    state: CellState,
    mine: bool,
}

#[derive(Clone)]
enum CellState {
    Unrevealed,
    Revealed(usize),
    Flagged,
}

impl Board {
    fn new(board_size: usize, num_mines: usize) -> Board {
        let mut board = Board {
            board: vec![vec![Cell::new(); board_size]; board_size],
            cursor_x: 0,
            cursor_y: 0,
            game_over: false,
        };

        let mut rng = rand::thread_rng();
        for _ in 0..num_mines {
            let x = rng.gen_range(0..board_size);
            let y = rng.gen_range(0..board_size);
            board.board[x][y].mine = true;
        }

        board
    }

    fn count_mines(&self, x: usize, y: usize) -> u8 {
        let mut count = 0;
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0
                    && nx < self.get_board_size() as i32
                    && ny >= 0
                    && ny < self.get_board_size() as i32
                {
                    if self.board[nx as usize][ny as usize].contains_mine() {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    fn reveal(&mut self, x: usize, y: usize) {
        let cell = &mut self.board[x][y];
        if cell.mine {
            self.game_over = true;
            return;
        }

        let mines = self.count_mines(x, y);
        let cell = &mut self.board[x][y];
        match cell.state {
            CellState::Revealed(_) | CellState::Flagged => return,
            CellState::Unrevealed => {
                cell.state = CellState::Revealed(mines as usize);
            }
        }

        if mines == 0 {
            let mut to_reveal = Vec::new();
            for dx in -1..=1 {
                for dy in -1..=1 {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0
                        && ny >= 0
                        && nx < self.board.len() as i32
                        && ny < self.board[0].len() as i32
                    {
                        to_reveal.push((nx as usize, ny as usize));
                    }
                }
            }
            for (nx, ny) in to_reveal {
                self.reveal(nx, ny);
            }
        }
    }

    fn flag(&mut self, x: usize, y: usize) {
        let cell = &mut self.board[x][y];
        match cell.state {
            CellState::Unrevealed => {
                cell.state = CellState::Flagged;
            }
            CellState::Revealed(_) => {}
            CellState::Flagged => {
                cell.state = CellState::Unrevealed;
            }
        }
    }

    fn print(&self) {
        let mut stdout = stdout();

        for (i, row) in self.board.iter().enumerate() {
            for (j, cell) in row.iter().enumerate() {
                let output = match cell.state {
                    CellState::Unrevealed => "*".to_string(),
                    CellState::Revealed(0) => " ".to_string(),
                    CellState::Revealed(n) => n.to_string(),
                    CellState::Flagged => "F".to_string(),
                };

                execute!(stdout, MoveTo(j as u16, i as u16)).unwrap();

                if self.cursor_x == i && self.cursor_y == j {
                    execute!(
                        stdout,
                        SetBackgroundColor(Color::Blue),
                        Print(output),
                        ResetColor
                    )
                    .unwrap();
                } else {
                    execute!(stdout, Print(output)).unwrap();
                }

                execute!(stdout, MoveTo((j + 1) as u16, i as u16)).unwrap();
            }
            execute!(stdout, MoveTo(0, (i + 1) as u16)).unwrap();

            execute!(stdout, MoveTo(0, (self.get_board_size() + 2) as u16)).unwrap();
            execute!(
                stdout,
                Print("Use arrow keys to move, space to reveal, f to flag, and q to quit")
            )
            .unwrap();
        }
    }

    fn set_game_over(&mut self, game_over: bool) {
        self.game_over = game_over;
    }

    fn get_board_size(&self) -> usize {
        self.board.len()
    }
}

impl Cell {
    fn new() -> Cell {
        Cell {
            state: CellState::Unrevealed,
            mine: false,
        }
    }

    fn contains_mine(&self) -> bool {
        self.mine
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        println!("Usage: minesweeper <board size> <number of mines>");
        return;
    }

    let board_size = args[1].parse::<usize>().unwrap_or(10);
    let num_mines = args[2].parse::<usize>().unwrap_or(10);

    let mut stdout = stdout();
    let mut board = Board::new(board_size, num_mines);

    crossterm::terminal::enable_raw_mode().unwrap();

    loop {
        execute!(stdout, terminal::Clear(terminal::ClearType::All)).unwrap();
        board.print();
        stdout.flush().unwrap();

        let input = crossterm::event::read().unwrap();

        match input {
            crossterm::event::Event::Key(key) => match key.code {
                crossterm::event::KeyCode::Char('q') => break,
                crossterm::event::KeyCode::Up => {
                    if board.cursor_x > 0 {
                        board.cursor_x -= 1;
                    }
                }
                crossterm::event::KeyCode::Down => {
                    if board.cursor_x < board_size - 1 {
                        board.cursor_x += 1;
                    }
                }
                crossterm::event::KeyCode::Left => {
                    if board.cursor_y > 0 {
                        board.cursor_y -= 1;
                    }
                }
                crossterm::event::KeyCode::Right => {
                    if board.cursor_y < board_size - 1 {
                        board.cursor_y += 1;
                    }
                }
                crossterm::event::KeyCode::Char(' ') => {
                    board.reveal(board.cursor_x, board.cursor_y);
                }
                crossterm::event::KeyCode::Char('f') => {
                    board.flag(board.cursor_x, board.cursor_y);
                }
                _ => {}
            },
            _ => {}
        }

        let mut all_cells_without_mines_revealed = true;
        for row in board.board.iter() {
            for cell in row.iter() {
                if !cell.contains_mine() {
                    match cell.state {
                        CellState::Unrevealed | CellState::Flagged => {
                            all_cells_without_mines_revealed = false;
                        }
                        CellState::Revealed(_) => {}
                    }
                }
            }
        }

        if all_cells_without_mines_revealed {
            board.set_game_over(true);
        }

        if board.game_over {
            execute!(stdout, terminal::Clear(terminal::ClearType::All)).unwrap();

            execute!(stdout, MoveTo(0, 0), Print("Game over!")).unwrap();

            break;
        }
    }

    crossterm::terminal::disable_raw_mode().unwrap();
}
