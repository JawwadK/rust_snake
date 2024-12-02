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

#[derive(PartialEq, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum Difficulty {
    Easy,
    Medium,
    Hard,
    Expert,
}

impl Difficulty {
    fn get_initial_speed(&self) -> f32 {
        match self {
            Difficulty::Easy => 0.2,    // Slower speed
            Difficulty::Medium => 0.15,  // Default speed
            Difficulty::Hard => 0.1,     // Faster speed
            Difficulty::Expert => 0.07,  // Very fast speed
        }
    }

    fn get_speed_increase_rate(&self) -> f32 {
        match self {
            Difficulty::Easy => 0.002,
            Difficulty::Medium => 0.003,
            Difficulty::Hard => 0.004,
            Difficulty::Expert => 0.005,
        }
    }

    fn get_score_multiplier(&self) -> u32 {
        match self {
            Difficulty::Easy => 1,
            Difficulty::Medium => 2,
            Difficulty::Hard => 3,
            Difficulty::Expert => 4,
        }
    }
}

struct GameState {
    snake: Vec<Position>,
    direction: Direction,
    food: Position,
    game_over: bool,
    movement_cooldown: f32,
    initial_cooldown: f32,
    last_update: f32,
    score: u32,
    difficulty: Difficulty,
    high_score: u32,
    eat_sound: audio::Source,
    game_over_sound: audio::Source,
}

impl GameState {
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

    pub fn new(ctx: &mut Context, difficulty: Difficulty) -> GameResult<Self> {
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

        let initial_cooldown = difficulty.get_initial_speed();

        // Load sound effects
        let eat_sound = audio::Source::new(ctx, "/eat.wav")?;
        let game_over_sound = audio::Source::new(ctx, "/game_over.wav")?;

        Ok(GameState {
            snake,
            direction: Direction::Right,
            food: GameState::generate_food_position(),
            game_over: false,
            movement_cooldown: initial_cooldown,
            initial_cooldown,
            last_update: 0.0,
            score: 0,
            difficulty,
            high_score: 0,
            eat_sound,
            game_over_sound,
        })
    }

    fn reset(&mut self) {
        self.snake.clear();
        self.snake.push(Position {
            x: GRID_SIZE / 2,
            y: GRID_SIZE / 2,
        });
        self.snake.push(Position {
            x: GRID_SIZE / 2 - 1,
            y: GRID_SIZE / 2,
        });
        self.snake.push(Position {
            x: GRID_SIZE / 2 - 2,
            y: GRID_SIZE / 2,
        });
        
        self.direction = Direction::Right;
        self.food = GameState::generate_food_position();
        self.game_over = false;
        self.movement_cooldown = self.initial_cooldown;
        self.score = 0;
    }

    fn update_speed(&mut self) {
        // Calculate new speed based on score
        let speed_increase = self.score as f32 * self.difficulty.get_speed_increase_rate();
        self.movement_cooldown = (self.initial_cooldown - speed_increase).max(0.05);
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
            self.high_score = self.high_score.max(self.score);
            self.game_over_sound.play_detached(ctx)?;
            return Ok(());
        }

        // Check self collision
        if self.check_collision(&new_head) {
            self.game_over = true;
            self.high_score = self.high_score.max(self.score);
            self.game_over_sound.play_detached(ctx)?;
            return Ok(());
        }

        self.snake.insert(0, new_head);

        // Check if snake ate food
        if new_head.x == self.food.x && new_head.y == self.food.y {
            self.food = GameState::generate_food_position();
            self.score += 10 * self.difficulty.get_score_multiplier();
            self.update_speed();
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

        // Draw score and difficulty
        let score_text = graphics::Text::new(format!(
            "Score: {} | High Score: {} | Speed: {:.2} | Difficulty: {:?}",
            self.score, 
            self.high_score,
            1.0 / self.movement_cooldown,
            self.difficulty
        ));
        canvas.draw(
            &score_text,
            graphics::DrawParam::default()
                .dest(Point2 { x: 10.0, y: 10.0 })
                .color(graphics::Color::WHITE),
        );

        if self.game_over {
            let game_over_text = graphics::Text::new(
                format!(
                    "Game Over!\nScore: {}\nPress R to restart\nPress 1-4 to change difficulty",
                    self.score
                )
            );
            canvas.draw(
                &game_over_text,
                graphics::DrawParam::default().dest(Point2 {
                    x: (SCREEN_SIZE as f32 / 2.0) - 100.0,
                    y: (SCREEN_SIZE as f32 / 2.0) - 40.0,
                }),
            );
        }

        canvas.finish(ctx)?;
        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult {
        if let Some(keycode) = input.keycode {
            match keycode {
                // Game controls
                KeyCode::Up if !self.game_over && self.direction != Direction::Down => {
                    self.direction = Direction::Up;
                }
                KeyCode::Down if !self.game_over && self.direction != Direction::Up => {
                    self.direction = Direction::Down;
                }
                KeyCode::Left if !self.game_over && self.direction != Direction::Right => {
                    self.direction = Direction::Left;
                }
                KeyCode::Right if !self.game_over && self.direction != Direction::Left => {
                    self.direction = Direction::Right;
                }
                
                // Restart game
                KeyCode::R if self.game_over => {
                    self.reset();
                }

                // Difficulty selection
                KeyCode::Key1 if self.game_over => {
                    self.difficulty = Difficulty::Easy;
                    self.initial_cooldown = self.difficulty.get_initial_speed();
                    self.reset();
                }
                KeyCode::Key2 if self.game_over => {
                    self.difficulty = Difficulty::Medium;
                    self.initial_cooldown = self.difficulty.get_initial_speed();
                    self.reset();
                }
                KeyCode::Key3 if self.game_over => {
                    self.difficulty = Difficulty::Hard;
                    self.initial_cooldown = self.difficulty.get_initial_speed();
                    self.reset();
                }
                KeyCode::Key4 if self.game_over => {
                    self.difficulty = Difficulty::Expert;
                    self.initial_cooldown = self.difficulty.get_initial_speed();
                    self.reset();
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

    let state = GameState::new(&mut ctx, Difficulty::Medium)?;
    event::run(ctx, event_loop, state)
}