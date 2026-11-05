//! Native game entry, separate from the editor and the headless abilities lab.
#[path = "../games/land-of-survivors/src/characters.rs"]
mod characters;
#[path = "../games/land-of-survivors/src/front_end.rs"]
mod front_end;
#[path = "../games/land-of-survivors/src/front_ui.rs"]
mod front_ui;
#[path = "../games/land-of-survivors/src/front_window.rs"]
mod front_window;
#[path = "../games/land-of-survivors/src/presentation.rs"]
mod presentation;
fn main() -> anyhow::Result<()> {
    front_window::main()
}
