use ggez::{
    event,
    graphics::{self, Color, DrawMode, DrawParam, Mesh, MeshBuilder, LineCap, Drawable},
    input::keyboard::{KeyCode, KeyInput},
    input::mouse::MouseButton,
    Context, GameResult,
};
use glam::Vec2;

const GRID_SIZE: i32 = 20;
const CELL_SIZE: f32 = 30.0;
const PACMAN_SPEED: f32 = 5.0;
const SCREEN_WIDTH: f32 = 800.0;
const SCREEN_HEIGHT: f32 = 600.0;
const MOUTH_SPEED: f32 = 0.2;
const MAX_MOUTH_ANGLE: f32 = 1.0; // Increased from 0.7 to 1.0 (about 57 degrees)

#[derive(Copy, Clone, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn to_vec2(&self) -> Vec2 {
        match self {
            Direction::Up => Vec2::new(0.0, -1.0),
            Direction::Down => Vec2::new(0.0, 1.0),
            Direction::Left => Vec2::new(-1.0, 0.0),
            Direction::Right => Vec2::new(1.0, 0.0),
        }
    }
}

struct DirectionController {
    queued_direction: Option<Direction>,
    current_direction: Option<Direction>,
}

impl DirectionController {
    fn new() -> Self {
        Self {
            queued_direction: None,
            current_direction: None,
        }
    }

    fn queue_direction(&mut self, new_direction: Direction) {
        self.queued_direction = Some(new_direction);
    }

    fn update(&mut self, position: Vec2) -> Option<Direction> {
        if self.is_aligned_with_grid(position) {
            if let Some(queued) = self.queued_direction {
                self.current_direction = Some(queued);
                self.queued_direction = None;
            }
        }
        self.current_direction
    }

    fn is_aligned_with_grid(&self, position: Vec2) -> bool {
        let cell_x = position.x / CELL_SIZE;
        let cell_y = position.y / CELL_SIZE;
        
        (cell_x.fract() < 0.1 || cell_x.fract() > 0.9) && 
        (cell_y.fract() < 0.1 || cell_y.fract() > 0.9)
    }
}

struct GameObject {
    pos: Vec2,
    direction: Vec2,
    size: f32,
    mouth_angle: f32,
    mouth_opening: bool,
}

struct GameState {
    pacman: GameObject,
    dots: Vec<Vec2>,
    score: i32,
    direction_controller: DirectionController,
    game_won: bool,
}

impl GameState {
    fn new() -> Self {
        // Create initial dots in a grid pattern
        let mut dots = Vec::new();
        for x in 1..GRID_SIZE-1 {
            for y in 1..GRID_SIZE-1 {
                dots.push(Vec2::new(
                    x as f32 * CELL_SIZE,
                    y as f32 * CELL_SIZE,
                ));
            }
        }

        GameState {
            pacman: GameObject {
                pos: Vec2::new(CELL_SIZE, GRID_SIZE as f32 * CELL_SIZE / 2.0),
                direction: Vec2::new(0.0, 0.0),
                size: CELL_SIZE * 0.8,
                mouth_angle: 0.0,
                mouth_opening: true,
            },
            dots,
            score: 0,
            direction_controller: DirectionController::new(),
            game_won: false,
        }
    }

    fn reset(&mut self) {
        // Reset dots
        self.dots.clear();
        for x in 1..GRID_SIZE-1 {
            for y in 1..GRID_SIZE-1 {
                self.dots.push(Vec2::new(
                    x as f32 * CELL_SIZE,
                    y as f32 * CELL_SIZE,
                ));
            }
        }

        // Reset pacman
        self.pacman.pos = Vec2::new(CELL_SIZE, GRID_SIZE as f32 * CELL_SIZE / 2.0);
        self.pacman.direction = Vec2::new(0.0, 0.0);
        
        // Reset score and game state
        self.score = 0;
        self.game_won = false;
        self.direction_controller = DirectionController::new();
        self.pacman.mouth_angle = 0.0;
        self.pacman.mouth_opening = true;
    }

    fn update(&mut self) {
        if self.game_won {
            return;  // Don't update game if won
        }

        // Update direction based on grid alignment
        if let Some(direction) = self.direction_controller.update(self.pacman.pos) {
            self.pacman.direction = direction.to_vec2();
        }

        // Update pacman position
        self.pacman.pos += self.pacman.direction * PACMAN_SPEED;
        
        // Keep pacman within bounds
        self.pacman.pos.x = self.pacman.pos.x.clamp(
            CELL_SIZE,
            CELL_SIZE * (GRID_SIZE - 1) as f32,
        );
        self.pacman.pos.y = self.pacman.pos.y.clamp(
            CELL_SIZE,
            CELL_SIZE * (GRID_SIZE - 1) as f32,
        );

        // Collect dots
        self.dots.retain(|dot| {
            let distance = (*dot - self.pacman.pos).length();
            if distance < CELL_SIZE * 0.5 {
                self.score += 10;
                false
            } else {
                true
            }
        });

        // Check for victory condition
        if self.dots.is_empty() {
            self.game_won = true;
        }

        // Update mouth animation
        if self.pacman.direction.length() > 0.0 {
            if self.pacman.mouth_opening {
                self.pacman.mouth_angle += MOUTH_SPEED;
                if self.pacman.mouth_angle >= MAX_MOUTH_ANGLE {
                    self.pacman.mouth_opening = false;
                }
            } else {
                self.pacman.mouth_angle -= MOUTH_SPEED;
                if self.pacman.mouth_angle <= 0.0 {
                    self.pacman.mouth_opening = true;
                }
            }
        } else {
            // Reset mouth when not moving
            self.pacman.mouth_angle = 0.0;
            self.pacman.mouth_opening = true;
        }
    }

    fn draw_grid(&self, ctx: &mut Context, canvas: &mut graphics::Canvas) -> GameResult {
        let mut mesh_builder = MeshBuilder::new();
        
        // Draw vertical lines
        for x in (0..=(SCREEN_WIDTH as i32)).step_by(CELL_SIZE as usize) {
            mesh_builder.line(
                &[
                    [x as f32, 0.0],
                    [x as f32, SCREEN_HEIGHT],
                ],
                1.0,
                Color::new(0.3, 0.3, 0.3, 1.0), // Grey color
            )?;
        }

        // Draw horizontal lines
        for y in (0..=(SCREEN_HEIGHT as i32)).step_by(CELL_SIZE as usize) {
            mesh_builder.line(
                &[
                    [0.0, y as f32],
                    [SCREEN_WIDTH, y as f32],
                ],
                1.0,
                Color::new(0.3, 0.3, 0.3, 1.0), // Grey color
            )?;
        }

        let grid_mesh = graphics::Mesh::from_data(ctx, mesh_builder.build());
        canvas.draw(&grid_mesh, DrawParam::default());
        
        Ok(())
    }

    fn is_point_in_rect(&self, point: Vec2, rect_pos: Vec2, rect_size: Vec2) -> bool {
        point.x >= rect_pos.x 
            && point.x <= rect_pos.x + rect_size.x 
            && point.y >= rect_pos.y 
            && point.y <= rect_pos.y + rect_size.y
    }
}

impl event::EventHandler for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        self.update();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);

        // Draw grid first (so it's behind everything else)
        self.draw_grid(ctx, &mut canvas)?;

        // Draw dots
        for dot in &self.dots {
            let mut mesh_builder = MeshBuilder::new();
            let mesh_data = mesh_builder
                .circle(
                    DrawMode::fill(),
                    [dot.x, dot.y],
                    CELL_SIZE * 0.2,
                    0.1,
                    Color::WHITE,
                )?
                .build();
            let dot_mesh = graphics::Mesh::from_data(ctx, mesh_data);
            canvas.draw(&dot_mesh, DrawParam::default());
        }

        // Draw Pacman
        let mut mesh_builder = MeshBuilder::new();
        
        // Calculate rotation angle based on direction
        let rotation = if self.pacman.direction.length() > 0.0 {
            self.pacman.direction.y.atan2(self.pacman.direction.x)
        } else {
            0.0 // Face right when not moving
        };

        // Draw Pacman body (a pie shape)
        let mesh_data = mesh_builder
            .circle(
                DrawMode::fill(),
                [0.0, 0.0],  // Center at origin for rotation
                self.pacman.size * 0.5,
                0.1,
                Color::YELLOW,
            )?
            .build();
        
        let pacman_mesh = graphics::Mesh::from_data(ctx, mesh_data);
        
        // Draw the pie-shaped mouth cutout (both sides)
        let mouth_mesh = Mesh::new_polygon(
            ctx,
            DrawMode::fill(),
            &[
                [0.0, 0.0],
                [self.pacman.size * 0.5, -self.pacman.size * 0.5 * self.pacman.mouth_angle.sin()],
                [self.pacman.size * 0.5, self.pacman.size * 0.5 * self.pacman.mouth_angle.sin()],
            ],
            Color::BLACK,
        )?;

        // Draw Pacman with proper positioning and rotation
        canvas.draw(
            &pacman_mesh,
            DrawParam::default()
                .dest([self.pacman.pos.x, self.pacman.pos.y])
                .rotation(rotation)
        );
        
        canvas.draw(
            &mouth_mesh,
            DrawParam::default()
                .dest([self.pacman.pos.x, self.pacman.pos.y])
                .rotation(rotation)
        );

        // Draw score
        let score_text = graphics::Text::new(format!("Score: {}", self.score));
        canvas.draw(
            &score_text,
            DrawParam::default()
                .color(Color::WHITE)
                .dest([10.0, 10.0]),
        );

        // Draw victory overlay if game is won
        if self.game_won {
            // Semi-transparent background
            let overlay = graphics::Mesh::new_rectangle(
                ctx,
                DrawMode::fill(),
                graphics::Rect::new(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT),
                Color::new(0.0, 0.0, 0.0, 0.7),
            )?;
            canvas.draw(&overlay, DrawParam::default());

            // "You Won!" text
            let won_text = graphics::Text::new("You Won!");
            let won_dims = won_text.dimensions(ctx);
            canvas.draw(
                &won_text,
                DrawParam::default()
                    .color(Color::WHITE)
                    .dest([
                        SCREEN_WIDTH * 0.5 - won_dims.unwrap().w * 0.5,
                        SCREEN_HEIGHT * 0.4,
                    ]),
            );

            // Final score text
            let score_text = graphics::Text::new(format!("Final Score: {}", self.score));
            let score_dims = score_text.dimensions(ctx);
            canvas.draw(
                &score_text,
                DrawParam::default()
                    .color(Color::WHITE)
                    .dest([
                        SCREEN_WIDTH * 0.5 - score_dims.unwrap().w * 0.5,
                        SCREEN_HEIGHT * 0.5,
                    ]),
            );

            // Play Again button
            let button_width = 200.0;
            let button_height = 50.0;
            let button_x = SCREEN_WIDTH * 0.5 - button_width * 0.5;
            let button_y = SCREEN_HEIGHT * 0.6;
            
            let button = graphics::Mesh::new_rectangle(
                ctx,
                DrawMode::fill(),
                graphics::Rect::new(button_x, button_y, button_width, button_height),
                Color::new(0.3, 0.3, 0.8, 1.0),
            )?;
            canvas.draw(&button, DrawParam::default());

            let button_text = graphics::Text::new("Play Again");
            let text_dims = button_text.dimensions(ctx);
            canvas.draw(
                &button_text,
                DrawParam::default()
                    .color(Color::WHITE)
                    .dest([
                        button_x + button_width * 0.5 - text_dims.unwrap().w * 0.5,
                        button_y + button_height * 0.5 - text_dims.unwrap().h * 0.5,
                    ]),
            );
        }

        canvas.finish(ctx)?;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        input: KeyInput,
        _repeat: bool,
    ) -> GameResult {
        match input.keycode {
            Some(KeyCode::Up) | Some(KeyCode::W) => {
                self.direction_controller.queue_direction(Direction::Up);
            }
            Some(KeyCode::Down) | Some(KeyCode::S) => {
                self.direction_controller.queue_direction(Direction::Down);
            }
            Some(KeyCode::Left) | Some(KeyCode::A) => {
                self.direction_controller.queue_direction(Direction::Left);
            }
            Some(KeyCode::Right) | Some(KeyCode::D) => {
                self.direction_controller.queue_direction(Direction::Right);
            }
            _ => (),
        }
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        if self.game_won && button == MouseButton::Left {
            // Check if click is within Play Again button bounds
            let button_width = 200.0;
            let button_height = 50.0;
            let button_x = SCREEN_WIDTH * 0.5 - button_width * 0.5;
            let button_y = SCREEN_HEIGHT * 0.6;
            
            if self.is_point_in_rect(
                Vec2::new(x, y),
                Vec2::new(button_x, button_y),
                Vec2::new(button_width, button_height),
            ) {
                self.reset();
            }
        }
        Ok(())
    }
}

#[allow(dead_code)]
fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("pacman", "you")
        .window_setup(ggez::conf::WindowSetup::default().title("Pacman"))
        .window_mode(
            ggez::conf::WindowMode::default()
                .dimensions(CELL_SIZE * GRID_SIZE as f32, CELL_SIZE * GRID_SIZE as f32),
        );
    let (ctx, event_loop) = cb.build()?;
    let state = GameState::new();
    event::run(ctx, event_loop, state)
}