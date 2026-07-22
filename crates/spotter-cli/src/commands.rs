//! Command dispatch for the Spotter CLI: maps the first word a user types to
//! a handler function, via a `HashMap<&str, Handler>` rather than an enum +
//! `match` (see the design discussion this module grew out of - a fixed enum
//! would give compile-time-checked dispatch, but this table is easier to keep
//! extending one command at a time as handlers get filled in).

use std::collections::HashMap;

use rand::seq::SliceRandom;
use spotter_core::exercise::{Category, Equipment, Exercise, ExerciseLibrary, Force, Level, Mechanic, Muscle};

use crate::input::{ArgType, getUserInput, parseArgs};
use std::str::FromStr;

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
    commands.insert("force", handle_force);
    commands.insert("mechanic", handle_mechanic);

    commands
}

// Handlers below are unimplemented placeholders (`todo!()` panics if called).
// Each needs the exact signature of `Handler` to be storable in the table above.

/// `info <exercise id or name>` - planned: look up and print one exercise in full.
/// Not implemented yet.
fn handle_info(args: &[&str], library: &ExerciseLibrary) -> ControlFlow {
    todo!()
}

/// `search <term>` (or `search "multi word term"`) - prints every exercise
/// whose name matches, ranked by [`ExerciseLibrary::smart_search`]'s match-quality
/// score (best matches first, printed as `score: N`, lower is better).
fn handle_search(args: &[&str], library: &ExerciseLibrary) -> ControlFlow {
    // start with basic implementation of search. add further args later
    let parsed_args = match parseArgs(args, None) {
        Ok(pargs) => pargs,
        Err(err) => {
            println!("{}", err);
            return ControlFlow::Continue
        }
    };
    if parsed_args.len() < 1 {
        println!("Error: Not enough arguments.");
        return ControlFlow::Continue;
    }

    // If there are any flags in parsed_args, error:
    if parsed_args
        .iter()
        .any(|arg| matches!(arg, ArgType::Flag(_) | ArgType::Option { .. }))
    {
        println!("Error: invalid flag.");
        return ControlFlow::Continue;
    }

    let mut words: Vec<&str> = Vec::new();
    for arg in &parsed_args {
        if let ArgType::Positional(word) = arg {
            words.push(word);
        }
    }
    let search_term = words.join(" ");

    let similarity_scores = library.smart_search(&search_term);

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

/// `muscle <muscle>` - lists exercises training the given muscle.
fn handle_muscle(args: &[&str], library: &ExerciseLibrary) -> ControlFlow {
    let parsed_args = match parseArgs(args, None) {
        Ok(pargs) => pargs,
        Err(err) => {
            println!("{}", err);
            return ControlFlow::Continue
        }
    };

    if parsed_args.len() != 1 {
        println!("Error: function should only take one argument. Multi-word arguments should go in quotes.");
        println!("Usage: muscle \"search_muscle\"");
        return ControlFlow::Continue;
    }

    // If there are any flags in parsed_args, error:
    if parsed_args
        .iter()
        .any(|arg| matches!(arg, ArgType::Flag(_) | ArgType::Option { .. }))
    {
        println!("Error: invalid flag.");
        return ControlFlow::Continue;
    }

    let muscle : Muscle = match &parsed_args[0] {
        ArgType::Positional(m) => match Muscle::from_str(&m) {
            Ok(mscl) => mscl,
            Err(_) => {
                println!("Error: invalid muscle.");
                return ControlFlow::Continue;
            }
        },
        _ => {
            println!("Error: invalid input.");
            println!("Usage: muscle <search_muscle>");
            return ControlFlow::Continue;
        }
    };


    let mut counter = 1;
    for exercise in library.find_by_muscle(muscle) {
        println!("{counter:>3}: {}", Exercise::short_display(exercise));
        counter += 1;
    }

    ControlFlow::Continue
}

/// `equipment <equipment>` - lists exercises requiring the given equipment.
fn handle_equipment(args: &[&str], library: &ExerciseLibrary) -> ControlFlow {
    let parsed_args = match parseArgs(args, None) {
        Ok(pargs) => pargs,
        Err(err) => {
            println!("{}", err);
            return ControlFlow::Continue
        }
    };

    if parsed_args.len() != 1 {
        println!("Error: function should only take one argument. Multi-word arguments should go in quotes.");
        println!("Usage: equipment \"search_equipment\"");
        return ControlFlow::Continue;
    }

    // If there are any flags in parsed_args, error:
    if parsed_args
        .iter()
        .any(|arg| matches!(arg, ArgType::Flag(_) | ArgType::Option { .. }))
    {
        println!("Error: invalid flag.");
        return ControlFlow::Continue;
    }

    let equipment: Equipment = match &parsed_args[0] {
        ArgType::Positional(e) => match Equipment::from_str(e) {
            Ok(eq) => eq,
            Err(_) => {
                println!("Error: invalid equipment.");
                return ControlFlow::Continue;
            }
        },
        _ => {
            println!("Error: invalid input.");
            println!("Usage: equipment <search_equipment>");
            return ControlFlow::Continue;
        }
    };

    let mut counter = 1;
    for exercise in library.find_by_equipment(equipment) {
        println!("{counter:>3}: {}", Exercise::short_display(exercise));
        counter += 1;
    }

    ControlFlow::Continue
}

/// `category <category>` - lists exercises in the given category.
fn handle_category(args: &[&str], library: &ExerciseLibrary) -> ControlFlow {
    let parsed_args = match parseArgs(args, None) {
        Ok(pargs) => pargs,
        Err(err) => {
            println!("{}", err);
            return ControlFlow::Continue
        }
    };

    if parsed_args.len() != 1 {
        println!("Error: function should only take one argument. Multi-word arguments should go in quotes.");
        println!("Usage: category \"search_category\"");
        return ControlFlow::Continue;
    }

    // If there are any flags in parsed_args, error:
    if parsed_args
        .iter()
        .any(|arg| matches!(arg, ArgType::Flag(_) | ArgType::Option { .. }))
    {
        println!("Error: invalid flag.");
        return ControlFlow::Continue;
    }

    let category: Category = match &parsed_args[0] {
        ArgType::Positional(c) => match Category::from_str(c) {
            Ok(cat) => cat,
            Err(_) => {
                println!("Error: invalid category.");
                return ControlFlow::Continue;
            }
        },
        _ => {
            println!("Error: invalid input.");
            println!("Usage: category <search_category>");
            return ControlFlow::Continue;
        }
    };

    let mut counter = 1;
    for exercise in library.find_by_category(category) {
        println!("{counter:>3}: {}", Exercise::short_display(exercise));
        counter += 1;
    }

    ControlFlow::Continue
}

/// `level <level>` - lists exercises at the given difficulty.
fn handle_level(args: &[&str], library: &ExerciseLibrary) -> ControlFlow {
    let parsed_args = match parseArgs(args, None) {
        Ok(pargs) => pargs,
        Err(err) => {
            println!("{}", err);
            return ControlFlow::Continue
        }
    };

    if parsed_args.len() != 1 {
        println!("Error: function should only take one argument. Multi-word arguments should go in quotes.");
        println!("Usage: level \"search_level\"");
        return ControlFlow::Continue;
    }

    // If there are any flags in parsed_args, error:
    if parsed_args
        .iter()
        .any(|arg| matches!(arg, ArgType::Flag(_) | ArgType::Option { .. }))
    {
        println!("Error: invalid flag.");
        return ControlFlow::Continue;
    }

    let level: Level = match &parsed_args[0] {
        ArgType::Positional(l) => match Level::from_str(l) {
            Ok(lvl) => lvl,
            Err(_) => {
                println!("Error: invalid level.");
                return ControlFlow::Continue;
            }
        },
        _ => {
            println!("Error: invalid input.");
            println!("Usage: level <search_level>");
            return ControlFlow::Continue;
        }
    };

    let mut counter = 1;
    for exercise in library.find_by_level(level) {
        println!("{counter:>3}: {}", Exercise::short_display(exercise));
        counter += 1;
    }

    ControlFlow::Continue
}

/// `force <force>` - lists exercises with the given force (push, pull, static).
fn handle_force(args: &[&str], library: &ExerciseLibrary) -> ControlFlow {
    let parsed_args = match parseArgs(args, None) {
        Ok(pargs) => pargs,
        Err(err) => {
            println!("{}", err);
            return ControlFlow::Continue
        }
    };

    if parsed_args.len() != 1 {
        println!("Error: function should only take one argument. Multi-word arguments should go in quotes.");
        println!("Usage: force \"search_force\"");
        return ControlFlow::Continue;
    }

    // If there are any flags in parsed_args, error:
    if parsed_args
        .iter()
        .any(|arg| matches!(arg, ArgType::Flag(_) | ArgType::Option { .. }))
    {
        println!("Error: invalid flag.");
        return ControlFlow::Continue;
    }

    let force: Force = match &parsed_args[0] {
        ArgType::Positional(f) => match Force::from_str(f) {
            Ok(frc) => frc,
            Err(_) => {
                println!("Error: invalid force.");
                return ControlFlow::Continue;
            }
        },
        _ => {
            println!("Error: invalid input.");
            println!("Usage: force <search_force>");
            return ControlFlow::Continue;
        }
    };

    let mut counter = 1;
    for exercise in library.find_by_force(force) {
        println!("{counter:>3}: {}", Exercise::short_display(exercise));
        counter += 1;
    }

    ControlFlow::Continue
}

/// `mechanic <mechanic>` - lists exercises with the given mechanic (isolation, compound).
fn handle_mechanic(args: &[&str], library: &ExerciseLibrary) -> ControlFlow {
    let parsed_args = match parseArgs(args, None) {
        Ok(pargs) => pargs,
        Err(err) => {
            println!("{}", err);
            return ControlFlow::Continue
        }
    };

    if parsed_args.len() != 1 {
        println!("Error: function should only take one argument. Multi-word arguments should go in quotes.");
        println!("Usage: mechanic \"search_mechanic\"");
        return ControlFlow::Continue;
    }

    // If there are any flags in parsed_args, error:
    if parsed_args
        .iter()
        .any(|arg| matches!(arg, ArgType::Flag(_) | ArgType::Option { .. }))
    {
        println!("Error: invalid flag.");
        return ControlFlow::Continue;
    }

    let mechanic: Mechanic = match &parsed_args[0] {
        ArgType::Positional(m) => match Mechanic::from_str(m) {
            Ok(mech) => mech,
            Err(_) => {
                println!("Error: invalid mechanic.");
                return ControlFlow::Continue;
            }
        },
        _ => {
            println!("Error: invalid input.");
            println!("Usage: mechanic <search_mechanic>");
            return ControlFlow::Continue;
        }
    };

    let mut counter = 1;
    for exercise in library.find_by_mechanic(mechanic) {
        println!("{counter:>3}: {}", Exercise::short_display(exercise));
        counter += 1;
    }

    ControlFlow::Continue
}

/// `help` - planned: list the available commands. Not implemented yet.
fn handle_help(args: &[&str], library: &ExerciseLibrary) -> ControlFlow {
    todo!()
}

/// `quit` / `exit` - signals the main loop to stop.
fn handle_quit(_args: &[&str], _library: &ExerciseLibrary) -> ControlFlow {
    ControlFlow::Quit
}

/// `clear` - clears the terminal screen and scrollback via raw ANSI escape codes.
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
