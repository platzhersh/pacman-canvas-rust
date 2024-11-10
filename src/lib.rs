use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

#[wasm_bindgen]
pub struct Game {
    state: GameState,
    context: CanvasRenderingContext2d,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<Game, JsValue> {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document.get_element_by_id(canvas_id).unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();

        let context = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()?;

        Ok(Game {
            state: GameState::new(),
            context,
        })
    }

    pub fn update(&mut self) {
        self.state.update();
    }

    pub fn render(&self) {
        // Clear canvas
        self.context.clear_rect(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT);
        
        // Draw grid
        self.context.set_stroke_style(&JsValue::from_str("#333333"));
        for x in (0..SCREEN_WIDTH as i32).step_by(CELL_SIZE as usize) {
            self.context.begin_path();
            self.context.move_to(x as f64, 0.0);
            self.context.line_to(x as f64, SCREEN_HEIGHT as f64);
            self.context.stroke();
        }
        // ... similar for horizontal lines ...

        // Draw Pacman
        self.context.set_fill_style(&JsValue::from_str("yellow"));
        self.context.begin_path();
        self.context.arc(
            self.state.pacman.pos.x as f64,
            self.state.pacman.pos.y as f64,
            (self.state.pacman.size * 0.5) as f64,
            self.state.pacman.mouth_angle as f64,
            2.0 * std::f64::consts::PI - self.state.pacman.mouth_angle as f64,
            false,
        ).unwrap();
        self.context.fill();
    }

    #[wasm_bindgen]
    pub fn handle_keydown(&mut self, key_code: &str) {
        match key_code {
            "ArrowUp" | "KeyW" => {
                self.state.direction_controller.queue_direction(Direction::Up);
            }
            "ArrowDown" | "KeyS" => {
                self.state.direction_controller.queue_direction(Direction::Down);
            }
            "ArrowLeft" | "KeyA" => {
                self.state.direction_controller.queue_direction(Direction::Left);
            }
            "ArrowRight" | "KeyD" => {
                self.state.direction_controller.queue_direction(Direction::Right);
            }
            _ => (),
        }
    }
} 