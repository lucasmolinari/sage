mod editor;
mod out;

use editor::Editor;
use std::io::{self};

fn main() -> io::Result<()> {
    let mut editor = Editor::new()?;

    editor.init()?;
    editor.poll()?;
    Ok(())
}
