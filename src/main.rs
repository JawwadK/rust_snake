use ggez::audio::{self, SoundSource};
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
    score: u32,
    // Sound effects
    eat_sound: audio::Source,
    game_over_sound: audio::Source,
}

impl GameState {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut snake = Vec::new();
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

        // Load sound effects
        let eat_sound = audio::Source::new(ctx, "/eat.wav")?;
        let game_over_sound = audio::Source::new(ctx, "/game_over.wav")?;

        Ok(GameState {
            snake,
            direction: Direction::Right,
            food: GameState::generate_food_position(),
            game_over: false,
            movement_cooldown: 0.15,
            last_update: 0.0,
            score: 0,
            eat_sound,
            game_over_sound,
        })
    }

    fn generate_food_position() -> Position {
        let mut rng = rand::thread_rng();
        Position {
            x: rng.gen_range(0..GRID_SIZE),
            y: rng.gen_range(0..GRID_SIZE),
        }
    }

    fn check_collision(&self, pos: &Position) -> bool {
        self.snake
            .iter()
            .skip(1)
            .any(|p| p.x == pos.x && p.y == pos.y)
    }

    fn update_snake(&mut self, ctx: &mut Context) -> GameResult {
        if self.game_over {
            return Ok(());
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
            self.game_over_sound.play_detached(ctx)?;
            return Ok(());
        }

        // Check self collision
        if self.check_collision(&new_head) {
            self.game_over = true;
            self.game_over_sound.play_detached(ctx)?;
            return Ok(());
        }

        self.snake.insert(0, new_head);

        // Check if snake ate food
        if new_head.x == self.food.x && new_head.y == self.food.y {
            self.food = GameState::generate_food_position();
            self.score += 10;
            self.eat_sound.play_detached(ctx)?;
        } else {
            self.snake.pop();
        }

        Ok(())
    }
}

impl EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let current_time = ctx.time.time_since_start().as_secs_f32();

        if current_time - self.last_update >= self.movement_cooldown {
            self.update_snake(ctx)?;
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
            canvas.draw(
                &text,
                graphics::DrawParam::default().dest(Point2 {
                    x: (SCREEN_SIZE as f32 / 2.0) - 40.0,
                    y: (SCREEN_SIZE as f32 / 2.0) - 10.0,
                }),
            );
        }

        // Update Score Display
        let score_text = graphics::Text::new(format!("Score: {}", self.score));
        canvas.draw(
            &score_text,
            graphics::DrawParam::default()
                .dest(Point2 { x: 10.0, y: 10.0 })
                .color(graphics::Color::WHITE),
        );
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
    let resource_dir = std::path::PathBuf::from("./resources");
    let window_setup = ggez::conf::WindowSetup::default().title("Snake Game");
    let window_mode = ggez::conf::WindowMode::default()
        .dimensions(SCREEN_SIZE as f32, SCREEN_SIZE as f32);
    
    let (mut ctx, event_loop) = ggez::ContextBuilder::new("snake", "author")
        .add_resource_path(resource_dir)
        .window_setup(window_setup)
        .window_mode(window_mode)
        .build()?;

    let state = GameState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}