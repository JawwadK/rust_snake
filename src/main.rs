use ggez::event::{self, EventHandler};
use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::mint::Point2;
use ggez::{graphics, Context, GameResult};
use rand::Rng;

const GRID_SIZE: i16 = 30;
const GRID_CELL_SIZE: i16 = 20;
const SCREEN_SIZE: i16 = GRID_SIZE * GRID_CELL_SIZE;

#[derive(Clone, Copy, PartialEq)]
struct Position {
    x: i16,
    y: i16,
}

#[derive(PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct GameState {
    snake: Vec<Position>,
    direction: Direction,
    food: Position,
    game_over: bool,
    movement_cooldown: f32,
    last_update: f32,
}

impl GameState {
    pub fn new() -> Self {
        let mut snake = Vec::new();
        // Start with a snake of length 3
        snake.push(Position {
            x: GRID_SIZE / 2,
            y: GRID_SIZE / 2,
        });
        snake.push(Position {
            x: GRID_SIZE / 2 - 1,
            y: GRID_SIZE / 2,
        });
        snake.push(Position {
            x: GRID_SIZE / 2 - 2,
            y: GRID_SIZE / 2,
        });

        GameState {
            snake,
            direction: Direction::Right,
            food: GameState::generate_food_position(),
            game_over: false,
            movement_cooldown: 0.15, // Adjust this to control game speed
            last_update: 0.0,
        }
    }

    fn generate_food_position() -> Position {
        let mut rng = rand::thread_rng();
        Position {
            x: rng.gen_range(0..GRID_SIZE),
            y: rng.gen_range(0..GRID_SIZE),
        }
    }

    fn check_collision(&self, pos: &Position) -> bool {
        // Check if snake hits itself
        self.snake
            .iter()
            .skip(1)
            .any(|p| p.x == pos.x && p.y == pos.y)
    }

    fn update_snake(&mut self) {
        if self.game_over {
            return;
        }

        let head = self.snake.first().unwrap().clone();
        let new_head = match self.direction {
            Direction::Up => Position {
                x: head.x,
                y: head.y - 1,
            },
            Direction::Down => Position {
                x: head.x,
                y: head.y + 1,
            },
            Direction::Left => Position {
                x: head.x - 1,
                y: head.y,
            },
            Direction::Right => Position {
                x: head.x + 1,
                y: head.y,
            },
        };

        // Check wall collision
        if new_head.x < 0 || new_head.x >= GRID_SIZE || new_head.y < 0 || new_head.y >= GRID_SIZE {
            self.game_over = true;
            return;
        }

        // Check self collision
        if self.check_collision(&new_head) {
            self.game_over = true;
            return;
        }

        self.snake.insert(0, new_head);

        // Check if snake ate food
        if new_head.x == self.food.x && new_head.y == self.food.y {
            // Generate new food
            self.food = GameState::generate_food_position();
        } else {
            // Remove tail if no food was eaten
            self.snake.pop();
        }
    }
}

impl EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let current_time = ctx.time.time_since_start().as_secs_f32();

        if current_time - self.last_update >= self.movement_cooldown {
            self.update_snake();
            self.last_update = current_time;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::BLACK);

        // Draw snake
        for pos in &self.snake {
            let rect = graphics::Rect::new(
                (pos.x * GRID_CELL_SIZE) as f32,
                (pos.y * GRID_CELL_SIZE) as f32,
                GRID_CELL_SIZE as f32,
                GRID_CELL_SIZE as f32,
            );
            canvas.draw(
                &graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::fill(),
                    rect,
                    graphics::Color::GREEN,
                )?,
                graphics::DrawParam::default(),
            );
        }

        // Draw food
        let food_rect = graphics::Rect::new(
            (self.food.x * GRID_CELL_SIZE) as f32,
            (self.food.y * GRID_CELL_SIZE) as f32,
            GRID_CELL_SIZE as f32,
            GRID_CELL_SIZE as f32,
        );
        canvas.draw(
            &graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                food_rect,
                graphics::Color::RED,
            )?,
            graphics::DrawParam::default(),
        );

        if self.game_over {
            let text = graphics::Text::new("Game Over!");
            let dest = Point2 {
                x: (SCREEN_SIZE as f32 / 2.0) - 40.0,
                y: (SCREEN_SIZE as f32 / 2.0) - 10.0,
            };
            canvas.draw(
                &text,
                graphics::DrawParam::default().dest(Point2 {
                    x: (SCREEN_SIZE as f32 / 2.0) - 40.0,
                    y: (SCREEN_SIZE as f32 / 2.0) - 10.0,
                }),
            );
        }

        canvas.finish(ctx)?;
        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult {
        if let Some(keycode) = input.keycode {
            if self.game_over {
                return Ok(());
            }

            match keycode {
                KeyCode::Up if self.direction != Direction::Down => {
                    self.direction = Direction::Up;
                }
                KeyCode::Down if self.direction != Direction::Up => {
                    self.direction = Direction::Down;
                }
                KeyCode::Left if self.direction != Direction::Right => {
                    self.direction = Direction::Left;
                }
                KeyCode::Right if self.direction != Direction::Left => {
                    self.direction = Direction::Right;
                }
                _ => (),
            }
        }
        Ok(())
    }
}

fn main() -> GameResult {
    let window_setup = ggez::conf::WindowSetup::default().title("Snake Game");
    let window_mode =
        ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE as f32, SCREEN_SIZE as f32);
    let (ctx, event_loop) = ggez::ContextBuilder::new("snake", "author")
        .window_setup(window_setup)
        .window_mode(window_mode)
        .build()?;

    let state = GameState::new();
    event::run(ctx, event_loop, state)
}
