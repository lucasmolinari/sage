mod editor;
mod grid;

use editor::Editor;
use std::env;
use std::io::{self};
use std::path::Path;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut editor = Editor::new()?;

    if args.len() > 1 {
        let path = Path::new(&args[1]).canonicalize()?;
        editor.open_file(path)?;
    }
    editor.init()?;
    editor.poll()?;
    editor.destroy()?;
    Ok(())
}
