use color_eyre::Result;

mod app;
mod entities;
mod gamemap;
mod los;
mod procgen;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let mut app = app::App::new();
    let result = app.run(terminal);
    ratatui::restore();
    result
}
