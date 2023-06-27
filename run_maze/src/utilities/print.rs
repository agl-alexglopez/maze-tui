use crate::maze;

use crossterm::{cursor, terminal, ExecutableCommand, QueueableCommand};
use std::io::{stdout, Write};

// The mazes look WAY better if the cursor square disapears while it builds.
pub struct InvisibleCursor;

impl InvisibleCursor {
    pub fn new() -> Self {
        stdout().execute(cursor::Hide).expect("Failed to hide cursor.");
        Self
    }
}

impl Drop for InvisibleCursor {
    fn drop(&mut self) {
        stdout().execute(cursor::Show).expect(
            "Failed to unhide your cursor. Sorry! Restart your terminal."
        );
    }
}

// Execute the command so clearing the screen forcefully flushes for the caller.
pub fn clear_screen() {
    stdout()
        .execute(terminal::Clear(terminal::ClearType::All))
        .expect("Could not clear screen, terminal may be incompatible.");
}

// Queue the command so setting the cursor position does NOT forcefully flush for caller.
pub fn set_cursor_position(p: maze::Point) {
    stdout()
        .queue(cursor::MoveTo((p.col) as u16, (p.row) as u16))
        .expect("Could not move cursor, terminal may be incompatible.");
}

pub fn flush() {
    stdout()
        .flush()
        .expect("Could not clear screen,terminal may be incompatible.");
}
