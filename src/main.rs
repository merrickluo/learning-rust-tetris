mod blocks;

extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate chrono;
extern crate timer;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL, };
use blocks::Block;
use blocks::Piece;

const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 20;

#[derive(Clone, Copy)]
struct Board {
    filled: [[i8; BOARD_HEIGHT]; BOARD_WIDTH],
    c_block: Block,
    c_position: (i32, i32),
}

impl Board {
    fn new() -> Board {
        Board {
            filled: [[0; BOARD_HEIGHT as usize]; BOARD_WIDTH as usize],
            c_block: Block::random_new(),
            c_position: (0, 0),
        }
    }


    fn fill(&mut self) {
        let piece = self.c_block.value();
        let (cx, cy) = self.c_position;
        let mut f = self.filled;
        for row in 0..5 {
            for col in 0..5 {
                if piece[row][col] > 0 {
                    let x = cx + row as i32;
                    let y = cy + col as i32;
                    f[x as usize][y as usize] = 1;
                }
            }
        }
        self.filled = f;
        self.check_line();
        self.new_block();
    }

    fn new_block(&mut self) {
        self.c_block = Block::random_new();
        self.c_position = (0, 0);
    }

    fn check_line(&mut self) {
        let filled = self.filled;

        for j in 0..BOARD_HEIGHT {
            let mut remove = true;
            for i in 0..BOARD_WIDTH {
                if filled[i][j] == 0 {
                    remove = false;
                }
            }
            if remove {
                self.delete_line(j);
            }
        }
    }

    fn delete_line(&mut self, line: usize) {
        let mut f = self.filled;
        for j in (1..line+1).rev() {
            for i in 0..BOARD_WIDTH {
                f[i][j] = f[i][j - 1];
            }
        }
        self.filled = f;
    }

    fn is_game_over(&self) -> bool {
        let filled = self.filled;
        for i in 0..BOARD_WIDTH {
            if filled[i][0] != 0 {
                return true;
            }
        }

        return false;
    }

    fn can_rotate(&self) -> bool {
        let mut blk = self.c_block.clone();
        let (x, y) = self.c_position;
        blk.rotate();

        return self.is_valid_move_b(x, y, &blk.value());
    }

    fn is_valid_move_b(&self, x: i32, y: i32, piece: &Piece) -> bool {
        for (i, row) in (x..x+5).enumerate() {
            for (j, col) in (y..y+5).enumerate() {
                if piece[i][j] != 0 {
                    if !self.is_block_valid(row, col) {
                        return false;
                    }
                    if !self.is_block_free(row as usize, col as usize) {
                        return false;
                    }
                }
            }
        }

        true
    }

    fn is_valid_move(&self, x: i32, y: i32) -> bool {
        let piece = &self.c_block.value();
        return self.is_valid_move_b(x, y, &piece);
    }

    fn is_block_valid(&self, row: i32, col: i32) -> bool {
        return row < BOARD_WIDTH as i32 && row >= 0
            && col < BOARD_HEIGHT as i32 && col >= 0;
    }

    fn is_block_free(&self, row: usize, col: usize) -> bool {
        return self.filled[row][col] == 0;
    }
}


pub struct TetrisApp {
    gl: GlGraphics,
    board: Board,

    timer: f64,
    game_over: bool,
}

const BLACK: [f32; 4] = [0.2, 0.2, 0.2, 0.0];
const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

impl TetrisApp {

    fn new(gl: GlGraphics) -> TetrisApp {
        TetrisApp {
            gl: gl,
            board: Board::new(),
            timer:0.0,
            game_over: false,
        }
    }

    fn keypress(&mut self, key: &Key) {
        let (mut cx, mut cy) = self.board.c_position;
        match *key {
            Key::Left => {
                cx -= 1
            }
            Key::Right => {
                cx += 1
            }
            Key::Up => {
                if self.board.can_rotate() {
                    self.board.c_block.rotate();
                    return
                }
            }
            Key::Down => {
                cy += 1
            }
            _ => {}
        }
        self.update_position(cx, cy, false);
    }

    fn update_position(&mut self, x: i32, y: i32, stop: bool) {
        if self.board.is_valid_move(x, y) {
            self.board.c_position = (x, y);
        } else if stop {
            self.board.fill();
            self.game_over = self.board.is_game_over();
        }
    }

    fn update(&mut self, args: &UpdateArgs) {
        self.timer += args.dt;
        if self.timer > 0.5 {
            let (cx, cy) = self.board.c_position;
            self.update_position(cx, cy + 1, true);
            self.timer = 0.0;
        }
    }

    fn render(&mut self, args: &RenderArgs) {
        if self.game_over {
            return;
        }

        use graphics::*;
        let board = &self.board;

        let block_size = args.width as f64 / BOARD_WIDTH as f64;
        let square = rectangle::square(0.0, 0.0, block_size);

        self.gl.draw(args.viewport(), |c, gl| {
            clear(BLACK, gl);

            // draw board
            for row in 0..BOARD_WIDTH {
                for col in 0..BOARD_HEIGHT {
                    if board.filled[row as usize][col as usize] == 1 {
                        let (x, y) = (row as f64 * block_size, col as f64 * block_size);
                        let transfrom = c.transform.trans(x, y);
                        rectangle(RED, square, transfrom, gl);
                    }
                }
            }

            let v = board.c_block.value();
            let (px, py) = board.c_position;
            for row in 0..5 {
                for col in 0..5 {
                    if v[row][col] > 0 {
                        let (x, y) = ((row as i32 + px) as f64 * block_size,
                                      (col as i32 + py) as f64 * block_size);
                        let transfrom = c.transform.trans(x, y);
                        rectangle(GREEN, square, transfrom, gl);
                    }
                }
            }
        });
    }
}
fn main() {
    let opengl = OpenGL::V3_2;

    let mut window: Window = WindowSettings::new(
            "playing-rust",
            [400, 800]
        )
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut app = TetrisApp::new(GlGraphics::new(opengl));

    let mut events = window.events();
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            app.render(&r);
        }

        if let Some(u) = e.update_args() {
            app.update(&u);
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            app.keypress(&key);
        }
    }
}














