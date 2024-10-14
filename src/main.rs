mod term;

use std::io::{self};

fn main() -> io::Result<()> {
    let stdout = std::io::stdout();
    term::init(&stdout)?;
    term::poll(&stdout)?;
    term::destroy(&stdout)?;
    Ok(())
}
