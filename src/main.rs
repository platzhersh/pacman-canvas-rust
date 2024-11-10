use ggez::{
    event,
    graphics::{self, Color, DrawMode, DrawParam, Mesh, MeshBuilder, LineCap},
    input::keyboard::{KeyCode, KeyInput},
    Context, GameResult,
};
use glam::Vec2;

const GRID_SIZE: i32 = 20;
const CELL_SIZE: f32 = 30.0;
const PACMAN_SPEED: f32 = 5.0;
const SCREEN_WIDTH: f32 = 800.0;
const SCREEN_HEIGHT: f32 = 600.0;

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
}

struct GameState {
    pacman: GameObject,
    dots: Vec<Vec2>,
    score: i32,
    direction_controller: DirectionController,
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
            },
            dots,
            score: 0,
            direction_controller: DirectionController::new(),
        }
    }

    fn update(&mut self) {
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
        let mesh_data = mesh_builder
            .circle(
                DrawMode::fill(),
                [self.pacman.pos.x, self.pacman.pos.y],
                self.pacman.size * 0.5,
                0.1,
                Color::YELLOW,
            )?
            .build();
        let pacman_mesh = graphics::Mesh::from_data(ctx, mesh_data);
        canvas.draw(&pacman_mesh, DrawParam::default());

        // Draw score
        let score_text = graphics::Text::new(format!("Score: {}", self.score));
        canvas.draw(
            &score_text,
            DrawParam::default()
                .color(Color::WHITE)
                .dest([10.0, 10.0]),
        );

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
            Some(KeyCode::Up) => {
                self.direction_controller.queue_direction(Direction::Up);
            }
            Some(KeyCode::Down) => {
                self.direction_controller.queue_direction(Direction::Down);
            }
            Some(KeyCode::Left) => {
                self.direction_controller.queue_direction(Direction::Left);
            }
            Some(KeyCode::Right) => {
                self.direction_controller.queue_direction(Direction::Right);
            }
            _ => (),
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