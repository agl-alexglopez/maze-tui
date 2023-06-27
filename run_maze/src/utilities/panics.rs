// I think it would be really mean to panic and then the user loses their cursor, as
// I have experienced. Because I hide the cursor to start, all of my code should panic
// when appropriate but responsibly take back the hook and restore the cursor.
// This is made to use as you would normal panic!(), except it just prints the message
// and exits. This is used throughout all modules in the repo as default panic.
#[macro_export]
macro_rules! maze_panic {
    ($($arg:tt)*) => {
        {
            use std::fmt::Write;
            let mut buf = String::new();
            write!(&mut buf, $($arg)*).expect("Couldn't write to buffer");
            eprintln!("{}", buf);
        }
        use std::io::stdout;
        use crossterm::{cursor::Show, ExecutableCommand};
        stdout().execute(Show).expect(
            "Failed to unhide the cursor. Sorry! Restart your terminal."
        );
        panic!();
    };
}
