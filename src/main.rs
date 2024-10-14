mod term;

use std::io::{self};
use term::Editor;

fn main() -> io::Result<()> {
    let mut editor = Editor::new()?;
    editor.init()?;
    editor.poll()?;
    editor.destroy()?;
    Ok(())
}
