//! Command dispatch for the Spotter CLI: maps the first word a user types to
//! a handler function, via a `HashMap<&str, Handler>` rather than an enum +
//! `match` (see the design discussion this module grew out of - a fixed enum
//! would give compile-time-checked dispatch, but this table is easier to keep
//! extending one command at a time as handlers get filled in).

use std::collections::HashMap;

use rand::seq::SliceRandom;
use spotter_core::exercise::{
    Category, Equipment, Exercise, ExerciseLibrary, Force, Level, Mechanic, Muscle,
};

use crate::input::{ArgType, get_flag_value, get_user_input, parse_args};
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

// Every handler below needs the exact signature of `Handler` to be storable
// in the table above. `handle_info` and `handle_help` are still `todo!()`
// placeholders (panic if called); everything else is implemented.

/// `info <exercise id or name>` - planned: look up and print one exercise in full.
/// Not implemented yet.
fn handle_info(args: &[&str], library: &ExerciseLibrary) -> ControlFlow {
    // args = exercise name (should only be ONE arg)
    let pargs = match parse_args(args, None) {
        Ok(val) => val,
        Err(e) => {
            println!("{e}");
            return ControlFlow::Quit;
        }
    };

    if pargs.len() != 1 {
        println!(
            "Error: info takes exactly one argument. Multi-word exercise names should be wrapped in quotes."
        );
        println!("Usage: info \"exercise name\"");
        return ControlFlow::Continue;
    }

    // Get target exercise from pargs.
    let target_exercise = match pargs.get(0) {
        Some(e) => match e {
            ArgType::Positional(s) => s,
            _ => {
                println!("Error: info only accepts an exercise name, not flags.");
                return ControlFlow::Quit;
            }
        },
        None => {
            println!("Error: no exercise name provided.");
            return ControlFlow::Quit;
        }
    };

    // Check that the target exercise is a valid exercise (and there is only one)
    //TODO: what if one exercise's name is PART of another one? Like: "biceps curl" and "barbell biceps curl"
    let found_exercises = library.find_by_name(target_exercise);
    // If none found, print error
    if found_exercises.len() < 1 {
        println!("Error: no exercises matched that name.");
        return ControlFlow::Continue;
    } else if found_exercises.len() > 1 {
        // IF more than one exercise, print the list and have the user input which one they are referencing
        let mut counter = 1;

        for e in &found_exercises {
            println!("{counter:>3}: {}", Exercise::short_display(e));
            counter += 1;
        }
        let user_in = get_user_input("Multiple exercises matched - enter the number of the one you meant: ".to_string());
        let mut index : usize = user_in.parse().unwrap();

        if index > counter - 1 || index < 1 {
            println!("Error: invalid selection.");
        } else {
            index -= 1;
            let matching_exercise = match found_exercises.get(index) {
            Some(e) => e,
            None => {
                println!("Error: no matching exercise found.");
                return ControlFlow::Continue;
            }
        };
        println!("{matching_exercise}");
        }
        

    } else {
        let matching_exercise = match found_exercises.get(0) {
            Some(e) => e,
            None => {
                println!("Error: no matching exercise found.");
                return ControlFlow::Continue;
            }
        };
        println!("{matching_exercise}");
    }

    return ControlFlow::Continue;
}

/// `search <term>` (or `search "multi word term"`) - prints every exercise
/// whose name matches, ranked by [`ExerciseLibrary::smart_search`]'s match-quality
/// score (best matches first, printed as `score: N`, lower is better).
///
/// Accepts two optional value-taking flags to narrow results before ranking:
/// `-level <level>` and `-equipment <equipment>` (e.g.
/// `search curl -level beginner -equipment barbell`).
fn handle_search(args: &[&str], library: &ExerciseLibrary) -> ControlFlow {
    // start with basic implementation of search. add further args later
    let parsed_args = match parse_args(args, Some(&["-level", "-equipment"])) {
        Ok(pargs) => pargs,
        Err(err) => {
            println!("{}", err);
            return ControlFlow::Continue;
        }
    };
    if parsed_args.len() < 1 {
        println!("Error: Not enough arguments.");
        return ControlFlow::Continue;
    }

    let positionals: Vec<&str> = parsed_args
        .iter()
        .filter_map(|arg| match arg {
            ArgType::Positional(s) => Some(s.as_str()),
            _ => None,
        })
        .collect();

    if positionals.is_empty() {
        println!("Error: expected a search term.");
        return ControlFlow::Continue;
    } else if positionals.len() > 1 {
        println!("Error: multi-word search terms should be in quotations.");
        return ControlFlow::Continue;
    }

    let search_term = positionals.join(" ");

    // Adding a new filterable flag is one more line here (plus registering it
    // above in the `parse_args` call, and one more `&&` clause in the closure below).
    let level: Option<Level> = match get_flag_value(&parsed_args, "-level") {
        Ok(value) => value,
        Err(err) => {
            println!("{err}");
            return ControlFlow::Continue;
        }
    };
    let equipment: Option<Equipment> = match get_flag_value(&parsed_args, "-equipment") {
        Ok(value) => value,
        Err(err) => {
            println!("{err}");
            return ControlFlow::Continue;
        }
    };

    let similarity_scores = library.smart_search(&search_term, |exercise| {
        level.map_or(true, |l| exercise.level == l)
            && equipment.map_or(true, |e| exercise.equipment == Some(e))
    });

    if similarity_scores.len() == 0 {
        println!("No matches found.");
    }

    let mut counter = 1;
    for scored in similarity_scores {
        println!("{counter:>3}: {}", Exercise::short_display(scored.exercise));
        counter += 1;
    }

    ControlFlow::Continue
}

/// `muscle <muscle>` - lists exercises training the given muscle.
fn handle_muscle(args: &[&str], library: &ExerciseLibrary) -> ControlFlow {
    let parsed_args = match parse_args(args, None) {
        Ok(pargs) => pargs,
        Err(err) => {
            println!("{}", err);
            return ControlFlow::Continue;
        }
    };

    if parsed_args.len() != 1 {
        println!(
            "Error: function should only take one argument. Multi-word arguments should go in quotes."
        );
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

    let muscle: Muscle = match &parsed_args[0] {
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
    let parsed_args = match parse_args(args, None) {
        Ok(pargs) => pargs,
        Err(err) => {
            println!("{}", err);
            return ControlFlow::Continue;
        }
    };

    if parsed_args.len() != 1 {
        println!(
            "Error: function should only take one argument. Multi-word arguments should go in quotes."
        );
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
    let parsed_args = match parse_args(args, None) {
        Ok(pargs) => pargs,
        Err(err) => {
            println!("{}", err);
            return ControlFlow::Continue;
        }
    };

    if parsed_args.len() != 1 {
        println!(
            "Error: function should only take one argument. Multi-word arguments should go in quotes."
        );
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
    let parsed_args = match parse_args(args, None) {
        Ok(pargs) => pargs,
        Err(err) => {
            println!("{}", err);
            return ControlFlow::Continue;
        }
    };

    if parsed_args.len() != 1 {
        println!(
            "Error: function should only take one argument. Multi-word arguments should go in quotes."
        );
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
    let parsed_args = match parse_args(args, None) {
        Ok(pargs) => pargs,
        Err(err) => {
            println!("{}", err);
            return ControlFlow::Continue;
        }
    };

    if parsed_args.len() != 1 {
        println!(
            "Error: function should only take one argument. Multi-word arguments should go in quotes."
        );
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
    let parsed_args = match parse_args(args, None) {
        Ok(pargs) => pargs,
        Err(err) => {
            println!("{}", err);
            return ControlFlow::Continue;
        }
    };

    if parsed_args.len() != 1 {
        println!(
            "Error: function should only take one argument. Multi-word arguments should go in quotes."
        );
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

/// `help` - lists the available commands and their syntax.
fn handle_help(_args: &[&str], _library: &ExerciseLibrary) -> ControlFlow {
    let rows: [(&str, &str); 12] = [
        ("info \"<exercise name>\"", "Show full details for one exercise"),
        (
            "search <term> [-level L] [-equipment E]",
            "Find exercises by name, best matches first",
        ),
        ("muscle <muscle>", "List exercises training a muscle"),
        ("equipment <equipment>", "List exercises requiring equipment"),
        ("category <category>", "List exercises in a category"),
        ("level <level>", "List exercises at a difficulty"),
        ("force <force>", "List exercises by force (push, pull, static)"),
        (
            "mechanic <mechanic>",
            "List exercises by mechanic (isolation, compound)",
        ),
        ("random [muscle]", "Show a random exercise, optionally by muscle"),
        ("clear", "Clear the terminal"),
        ("help", "Show this message"),
        ("quit / exit", "Exit Spotter"),
    ];

    println!("Available commands:");
    for (syntax, description) in rows {
        println!("  {syntax:<42} {description}");
    }
    println!();
    println!("Multi-word arguments need quotes, e.g. muscle \"lower back\".");

    ControlFlow::Continue
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
