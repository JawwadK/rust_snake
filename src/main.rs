//Most up to date snake_game
use ggez::audio::{self, SoundSource};
use ggez::event::{self, EventHandler};
use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::mint::{Point2, Vector2};
use ggez::{graphics, Context, GameResult};
use rand::Rng;
use std::f32::consts::PI;
use serde::{Deserialize, Serialize};
use std::fs;
use chrono::{DateTime, Local};

const GRID_SIZE: i16 = 30;
const GRID_CELL_SIZE: i16 = 20;
const SCREEN_SIZE: i16 = GRID_SIZE * GRID_CELL_SIZE;
const SUBMENU_TRANSITION_TIME: f32 = 0.3;
const MAX_SCORES_PER_DIFFICULTY: usize = 5;

// Colors
const BACKGROUND_COLOR: graphics::Color = graphics::Color::new(0.1, 0.1, 0.15, 1.0);
const GRID_COLOR: graphics::Color = graphics::Color::new(0.15, 0.15, 0.2, 1.0);
const FOOD_COLORS: [graphics::Color; 5] = [
    graphics::Color::new(1.0, 0.0, 0.0, 1.0),  // Red
    graphics::Color::new(1.0, 0.2, 0.2, 1.0),  // Light red
    graphics::Color::new(1.0, 0.4, 0.4, 1.0),  // Lighter red
    graphics::Color::new(1.0, 0.6, 0.6, 1.0),  // Even lighter red
    graphics::Color::new(1.0, 0.8, 0.8, 1.0),  // Very light red
];

#[derive(Clone, Copy, PartialEq)]
struct Position {
    x: i16,
    y: i16,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum GameState {
    Menu,
    Playing,
    Paused,
    GameOver,
}

#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
enum Difficulty {
    Easy,
    Medium,
    Hard,
    Expert,
}

#[derive(Serialize, Deserialize, Clone)]
struct ScoreEntry {
    player_name: String,
    score: u32,
    difficulty: Difficulty,
    timestamp: DateTime<Local>,
}

#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
enum MenuState {
    Main,
    Difficulty,
    HighScores,
    EnteringName,
}

#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
struct DifficultyInfo {
    speed: f32,
    score_multiplier: f32,
}

impl Difficulty {
    fn get_info(&self) -> DifficultyInfo {
        match self {
            Difficulty::Easy => DifficultyInfo {
                speed: 0.2,
                score_multiplier: 1.0,
            },
            Difficulty::Medium => DifficultyInfo {
                speed: 0.15,
                score_multiplier: 1.5,
            },
            Difficulty::Hard => DifficultyInfo {
                speed: 0.1,
                score_multiplier: 2.0,
            },
            Difficulty::Expert => DifficultyInfo {
                speed: 0.07,
                score_multiplier: 3.0,
            },
        }
    }
}

struct Game {
    state: GameState,
    snake: Vec<Position>,
    direction: Direction,
    next_direction: Direction,
    food: Position,
    food_animation: f32,
    movement_cooldown: f32,
    initial_cooldown: f32,
    last_update: f32,
    score: u32,
    difficulty: Difficulty,
    high_score: u32,
    eat_sound: audio::Source,
    game_over_sound: audio::Source,
    menu_selection: usize,
    particle_effects: Vec<ParticleEffect>,
    menu_state: MenuState,
    high_scores: Vec<ScoreEntry>,
    submenu_transition: f32,
    player_name: String,
    name_input_active: bool,
}

struct ParticleEffect {
    position: Position,
    particles: Vec<Particle>,
    lifetime: f32,
}

struct Particle {
    pos: Point2<f32>,
    vel: Vector2<f32>,
    color: graphics::Color,
    size: f32,
    lifetime: f32,
}

impl ParticleEffect {
    fn new(position: Position) -> Self {
        let mut particles = Vec::new();
        let mut rng = rand::thread_rng();
        
        for _ in 0..20 {
            let angle = rng.gen_range(0.0..2.0 * PI);
            let speed = rng.gen_range(50.0..150.0);
            let size = rng.gen_range(2.0..5.0);
            
            particles.push(Particle {
                pos: Point2 {
                    x: (position.x * GRID_CELL_SIZE) as f32 + GRID_CELL_SIZE as f32 / 2.0,
                    y: (position.y * GRID_CELL_SIZE) as f32 + GRID_CELL_SIZE as f32 / 2.0,
                },
                vel: Vector2 {
                    x: angle.cos() * speed,
                    y: angle.sin() * speed,
                },
                color: graphics::Color::new(1.0, rng.gen_range(0.5..1.0), 0.0, 1.0),
                size,
                lifetime: 1.0,
            });
        }
        
        ParticleEffect {
            position,
            particles,
            lifetime: 1.0,
        }
    }

    fn update(&mut self, dt: f32) {
        self.lifetime -= dt;
        for particle in &mut self.particles {
            particle.pos.x += particle.vel.x * dt;
            particle.pos.y += particle.vel.y * dt;
            particle.lifetime -= dt;
            particle.color.a = particle.lifetime;
        }
    }
}

impl Game {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let eat_sound = audio::Source::new(ctx, "/eat.wav")?;
        let game_over_sound = audio::Source::new(ctx, "/game_over.wav")?;
        let high_scores = Self::load_high_scores().unwrap_or_default();

        Ok(Game {
            state: GameState::Menu,
            snake: Vec::new(),
            direction: Direction::Right,
            next_direction: Direction::Right,
            food: Position { x: 0, y: 0 },
            food_animation: 0.0,
            movement_cooldown: 0.15,
            initial_cooldown: 0.15,
            last_update: 0.0,
            score: 0,
            difficulty: Difficulty::Medium,
            high_score: 0,
            eat_sound,
            game_over_sound,
            menu_selection: 0,
            particle_effects: Vec::new(),
            menu_state: MenuState::Main,
            high_scores,
            submenu_transition: 0.0,
            player_name: String::new(),
            name_input_active: false,
        })
    }
    fn load_high_scores() -> std::io::Result<Vec<ScoreEntry>> {
        match fs::read_to_string("high_scores.json") {
            Ok(contents) => Ok(serde_json::from_str(&contents).unwrap_or_default()),
            Err(_) => Ok(Vec::new()),
        }
    }

    fn save_high_scores(&self) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(&self.high_scores)?;
        fs::write("high_scores.json", json)
    }

    fn add_high_score(&mut self, score: u32) {
        if self.player_name.is_empty() {
            self.menu_state = MenuState::EnteringName;
            self.name_input_active = true;
            return;
        }

        let entry = ScoreEntry {
            player_name: self.player_name.clone(),
            score,
            difficulty: self.difficulty,
            timestamp: Local::now(),
        };

        self.high_scores.push(entry);
        self.high_scores.sort_by(|a, b| b.score.cmp(&a.score));

        // Keep only top scores per difficulty
        let mut filtered_scores = Vec::new();
        for diff in [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard, Difficulty::Expert] {
            let mut count = 0;
            for score in self.high_scores.iter() {
                if score.difficulty == diff {
                    if count < MAX_SCORES_PER_DIFFICULTY {
                        filtered_scores.push(score.clone());
                        count += 1;
                    }
                }
            }
        }
        self.high_scores = filtered_scores;
        self.save_high_scores().unwrap_or_else(|e| eprintln!("Failed to save high scores: {}", e));
    }

fn draw_difficulty_menu(&self, _ctx: &mut Context, canvas: &mut graphics::Canvas) -> GameResult {
    let mut title_text = graphics::Text::new("Select Difficulty");
    let title = title_text.set_scale(40.0);
    canvas.draw(
        title,  // No & needed, set_scale returns &mut Text
        graphics::DrawParam::default()
            .dest(Point2 {
                x: (SCREEN_SIZE as f32 / 2.0) - 100.0,
                y: 50.0,
            })
            .color(graphics::Color::WHITE),
    );

    let difficulties = [
        (Difficulty::Easy, "Easy"),
        (Difficulty::Medium, "Medium"),
        (Difficulty::Hard, "Hard"),
        (Difficulty::Expert, "Expert"),
    ];

    for (i, (diff, name)) in difficulties.iter().enumerate() {
        let info = diff.get_info();
        let color = if *diff == self.difficulty {
            graphics::Color::GREEN
        } else {
            graphics::Color::WHITE
        };

        let mut diff_text = graphics::Text::new(format!(
            "{}: Speed {:.1}x, Score {:.1}x",
            name,
            1.0 / info.speed,
            info.score_multiplier
        ));
        let diff_text = diff_text.set_scale(24.0);
        
        canvas.draw(
            diff_text,  // No & needed, set_scale returns &mut Text
            graphics::DrawParam::default()
                .dest(Point2 {
                    x: (SCREEN_SIZE as f32 / 2.0) - 150.0,
                    y: 150.0 + (i as f32 * 50.0),
                })
                .color(color),
        );
    }

    let mut back_text = graphics::Text::new("Press ESC to return");
    let back_text = back_text.set_scale(20.0);
    canvas.draw(
        back_text,  // No & needed, set_scale returns &mut Text
        graphics::DrawParam::default()
            .dest(Point2 {
                x: (SCREEN_SIZE as f32 / 2.0) - 80.0,
                y: SCREEN_SIZE as f32 - 50.0,
            })
            .color(graphics::Color::YELLOW),
    );

    Ok(())
}
fn draw_high_scores(&self, _ctx: &mut Context, canvas: &mut graphics::Canvas) -> GameResult {
    let mut title_text = graphics::Text::new("High Scores");
    let title = title_text.set_scale(40.0);
    canvas.draw(
        title,  // No need for & as set_scale returns &mut Text
        graphics::DrawParam::default()
            .dest(Point2 {
                x: (SCREEN_SIZE as f32 / 2.0) - 100.0,
                y: 50.0,
            })
            .color(graphics::Color::WHITE),
    );

    let difficulties = [
        (Difficulty::Easy, "Easy"),
        (Difficulty::Medium, "Medium"),
        (Difficulty::Hard, "Hard"),
        (Difficulty::Expert, "Expert"),
    ];

    for (i, (diff, name)) in difficulties.iter().enumerate() {
        let diff_scores: Vec<_> = self.high_scores.iter()
            .filter(|score| score.difficulty == *diff)
            .take(MAX_SCORES_PER_DIFFICULTY)
            .collect();

        let mut header_text = graphics::Text::new(format!("--- {} ---", name));
        let header = header_text.set_scale(24.0);
        canvas.draw(
            header,  // No need for & as set_scale returns &mut Text
            graphics::DrawParam::default()
                .dest(Point2 {
                    x: 50.0,
                    y: 120.0 + (i as f32 * 120.0),
                })
                .color(graphics::Color::YELLOW),
        );

        for (j, score) in diff_scores.iter().enumerate() {
            let mut score_text = graphics::Text::new(format!(
                "{:2}. {:8} {:6} {}",
                j + 1,
                score.player_name,
                score.score,
                score.timestamp.format("%Y-%m-%d %H:%M"),
            ));
            let score_text = score_text.set_scale(20.0);
            canvas.draw(
                score_text,  // No need for & as set_scale returns &mut Text
                graphics::DrawParam::default()
                    .dest(Point2 {
                        x: 50.0,
                        y: 150.0 + (i as f32 * 120.0) + (j as f32 * 25.0),
                    })
                    .color(graphics::Color::WHITE),
            );
        }
    }

    let mut back_text = graphics::Text::new("Press ESC to return");
    let back_text = back_text.set_scale(20.0);
    canvas.draw(
        back_text,  // No need for & as set_scale returns &mut Text
        graphics::DrawParam::default()
            .dest(Point2 {
                x: (SCREEN_SIZE as f32 / 2.0) - 80.0,
                y: SCREEN_SIZE as f32 - 50.0,
            })
            .color(graphics::Color::YELLOW),
    );

    Ok(())
}





    fn reset(&mut self) {
        self.snake.clear();
        // Initialize snake at the center
        for i in 0..3 {
            self.snake.push(Position {
                x: GRID_SIZE / 2 - i as i16,
                y: GRID_SIZE / 2,
            });
        }
        self.spawn_food();
        self.direction = Direction::Right;
        self.next_direction = Direction::Right;
        self.score = 0;
        self.movement_cooldown = self.initial_cooldown;
        self.particle_effects.clear();
    }

    fn spawn_food(&mut self) {
        let mut rng = rand::thread_rng();
        loop {
            let pos = Position {
                x: rng.gen_range(0..GRID_SIZE),
                y: rng.gen_range(0..GRID_SIZE),
            };
            if !self.snake.contains(&pos) {
                self.food = pos;
                break;
            }
        }
    }

fn draw_menu(&mut self, _ctx: &mut Context, canvas: &mut graphics::Canvas) -> GameResult {
        // Create mutable Text objects
        let mut title_text = graphics::Text::new("SNAKE GAME");
        let title = title_text.set_scale(48.0);
        
        let menu_items = [
            "Play Game",
            "Difficulty",
            "High Scores",
            "Exit",
        ];

        // Draw title
        canvas.draw(
            title,
            graphics::DrawParam::default()
                .dest(Point2 {
                    x: (SCREEN_SIZE as f32 / 2.0) - 100.0,
                    y: 100.0,
                })
                .color(graphics::Color::WHITE),
        );

        // Draw menu items
        for (i, item) in menu_items.iter().enumerate() {
            let color = if i == self.menu_selection {
                graphics::Color::GREEN
            } else {
                graphics::Color::WHITE
            };

            let mut menu_text = graphics::Text::new(*item);
            let text = menu_text.set_scale(32.0);

            canvas.draw(
                text,
                graphics::DrawParam::default()
                    .dest(Point2 {
                        x: (SCREEN_SIZE as f32 / 2.0) - 50.0,
                        y: 250.0 + (i as f32 * 50.0),
                    })
                    .color(color),
            );
        }

        Ok(())
    }
    fn draw_game(&mut self, ctx: &mut Context, canvas: &mut graphics::Canvas) -> GameResult {
        // Draw grid
        for i in 0..GRID_SIZE {
            for j in 0..GRID_SIZE {
                if (i + j) % 2 == 0 {
                    let rect = graphics::Rect::new(
                        (i * GRID_CELL_SIZE) as f32,
                        (j * GRID_CELL_SIZE) as f32,
                        GRID_CELL_SIZE as f32,
                        GRID_CELL_SIZE as f32,
                    );
                    canvas.draw(
                        &graphics::Mesh::new_rectangle(
                            ctx,
                            graphics::DrawMode::fill(),
                            rect,
                            GRID_COLOR,
                        )?,
                        graphics::DrawParam::default(),
                    );
                }
            }
        }

        // Draw snake with gradient effect
        for (i, pos) in self.snake.iter().enumerate() {
            let progress = i as f32 / self.snake.len() as f32;
            let color = graphics::Color::new(
                0.0,
                0.8 + progress * 0.2,
                0.0,
                1.0,
            );

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
                    color,
                )?,
                graphics::DrawParam::default(),
            );
        }

        // Draw animated food
        let food_scale = 1.0 + (self.food_animation * PI).sin() * 0.2;
        let food_color_index = ((self.food_animation * 5.0) as usize) % FOOD_COLORS.len();
        let food_size = GRID_CELL_SIZE as f32 * food_scale;
        let food_offset = (GRID_CELL_SIZE as f32 - food_size) / 2.0;

        let food_rect = graphics::Rect::new(
            (self.food.x * GRID_CELL_SIZE) as f32 + food_offset,
            (self.food.y * GRID_CELL_SIZE) as f32 + food_offset,
            food_size,
            food_size,
        );
        canvas.draw(
            &graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                food_rect,
                FOOD_COLORS[food_color_index],
            )?,
            graphics::DrawParam::default(),
        );

        // Draw particle effects
        for effect in &self.particle_effects {
            for particle in &effect.particles {
                let rect = graphics::Rect::new(
                    particle.pos.x - particle.size / 2.0,
                    particle.pos.y - particle.size / 2.0,
                    particle.size,
                    particle.size,
                );
                canvas.draw(
                    &graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        rect,
                        particle.color,
                    )?,
                    graphics::DrawParam::default(),
                );
            }
        }

        // Draw UI
        let score_text = graphics::Text::new(format!(
            "Score: {} | High Score: {} | Speed: {:.2} | {:?}",
            self.score,
            self.high_score,
            1.0 / self.movement_cooldown,
            self.difficulty,
        ));
        canvas.draw(
            &score_text,
            graphics::DrawParam::default()
                .dest(Point2 { x: 10.0, y: 10.0 })
                .color(graphics::Color::WHITE),
        );

        Ok(())
    }

    fn update_game(&mut self, ctx: &mut Context, dt: f32) -> GameResult {
        self.food_animation = (self.food_animation + dt) % (2.0 * PI);
        
        // Update particle effects
        self.particle_effects.retain_mut(|effect| {
            effect.update(dt);
            effect.lifetime > 0.0
        });

        // Update snake movement
        let current_time = ctx.time.time_since_start().as_secs_f32();
        if current_time - self.last_update >= self.movement_cooldown {
            self.last_update = current_time;
            self.direction = self.next_direction;

            let head = self.snake.first().unwrap().clone();
            let new_head = match self.direction {
                Direction::Up => Position { x: head.x, y: head.y - 1 },
                Direction::Down => Position { x: head.x, y: head.y + 1 },
                Direction::Left => Position { x: head.x - 1, y: head.y },
                Direction::Right => Position { x: head.x + 1, y: head.y },
            };

            // Check collisions
            if new_head.x < 0 || new_head.x >= GRID_SIZE || new_head.y < 0 || new_head.y >= GRID_SIZE 
                || self.snake.contains(&new_head) {
                self.state = GameState::GameOver;
                self.high_score = self.high_score.max(self.score);
                self.game_over_sound.play_detached(ctx)?;
                return Ok(());
            }

            // Move snake
            self.snake.insert(0, new_head);

            // Check food collision
            if new_head == self.food {
                self.score += 10;
                self.eat_sound.play_detached(ctx)?;
                self.particle_effects.push(ParticleEffect::new(self.food));
                self.spawn_food();
                // Speed up
                self.movement_cooldown = (self.movement_cooldown * 0.95).max(0.05);
            } else {
                self.snake.pop();
            }
        }

        Ok(())
    }
}

impl EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let dt = ctx.time.delta().as_secs_f32();
        
        match self.state {
            GameState::Playing => self.update_game(ctx, dt)?,
            GameState::Menu => {
                // Update menu transitions if needed
                self.submenu_transition = (self.submenu_transition + dt).min(SUBMENU_TRANSITION_TIME);
            }
            _ => (),
        }

        Ok(())
    }

fn draw(&mut self, ctx: &mut Context) -> GameResult {
    let mut canvas = graphics::Canvas::from_frame(ctx, BACKGROUND_COLOR);

    match self.state {
        GameState::Menu => {
            match self.menu_state {
                MenuState::Main => self.draw_menu(ctx, &mut canvas)?,
                MenuState::Difficulty => self.draw_difficulty_menu(ctx, &mut canvas)?,
                MenuState::HighScores => self.draw_high_scores(ctx, &mut canvas)?,
                MenuState::EnteringName => {
                    let prompt_text = format!("Enter your name: {}_", self.player_name);
                    let mut name_prompt = graphics::Text::new(prompt_text);
                    // Store reference from set_scale
                    let name_prompt = name_prompt.set_scale(32.0);

                    canvas.draw(
                        name_prompt,  // Already a reference
                        graphics::DrawParam::default()
                            .dest(Point2 {
                                x: (SCREEN_SIZE as f32 / 2.0) - 150.0,
                                y: (SCREEN_SIZE as f32 / 2.0),
                            })
                            .color(graphics::Color::WHITE),
                    );
                }
            }
        }
        GameState::Playing | GameState::Paused => self.draw_game(ctx, &mut canvas)?,
        GameState::GameOver => {
            self.draw_game(ctx, &mut canvas)?;
            
            let game_over_string = format!(
                "Game Over!\nScore: {}\nPress R to restart\nPress M for menu",
                self.score
            );
            let mut game_over_text = graphics::Text::new(game_over_string);
            // Store reference from set_scale
            let game_over_text = game_over_text.set_scale(32.0);

            canvas.draw(
                game_over_text,  // Already a reference
                graphics::DrawParam::default()
                    .dest(Point2 {
                        x: (SCREEN_SIZE as f32 / 2.0) - 100.0,
                        y: (SCREEN_SIZE as f32 / 2.0) - 60.0,
                    })
                    .color(graphics::Color::WHITE),
            );
        }
    }

    canvas.finish(ctx)?;
    Ok(())
}

    fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult {
        if let Some(keycode) = input.keycode {
            match self.state {
                GameState::Menu => {
                    match self.menu_state {
                        MenuState::Main => {
                            match keycode {
                                KeyCode::Up => {
                                    self.menu_selection = self.menu_selection.checked_sub(1).unwrap_or(3);
                                }
                                KeyCode::Down => {
                                    self.menu_selection = (self.menu_selection + 1) % 4;
                                }
                                KeyCode::Return => {
                                    match self.menu_selection {
                                        0 => {
                                            self.reset();
                                            self.state = GameState::Playing;
                                        }
                                        1 => self.menu_state = MenuState::Difficulty,
                                        2 => self.menu_state = MenuState::HighScores,
                                        3 => std::process::exit(0),
                                        _ => {}
                                    }
                                }
                                _ => {}
                            }
                        }
                        MenuState::Difficulty => {
                            match keycode {
                                KeyCode::Up => {
                                    self.difficulty = match self.difficulty {
                                        Difficulty::Easy => Difficulty::Expert,
                                        Difficulty::Medium => Difficulty::Easy,
                                        Difficulty::Hard => Difficulty::Medium,
                                        Difficulty::Expert => Difficulty::Hard,
                                    };
                                    self.initial_cooldown = self.difficulty.get_info().speed;
                                }
                                KeyCode::Down => {
                                    self.difficulty = match self.difficulty {
                                        Difficulty::Easy => Difficulty::Medium,
                                        Difficulty::Medium => Difficulty::Hard,
                                        Difficulty::Hard => Difficulty::Expert,
                                        Difficulty::Expert => Difficulty::Easy,
                                    };
                                    self.initial_cooldown = self.difficulty.get_info().speed;
                                }
                                KeyCode::Escape => self.menu_state = MenuState::Main,
                                _ => {}
                            }
                        }
                        MenuState::HighScores => {
                            if keycode == KeyCode::Escape {
                                self.menu_state = MenuState::Main;
                            }
                        }
                        MenuState::EnteringName => {
                            match keycode {
                                KeyCode::Return => {
                                    if !self.player_name.is_empty() {
                                        self.add_high_score(self.score);
                                        self.menu_state = MenuState::HighScores;
                                        self.name_input_active = false;
                                    }
                                }
                                KeyCode::Back => {
                                    self.player_name.pop();
                                }
                                _ => {}
                            }
                        }
                    }
                }
                GameState::Playing => {
                    match keycode {
                        KeyCode::Up if self.direction != Direction::Down => {
                            self.next_direction = Direction::Up;
                        }
                        KeyCode::Down if self.direction != Direction::Up => {
                            self.next_direction = Direction::Down;
                        }
                        KeyCode::Left if self.direction != Direction::Right => {
                            self.next_direction = Direction::Left;
                        }
                        KeyCode::Right if self.direction != Direction::Left => {
                            self.next_direction = Direction::Right;
                        }
                        KeyCode::Escape => {
                            self.state = GameState::Paused;
                        }
                        _ => {}
                    }
                }
                GameState::Paused => {
                    match keycode {
                        KeyCode::Escape => {
                            self.state = GameState::Playing;
                        }
                        KeyCode::M => {
                            self.state = GameState::Menu;
                        }
                        _ => {}
                    }
                }
                GameState::GameOver => {
                    match keycode {
                        KeyCode::R => {
                            if !self.name_input_active {
                                self.add_high_score(self.score);
                            } else {
                                self.reset();
                                self.state = GameState::Playing;
                            }
                        }
                        KeyCode::M => {
                            self.state = GameState::Menu;
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn text_input_event(&mut self, _ctx: &mut Context, character: char) -> GameResult {
        if self.name_input_active && self.player_name.len() < 8 && character.is_alphanumeric() {
            self.player_name.push(character);
        }
        Ok(())
    }
}
// Add Direction enum that was missing
#[derive(PartialEq, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

fn main() -> GameResult {
    let resource_dir = std::path::PathBuf::from("./resources");
    let window_setup = ggez::conf::WindowSetup::default()
        .title("Snake Game")
        .vsync(true);
    let window_mode = ggez::conf::WindowMode::default()
        .dimensions(SCREEN_SIZE as f32, SCREEN_SIZE as f32)
        .resizable(false);
    
    let (mut ctx, event_loop) = ggez::ContextBuilder::new("snake", "author")
        .add_resource_path(resource_dir)
        .window_setup(window_setup)
        .window_mode(window_mode)
        .build()?;

    let game = Game::new(&mut ctx)?;
    event::run(ctx, event_loop, game)
}