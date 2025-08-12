use color_eyre::Result;

mod app;
mod components;
mod engine;
mod entities;
mod gamemap;
mod inventory;
mod los;
mod pathfinding;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let mut app = app::App::new();
    let result = app.run(terminal);
    ratatui::restore();
    result
}
