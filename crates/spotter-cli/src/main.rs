use std::io::Write;
mod commands;

use std::io;
use std::path::Path;

use spotter_core::exercise::Exercise;
use spotter_core::exercise::ExerciseLibrary;
use spotter_core::exercise::Level;

fn getUserInput(cliMsg: String) -> String {
    let mut input = String::new();

    print!("{cliMsg}");
    let _ = io::stdout().flush();
    // Read the line from standard input
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    // Remove the trailing newline character (\n)
    input.trim().to_string()
}

fn main() {
    let path = Path::new("data/free-exercise-db/exercises");
    let library = match ExerciseLibrary::load(path) {
        Ok(library) => library,
        Err(err) => {
            eprintln!("Couldn't load exercise library: {err}");
            std::process::exit(1);
        }
    };

    // Workout CLI Loop
    println!("===== SPOTTER CLI INTERFACE =====");
    loop {
        let userCmd: String = getUserInput("$ ".to_string());
        println!("User Command: {userCmd}");
    }
}
