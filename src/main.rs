use std::env;

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Expected only the path to the ROM.");
    }

    Ok(())
}
