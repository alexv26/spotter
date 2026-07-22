use std::collections::HashMap;
mod commands;
mod input;

use std::path::Path;

use spotter_core::exercise::Exercise;
use spotter_core::exercise::ExerciseLibrary;
use spotter_core::exercise::Level;

use crate::commands::Handler;
use crate::commands::build_command_table;
use crate::input::getUserInput;

fn main() {
    let path = Path::new("data/free-exercise-db/exercises");
    let library = match ExerciseLibrary::load(path) {
        Ok(library) => library,
        Err(err) => {
            eprintln!("Couldn't load exercise library: {err}");
            std::process::exit(1);
        }
    };

    // Clear Terminal
    println!("\x1B[2J\x1B[3J\x1B[1;1H");

    // Workout CLI Loop
    println!("===== SPOTTER CLI INTERFACE =====");
    let commands_table: HashMap<&'static str, Handler> = build_command_table();

    loop {
        let user_cmd: String = getUserInput("$ ".to_string());
        let split_str: Vec<&str> = user_cmd.split_whitespace().collect();
        let args: &[&str] = &split_str[..];

        if args.is_empty() {
            continue;
        }

        let control_flow = match commands_table.get(args[0]) {
            Some(fxn) => fxn(&args[1..], &library),
            None => {
                println!("Unknown command: {}", args[0]);
                commands::ControlFlow::Continue
            }
        };

        if let commands::ControlFlow::Quit = control_flow {
            break;
        }
    }
}
