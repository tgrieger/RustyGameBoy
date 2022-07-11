pub mod rom;

use std::{env, io};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Expected only the path to the ROM.",
        ));
    }

    let r = rom::Rom::new(&args[1])?;

    Ok(())
}
