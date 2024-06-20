mod run;
mod tui;

fn main() -> tui::Result<()> {
    let status = run::run();
    status?;
    Ok(())
}
