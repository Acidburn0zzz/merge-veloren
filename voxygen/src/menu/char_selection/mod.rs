mod ui;

use crate::{
    window::{Event, Window},
    session::SessionState,
    GlobalState, PlayState, PlayStateResult,
};
use common::clock::Clock;
use std::time::Duration;
use ui::CharSelectionUi;
use vek::*;

const FPS: u64 = 60;

pub struct CharSelectionState {
    char_selection_ui: CharSelectionUi,
}

impl CharSelectionState {
    /// Create a new `CharSelectionState`
    pub fn new(window: &mut Window) -> Self {
        Self {
            char_selection_ui: CharSelectionUi::new(window),
        }
    }
}

// The background colour
const BG_COLOR: Rgba<f32> = Rgba {
    r: 0.0,
    g: 0.3,
    b: 1.0,
    a: 1.0,
};

impl PlayState for CharSelectionState {
    fn play(&mut self, global_state: &mut GlobalState) -> PlayStateResult {
        // Set up an fps clock
        let mut clock = Clock::new();

        loop {
            // Handle window events
            for event in global_state.window.fetch_events() {
                match event {
                    Event::Close => return PlayStateResult::Shutdown,
                    // When any key is pressed, go to the main menu
                    Event::Char(_) =>
                        return PlayStateResult::Push(
                            Box::new(SessionState::new(&mut global_state.window).unwrap()) // TODO: Handle this error
                        ),
                    // Pass events to ui
                    Event::Ui(event) => {
                        self.char_selection_ui.handle_event(event);
                    }
                    // Ignore all other events
                    _ => {}
                }
            }

            global_state.window.renderer_mut().clear(BG_COLOR);

            // Maintain the UI
            self.char_selection_ui.maintain(global_state.window.renderer_mut());

            // Draw the UI to the screen
            self.char_selection_ui.render(global_state.window.renderer_mut());

            // Finish the frame
            global_state.window.renderer_mut().flush();
            global_state
                .window
                .swap_buffers()
                .expect("Failed to swap window buffers");

            // Wait for the next tick
            clock.tick(Duration::from_millis(1000 / FPS));
        }
    }

    fn name(&self) -> &'static str {
        "Title"
    }
}
