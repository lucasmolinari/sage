mod editor;
mod out;

use editor::Editor;
use std::io::{self};

pub const TAB_SZ: usize = 8;

fn main() -> io::Result<()> {
    let mut editor = Editor::new()?;

    editor.init()?;
    editor.poll()?;
    Ok(())
}
