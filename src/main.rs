use ggez::{
    event,
    graphics::{self, Color, DrawMode, DrawParam, Mesh, MeshBuilder},
    input::keyboard::{KeyCode, KeyInput},
    Context, GameResult,
};
use glam::Vec2;

const GRID_SIZE: i32 = 20;
const CELL_SIZE: f32 = 30.0;
const PACMAN_SPEED: f32 = 5.0;

struct GameObject {
    pos: Vec2,
    direction: Vec2,
    size: f32,
}

struct GameState {
    pacman: GameObject,
    dots: Vec<Vec2>,
    score: i32,
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
                pos: Vec2::new(CELL_SIZE * 1.5, CELL_SIZE * 1.5),
                direction: Vec2::new(0.0, 0.0),
                size: CELL_SIZE * 0.8,
            },
            dots,
            score: 0,
        }
    }

    fn update(&mut self) {
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
}

impl event::EventHandler for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        self.update();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);

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
                self.pacman.direction = Vec2::new(0.0, -1.0);
            }
            Some(KeyCode::Down) => {
                self.pacman.direction = Vec2::new(0.0, 1.0);
            }
            Some(KeyCode::Left) => {
                self.pacman.direction = Vec2::new(-1.0, 0.0);
            }
            Some(KeyCode::Right) => {
                self.pacman.direction = Vec2::new(1.0, 0.0);
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