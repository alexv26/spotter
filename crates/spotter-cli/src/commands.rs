use std::collections::HashMap;

use rand::seq::SliceRandom;
use spotter_core::exercise::{Exercise, ExerciseLibrary, Muscle};

use crate::getUserInput;

/// What the main loop should do after a command runs.
/// A hashmap-of-functions has no `match` to fall through to a `Quit` arm,
/// so handlers report back explicitly instead.
pub enum ControlFlow {
    Continue,
    Quit,
}

/// The shape every command handler must have. `fn(...)` (lowercase) is a
/// plain function pointer type, not a closure - it's what lets every
/// `handle_*` function below be stored as the same type in one HashMap.
pub type Handler = fn(args: &[&str], library: &ExerciseLibrary) -> ControlFlow;

/// Maps a command name (the first word the user types, e.g. "search") to
/// the function that handles it. Look up by name, then call the result
/// with the remaining words as `args`.
pub fn build_command_table() -> HashMap<&'static str, Handler> {
    let mut commands: HashMap<&'static str, Handler> = HashMap::new();
    commands.insert("info", handle_info);
    commands.insert("search", handle_search);
    commands.insert("muscle", handle_muscle);
    commands.insert("equipment", handle_equipment);
    commands.insert("category", handle_category);
    commands.insert("level", handle_level);
    commands.insert("help", handle_help);
    commands.insert("quit", handle_quit);
    commands.insert("exit", handle_quit);
    commands.insert("random", handle_random);
    commands.insert("clear", handle_clear);

    commands
}

// Handlers below are unimplemented placeholders (`todo!()` panics if called).
// Each needs the exact signature of `Handler` to be storable in the table above.

fn handle_info(args: &[&str], library: &ExerciseLibrary) -> ControlFlow {
    todo!()
}

fn handle_search(args: &[&str], library: &ExerciseLibrary) -> ControlFlow {
    // start with basic implementation of search. add further args later

    if args.len() < 1 {
        println!("Error: Not enough arguments.");
        return ControlFlow::Continue;
    }

    let mut search_term = args[0];
    #[warn(unused_assignments)]
    let mut joined_term = String::new();

    if search_term.contains('\"') {
        let mut terms: Vec<&str> = Vec::new();
        terms.push(args[0].trim_matches('\"'));
        for i in 1..args.len() {
            let arg = args[i];
            let word: &str = arg.trim_matches('\"');
            terms.push(word);
            if arg.contains('\"') {
                break;
            }
        }
        joined_term = terms.join(" ");
        search_term = &joined_term;
    }

    let similarity_scores = library.smart_search(search_term);

    let mut counter = 1;
    for scored in similarity_scores {
        println!(
            "{counter:>3}: {}, score: {}",
            Exercise::short_display(scored.exercise),
            scored.score
        );
        counter += 1;
    }

    ControlFlow::Continue
}

fn handle_muscle(args: &[&str], library: &ExerciseLibrary) -> ControlFlow {
    todo!()
}

fn handle_equipment(args: &[&str], library: &ExerciseLibrary) -> ControlFlow {
    todo!()
}

fn handle_category(args: &[&str], library: &ExerciseLibrary) -> ControlFlow {
    todo!()
}

fn handle_level(args: &[&str], library: &ExerciseLibrary) -> ControlFlow {
    todo!()
}

fn handle_help(args: &[&str], library: &ExerciseLibrary) -> ControlFlow {
    todo!()
}

fn handle_quit(_args: &[&str], _library: &ExerciseLibrary) -> ControlFlow {
    ControlFlow::Quit
}

pub fn handle_clear(_args: &[&str], _library: &ExerciseLibrary) -> ControlFlow {
    // \x1B[2J clears the screen.
    // \x1B[3J clears the scrollback buffer.
    // \x1B[1;1H moves the cursor to row 1, column 1.
    println!("\x1B[2J\x1B[3J\x1B[1;1H");
    ControlFlow::Continue
}
/// Worked example: `random` with no args picks from every exercise;
/// `random <muscle>` (e.g. `random biceps`) narrows the pool to that muscle first.
/// `args` is just the words typed after the command name - `args.first()` is
/// "was there a word there at all", and if so, we try to turn that word into
/// a `Muscle` before doing anything else with it.
fn handle_random(args: &[&str], library: &ExerciseLibrary) -> ControlFlow {
    let candidates: Vec<&Exercise> = match args.first() {
        None => library.catalog.values().collect(),
        Some(muscle_arg) => match muscle_arg.parse::<Muscle>() {
            Ok(muscle) => library.find_by_muscle(muscle),
            Err(err) => {
                println!("{err}");
                return ControlFlow::Continue;
            }
        },
    };

    match candidates.choose(&mut rand::thread_rng()) {
        Some(exercise) => println!("{exercise}"),
        None => println!("No exercises matched."),
    }

    ControlFlow::Continue
}
