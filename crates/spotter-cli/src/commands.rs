//! Command dispatch for the Spotter CLI: maps the first word a user types to
//! a handler function, via a `HashMap<&str, Handler>` rather than an enum +
//! `match` (see the design discussion this module grew out of - a fixed enum
//! would give compile-time-checked dispatch, but this table is easier to keep
//! extending one command at a time as handlers get filled in).

use std::collections::HashMap;

use rand::seq::SliceRandom;
use spotter_core::{exercise::{
    Category, Equipment, Exercise, ExerciseLibrary, Force, Level, Mechanic, Muscle, ScoredExercise,
}, workout::{LoggedExercise, SetType, WorkoutLog}};

use crate::input::{ArgType, get_flag_value, get_user_input, parse_args};
use std::str::FromStr;

/// Runs `parse_args`, printing the error and returning `$on_err` from the
/// calling function if it fails. `return` inside a macro expands into the
/// call site, so `$on_err` can be `ControlFlow::Continue`, `ControlFlow::Quit`,
/// `LoggingControlFlow::Continue`, or anything else the caller's return type needs.
macro_rules! parse_args_or_return {
    ($args:expr, $flags:expr, $on_err:expr) => {
        match parse_args($args, $flags) {
            Ok(pargs) => pargs,
            Err(err) => {
                println!("{err}");
                return $on_err;
            }
        }
    };
}

/// Resolves a non-empty list of name-search candidates (from `info` or the
/// in-workout `add`) down to the one the user means: the only one if there's
/// just one match, otherwise lists them and asks for a number. Prints its own
/// error and returns `None` if the selection doesn't parse or is out of range
/// - callers only need to handle the empty-list ("no matches") case themselves.
fn choose_exercise<'a>(matches: &[&'a Exercise]) -> Option<&'a Exercise> {
    if matches.len() == 1 {
        return Some(matches[0]);
    }

    for (i, e) in matches.iter().enumerate() {
        println!("{:>3}: {}", i + 1, Exercise::short_display(e));
    }
    let user_in = get_user_input("Multiple exercises matched - enter the number of the one you meant: ");
    let index: usize = match user_in.parse() {
        Ok(n) => n,
        Err(_) => {
            println!("Error: invalid selection.");
            return None;
        }
    };

    match index.checked_sub(1).and_then(|i| matches.get(i)) {
        Some(e) => Some(*e),
        None => {
            println!("Error: invalid selection.");
            None
        }
    }
}

/// Shared by the top-level `search` command and the in-workout-log `search`:
/// pulls the search term and `-level`/`-equipment` flags out of already-parsed
/// args and runs `smart_search`. Returns the error message to print on bad
/// input, rather than printing it itself, so callers can return whatever
/// their own `ControlFlow` type needs after printing it.
fn run_search<'a>(library: &'a ExerciseLibrary, parsed_args: &[ArgType]) -> Result<Vec<ScoredExercise<'a>>, String> {
    let positionals: Vec<&str> = parsed_args
        .iter()
        .filter_map(|arg| match arg {
            ArgType::Positional(s) => Some(s.as_str()),
            _ => None,
        })
        .collect();

    if positionals.is_empty() {
        return Err("Error: expected a search term.".to_string());
    } else if positionals.len() > 1 {
        return Err("Error: multi-word search terms should be in quotations.".to_string());
    }

    let search_term = positionals.join(" ");

    // Adding a new filterable flag is one more line here (plus registering it
    // above in the `parse_args` call, and one more `&&` clause in the closure below).
    let level = get_flag_value::<Level>(parsed_args, "-level").map_err(|e| e.to_string())?;
    let equipment = get_flag_value::<Equipment>(parsed_args, "-equipment").map_err(|e| e.to_string())?;

    Ok(library.smart_search(&search_term, |exercise| {
        level.map_or(true, |l| exercise.level == l)
            && equipment.map_or(true, |e| exercise.equipment == Some(e))
    }))
}

/// Joins every positional in `parsed_args` back into one string, e.g. for a
/// command whose whole rest-of-line is one piece of free text (an exercise
/// name, a note). Handles `cmd word word` and `cmd "quoted phrase"` the same
/// way, since `parse_args` has already stripped the quotes off the latter.
/// Flags are silently dropped - callers of this don't accept any.
fn joined_positionals(parsed_args: &[ArgType]) -> String {
    parsed_args
        .iter()
        .filter_map(|arg| match arg {
            ArgType::Positional(s) => Some(s.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn print_search_results(results: Vec<ScoredExercise>) {
    if results.is_empty() {
        println!("No matches found.");
    }
    for (i, scored) in results.into_iter().enumerate() {
        println!("{:>3}: {}", i + 1, Exercise::short_display(scored.exercise));
    }
}

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
    commands.insert("log", handle_logging);

    commands
}

// Every handler below needs the exact signature of `Handler` to be storable
// in the table above. `handle_info` and `handle_help` are still `todo!()`
// placeholders (panic if called); everything else is implemented.

/// `info <exercise id or name>` - planned: look up and print one exercise in full.
/// Not implemented yet.
fn handle_info(args: &[&str], library: &ExerciseLibrary) -> ControlFlow {
    // args = exercise name (should only be ONE arg)
    let pargs = parse_args_or_return!(args, None, ControlFlow::Quit);

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
    if found_exercises.is_empty() {
        println!("Error: no exercises matched that name.");
        return ControlFlow::Continue;
    }

    if let Some(matching_exercise) = choose_exercise(&found_exercises) {
        println!("{matching_exercise}");
    }

    ControlFlow::Continue
}

/// `search <term>` (or `search "multi word term"`) - prints every exercise
/// whose name matches, ranked by [`ExerciseLibrary::smart_search`]'s match-quality
/// score (best matches first, printed as `score: N`, lower is better).
///
/// Accepts two optional value-taking flags to narrow results before ranking:
/// `-level <level>` and `-equipment <equipment>` (e.g.
/// `search curl -level beginner -equipment barbell`).
fn handle_search(args: &[&str], library: &ExerciseLibrary) -> ControlFlow {
    let parsed_args = parse_args_or_return!(args, Some(&["-level", "-equipment"]), ControlFlow::Continue);

    match run_search(library, &parsed_args) {
        Ok(results) => print_search_results(results),
        Err(msg) => println!("{msg}"),
    }

    ControlFlow::Continue
}

/// `muscle <muscle>` - lists exercises training the given muscle.
fn handle_muscle(args: &[&str], library: &ExerciseLibrary) -> ControlFlow {
    let parsed_args = parse_args_or_return!(args, None, ControlFlow::Continue);

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
    let parsed_args = parse_args_or_return!(args, None, ControlFlow::Continue);

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
    let parsed_args = parse_args_or_return!(args, None, ControlFlow::Continue);

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
    let parsed_args = parse_args_or_return!(args, None, ControlFlow::Continue);

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
    let parsed_args = parse_args_or_return!(args, None, ControlFlow::Continue);

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
    let parsed_args = parse_args_or_return!(args, None, ControlFlow::Continue);

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
    let rows: [(&str, &str); 13] = [
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
        ("log", "Start workout logging"),
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

pub enum LoggingControlFlow {
    Continue,
    Discard,
    End,
}

pub type LoggingHandler = fn(args: &[&str], library: &ExerciseLibrary, workout_log: &mut WorkoutLog) -> LoggingControlFlow;

/// Main loop to log workouts
fn handle_logging(args: &[&str], library: &ExerciseLibrary) -> ControlFlow {
    let commands_table = build_logging_command_table();
    let mut workout_log = WorkoutLog::new();
    println!("Workout logging begun. Type 'help' to see commands, 'end' to save, and 'discard' to discard the workout.");
    loop {
        let user_cmd: String = get_user_input("workout> ");
        let split_str: Vec<&str> = user_cmd.split_whitespace().collect();
        let args: &[&str] = &split_str[..];

        if args.is_empty() {
            continue;
        }

        let control_flow = match commands_table.get(args[0]) {
            Some(fxn) => fxn(&args[1..], &library, &mut workout_log),
            None => {
                println!("Unknown command: {}", args[0]);
                LoggingControlFlow::Continue
            }
        };

        if let LoggingControlFlow::Discard = control_flow {
            println!("Workout discarded.");
            break;
        }

        if let LoggingControlFlow::End = control_flow {
            println!("Workout ended.");
            workout_log.print_statistics();
            break;
        }
    }
    return ControlFlow::Continue
}

pub fn build_logging_command_table() -> HashMap<&'static str, LoggingHandler> {
    let mut commands: HashMap<&'static str, LoggingHandler> = HashMap::new();
    commands.insert("help", handle_log_help);
    commands.insert("discard", handle_log_discard);
    commands.insert("end", handle_log_end);
    commands.insert("search", handle_log_search);
    commands.insert("add", handle_log_add_exercise);
    commands.insert("remove", handle_log_remove_exercise);
    commands.insert("list", handle_log_list);
    commands.insert("show", handle_log_list);
    commands.insert("current", handle_log_current);
    commands.insert("next", handle_log_next);
    commands.insert("prev", handle_log_prev);
    commands.insert("goto", handle_log_goto);
    commands.insert("set", handle_log_add_set);
    commands.insert("complete", handle_log_complete_set);
    commands.insert("uncomplete", handle_log_uncomplete_set);
    commands.insert("delset", handle_log_delete_set);
    commands.insert("note", handle_log_note);
    commands.insert("name", handle_log_name);
    commands.insert("desc", handle_log_desc);
    commands
}

/// `help` - lists workout-logging commands.
fn handle_log_help(_args: &[&str], _library: &ExerciseLibrary, _workout_log: &mut WorkoutLog) -> LoggingControlFlow {
    let rows: [(&str, &str); 17] = [
        (
            "search <term> [-level L] [-equipment E]",
            "Find exercises by name, without adding one",
        ),
        ("add \"<exercise name>\"", "Add an exercise, and focus it"),
        ("remove [number]", "Remove an exercise (the focused one, if no number)"),
        ("list", "Show the whole workout"),
        ("current", "Show the focused exercise"),
        ("next / prev", "Move focus to the next/previous exercise"),
        ("goto <number>", "Focus an exercise by its number in \"list\""),
        (
            "set <weight> <reps> [-warmup|-failure|-drop]",
            "Log a set on the focused exercise",
        ),
        ("complete <set#> [weight] [reps]", "Mark a set complete, optionally correcting it"),
        ("uncomplete <set#>", "Mark a set not yet performed"),
        ("delset <set#>", "Delete a set"),
        ("note <text>", "Set a note on the focused exercise"),
        ("name <text>", "Name the workout"),
        ("desc <text>", "Describe the workout"),
        ("end", "Save and finish the workout"),
        ("discard", "Discard the workout"),
        ("help", "Show this message"),
    ];

    println!("Workout logging commands:");
    for (syntax, description) in rows {
        println!("  {syntax:<46} {description}");
    }

    LoggingControlFlow::Continue
}

/// `search <term> [-level L] [-equipment E]` - same ranked search as the
/// top-level `search` command, for looking an exercise up by name without
/// leaving the workout (and without adding it - see `add` for that).
fn handle_log_search(args: &[&str], library: &ExerciseLibrary, _workout_log: &mut WorkoutLog) -> LoggingControlFlow {
    let parsed_args = parse_args_or_return!(args, Some(&["-level", "-equipment"]), LoggingControlFlow::Continue);

    match run_search(library, &parsed_args) {
        Ok(results) => print_search_results(results),
        Err(msg) => println!("{msg}"),
    }

    LoggingControlFlow::Continue
}

/// `add "<exercise name>"` - searches the library by name (disambiguating like
/// `info` does if more than one exercise matches), adds it to the workout, and
/// focuses it so the next `set`/`note` acts on it without naming it again.
fn handle_log_add_exercise(args: &[&str], library: &ExerciseLibrary, workout_log: &mut WorkoutLog) -> LoggingControlFlow {
    let parsed_args = parse_args_or_return!(args, None, LoggingControlFlow::Continue);
    let search_term = joined_positionals(&parsed_args);

    if search_term.is_empty() {
        println!("Error: expected an exercise name.");
        println!("Usage: add \"exercise name\"");
        return LoggingControlFlow::Continue;
    }

    let found_exercises = library.find_by_name(&search_term);

    if found_exercises.is_empty() {
        println!("Error: no exercises matched that name.");
        return LoggingControlFlow::Continue;
    }

    let exercise = match choose_exercise(&found_exercises) {
        Some(e) => e,
        None => return LoggingControlFlow::Continue,
    };
    let exercise_id = exercise.id.clone();
    let exercise_name = exercise.name.clone();

    workout_log.add_exercise(exercise_id);
    println!("Added \"{exercise_name}\" as exercise {}.", workout_log.exercises.len());

    LoggingControlFlow::Continue
}

/// `remove [number]` - removes the exercise at `number` (as shown by `list`),
/// or the focused exercise if no number is given.
fn handle_log_remove_exercise(args: &[&str], library: &ExerciseLibrary, workout_log: &mut WorkoutLog) -> LoggingControlFlow {
    if workout_log.exercises.is_empty() {
        println!("Error: no exercises in this workout yet.");
        return LoggingControlFlow::Continue;
    }

    let index = if args.is_empty() {
        match workout_log.current_index() {
            Some(i) => i,
            None => {
                println!("Error: no focused exercise. Usage: remove <number>");
                return LoggingControlFlow::Continue;
            }
        }
    } else {
        match args[0].parse::<usize>() {
            Ok(n) if n >= 1 && n <= workout_log.exercises.len() => n - 1,
            _ => {
                println!("Error: invalid exercise number. Use \"list\" to see numbers.");
                return LoggingControlFlow::Continue;
            }
        }
    };

    let exercise_id = workout_log.exercises[index].exercise_id.clone();
    let exercise_name = library.catalog.get(&exercise_id).map(|e| e.name.as_str()).unwrap_or(&exercise_id);
    println!("Removed \"{exercise_name}\".");
    workout_log.remove_exercise_at(index);

    LoggingControlFlow::Continue
}

/// Shared by `current` and by `next`/`prev`/`goto` to show where focus landed.
fn print_current_exercise(library: &ExerciseLibrary, workout_log: &WorkoutLog) {
    match workout_log.current_index() {
        None => println!("No focused exercise. Use \"add <exercise name>\" first."),
        Some(i) => print_exercise_detail(library, &workout_log.exercises[i], i),
    }
}

fn print_exercise_detail(library: &ExerciseLibrary, exercise: &LoggedExercise, index: usize) {
    let name = library
        .catalog
        .get(&exercise.exercise_id)
        .map(|e| e.name.as_str())
        .unwrap_or(&exercise.exercise_id);
    println!("{:>3}. {}", index + 1, name);
    if !exercise.note.is_empty() {
        println!("     note: {}", exercise.note);
    }
    if exercise.sets.is_empty() {
        println!("     (no sets logged)");
    } else {
        for (j, set) in exercise.sets.iter().enumerate() {
            println!("     {:>2}. {}", j + 1, set);
        }
    }
}

/// `list`/`show` - prints the whole workout: every exercise (marking the
/// focused one with `>`) and its sets.
fn handle_log_list(_args: &[&str], library: &ExerciseLibrary, workout_log: &mut WorkoutLog) -> LoggingControlFlow {
    let name = if workout_log.name.is_empty() { "(unnamed workout)" } else { &workout_log.name };
    println!("{name}");

    if workout_log.exercises.is_empty() {
        println!("  No exercises logged yet. Use \"add <exercise name>\" to start.");
        return LoggingControlFlow::Continue;
    }

    for (i, exercise) in workout_log.exercises.iter().enumerate() {
        let marker = if workout_log.current_index() == Some(i) { ">" } else { " " };
        print!("{marker}");
        print_exercise_detail(library, exercise, i);
    }

    LoggingControlFlow::Continue
}

/// `current` - shows just the focused exercise and its sets.
fn handle_log_current(_args: &[&str], library: &ExerciseLibrary, workout_log: &mut WorkoutLog) -> LoggingControlFlow {
    print_current_exercise(library, workout_log);
    LoggingControlFlow::Continue
}

/// `next` - focuses the exercise after the current one.
fn handle_log_next(_args: &[&str], library: &ExerciseLibrary, workout_log: &mut WorkoutLog) -> LoggingControlFlow {
    if workout_log.exercises.is_empty() {
        println!("Error: no exercises in this workout yet.");
        return LoggingControlFlow::Continue;
    }
    if !workout_log.next_exercise() {
        println!("Already at the last exercise.");
        return LoggingControlFlow::Continue;
    }
    print_current_exercise(library, workout_log);
    LoggingControlFlow::Continue
}

/// `prev` - focuses the exercise before the current one.
fn handle_log_prev(_args: &[&str], library: &ExerciseLibrary, workout_log: &mut WorkoutLog) -> LoggingControlFlow {
    if workout_log.exercises.is_empty() {
        println!("Error: no exercises in this workout yet.");
        return LoggingControlFlow::Continue;
    }
    if !workout_log.prev_exercise() {
        println!("Already at the first exercise.");
        return LoggingControlFlow::Continue;
    }
    print_current_exercise(library, workout_log);
    LoggingControlFlow::Continue
}

/// `goto <number>` - focuses the exercise at that position in `list`.
fn handle_log_goto(args: &[&str], library: &ExerciseLibrary, workout_log: &mut WorkoutLog) -> LoggingControlFlow {
    let arg = match args.first() {
        Some(a) => a,
        None => {
            println!("Usage: goto <exercise number>");
            return LoggingControlFlow::Continue;
        }
    };

    let index = match arg.parse::<usize>() {
        Ok(n) if n >= 1 => n - 1,
        _ => {
            println!("Error: invalid exercise number.");
            return LoggingControlFlow::Continue;
        }
    };

    if !workout_log.set_current(index) {
        println!("Error: no exercise number {arg}. Use \"list\" to see numbers.");
        return LoggingControlFlow::Continue;
    }

    print_current_exercise(library, workout_log);
    LoggingControlFlow::Continue
}

/// `set <weight> <reps> [-warmup|-failure|-drop]` - logs a new set on the
/// focused exercise. New sets start uncompleted - use `complete` once the set
/// is actually performed, so a planned set and a performed one are distinct.
fn handle_log_add_set(args: &[&str], _library: &ExerciseLibrary, workout_log: &mut WorkoutLog) -> LoggingControlFlow {
    if workout_log.current_index().is_none() {
        println!("Error: no focused exercise. Use \"add <exercise name>\" first.");
        return LoggingControlFlow::Continue;
    }

    let parsed_args = parse_args_or_return!(args, None, LoggingControlFlow::Continue);

    let positionals: Vec<&str> = parsed_args
        .iter()
        .filter_map(|arg| match arg {
            ArgType::Positional(s) => Some(s.as_str()),
            _ => None,
        })
        .collect();

    if positionals.len() != 2 {
        println!("Error: expected a weight and a rep count.");
        println!("Usage: set <weight> <reps> [-warmup] [-failure] [-drop]");
        return LoggingControlFlow::Continue;
    }

    let weight: f32 = match positionals[0].parse() {
        Ok(w) => w,
        Err(_) => {
            println!("Error: invalid weight.");
            return LoggingControlFlow::Continue;
        }
    };
    let reps: u32 = match positionals[1].parse() {
        Ok(r) => r,
        Err(_) => {
            println!("Error: invalid rep count.");
            return LoggingControlFlow::Continue;
        }
    };

    let has_flag = |name: &str| parsed_args.iter().any(|a| matches!(a, ArgType::Flag(f) if f.as_str() == name));
    let kind = if has_flag("-warmup") {
        SetType::Warmup
    } else if has_flag("-failure") {
        SetType::Failure
    } else if has_flag("-drop") {
        SetType::Drop
    } else {
        SetType::Normal
    };

    let exercise = workout_log.current_exercise_mut().expect("checked above");
    exercise.log_set(weight, reps, kind);
    println!("Logged set {}: {weight} x {reps} ({kind}).", exercise.sets.len());

    LoggingControlFlow::Continue
}

/// `complete <set#> [weight] [reps]` - marks a set on the focused exercise as
/// performed. With just a set number, keeps the weight/reps it was logged
/// with; with all three, overwrites them with what was actually performed.
fn handle_log_complete_set(args: &[&str], _library: &ExerciseLibrary, workout_log: &mut WorkoutLog) -> LoggingControlFlow {
    if args.is_empty() {
        println!("Usage: complete <set number> [weight] [reps]");
        return LoggingControlFlow::Continue;
    }

    let set_num: usize = match args[0].parse() {
        Ok(n) => n,
        Err(_) => {
            println!("Error: invalid set number.");
            return LoggingControlFlow::Continue;
        }
    };

    let (weight, reps) = match args.len() {
        1 => (None, None),
        3 => {
            let w: f32 = match args[1].parse() {
                Ok(w) => w,
                Err(_) => {
                    println!("Error: invalid weight.");
                    return LoggingControlFlow::Continue;
                }
            };
            let r: u32 = match args[2].parse() {
                Ok(r) => r,
                Err(_) => {
                    println!("Error: invalid rep count.");
                    return LoggingControlFlow::Continue;
                }
            };
            (Some(w), Some(r))
        }
        _ => {
            println!("Error: provide either just a set number, or a set number with both weight and reps.");
            println!("Usage: complete <set number> [weight] [reps]");
            return LoggingControlFlow::Continue;
        }
    };

    let exercise = match workout_log.current_exercise_mut() {
        Some(e) => e,
        None => {
            println!("Error: no focused exercise.");
            return LoggingControlFlow::Continue;
        }
    };

    if !exercise.complete_set(set_num, weight, reps) {
        println!("Error: no set number {set_num}. Use \"current\" to see sets.");
        return LoggingControlFlow::Continue;
    }

    println!("Set {set_num} marked complete.");
    LoggingControlFlow::Continue
}

/// `uncomplete <set#>` - undoes `complete` (e.g. logged by mistake).
fn handle_log_uncomplete_set(args: &[&str], _library: &ExerciseLibrary, workout_log: &mut WorkoutLog) -> LoggingControlFlow {
    let set_num: usize = match args.first().and_then(|a| a.parse().ok()) {
        Some(n) => n,
        None => {
            println!("Usage: uncomplete <set number>");
            return LoggingControlFlow::Continue;
        }
    };

    let exercise = match workout_log.current_exercise_mut() {
        Some(e) => e,
        None => {
            println!("Error: no focused exercise.");
            return LoggingControlFlow::Continue;
        }
    };

    if !exercise.uncomplete_set(set_num) {
        println!("Error: no set number {set_num}.");
        return LoggingControlFlow::Continue;
    }

    println!("Set {set_num} marked incomplete.");
    LoggingControlFlow::Continue
}

/// `delset <set#>` - deletes a set from the focused exercise.
fn handle_log_delete_set(args: &[&str], _library: &ExerciseLibrary, workout_log: &mut WorkoutLog) -> LoggingControlFlow {
    let set_num: usize = match args.first().and_then(|a| a.parse().ok()) {
        Some(n) => n,
        None => {
            println!("Usage: delset <set number>");
            return LoggingControlFlow::Continue;
        }
    };

    let exercise = match workout_log.current_exercise_mut() {
        Some(e) => e,
        None => {
            println!("Error: no focused exercise.");
            return LoggingControlFlow::Continue;
        }
    };

    if !exercise.delete_set(set_num) {
        println!("Error: no set number {set_num}.");
        return LoggingControlFlow::Continue;
    }

    println!("Deleted set {set_num}.");
    LoggingControlFlow::Continue
}

/// `note <text>` - sets a free-text note on the focused exercise.
fn handle_log_note(args: &[&str], _library: &ExerciseLibrary, workout_log: &mut WorkoutLog) -> LoggingControlFlow {
    let parsed_args = parse_args_or_return!(args, None, LoggingControlFlow::Continue);
    let text = joined_positionals(&parsed_args);

    if text.is_empty() {
        println!("Usage: note <text>");
        return LoggingControlFlow::Continue;
    }

    let exercise = match workout_log.current_exercise_mut() {
        Some(e) => e,
        None => {
            println!("Error: no focused exercise. Use \"add <exercise name>\" first.");
            return LoggingControlFlow::Continue;
        }
    };

    exercise.update_note(text);
    println!("Note updated.");
    LoggingControlFlow::Continue
}

/// `name <text>` - names the workout (also asked for by `end` if still unset).
fn handle_log_name(args: &[&str], _library: &ExerciseLibrary, workout_log: &mut WorkoutLog) -> LoggingControlFlow {
    let parsed_args = parse_args_or_return!(args, None, LoggingControlFlow::Continue);
    let text = joined_positionals(&parsed_args);

    if text.is_empty() {
        println!("Usage: name <workout name>");
        return LoggingControlFlow::Continue;
    }
    workout_log.update_name(text);
    println!("Workout name set to \"{}\".", workout_log.name);
    LoggingControlFlow::Continue
}

/// `desc <text>` - describes the workout (also asked for by `end` if still unset).
fn handle_log_desc(args: &[&str], _library: &ExerciseLibrary, workout_log: &mut WorkoutLog) -> LoggingControlFlow {
    let parsed_args = parse_args_or_return!(args, None, LoggingControlFlow::Continue);
    let text = joined_positionals(&parsed_args);

    if text.is_empty() {
        println!("Usage: desc <description>");
        return LoggingControlFlow::Continue;
    }
    workout_log.update_description(text);
    println!("Description updated.");
    LoggingControlFlow::Continue
}

/// `end` - saves and finishes the workout. Refuses (staying in the logging
/// loop) if no exercises were ever added, since `WorkoutLog::finish` itself
/// just no-ops in that case rather than reporting why.
fn handle_log_end(_args: &[&str], _library: &ExerciseLibrary, workout_log: &mut WorkoutLog) -> LoggingControlFlow {
    if workout_log.exercises.is_empty() {
        println!("Error: add at least one exercise before ending the workout.");
        return LoggingControlFlow::Continue;
    }

    if workout_log.name.is_empty() {
        let name = get_user_input("Enter workout name: ");
        workout_log.update_name(name);
    }

    if workout_log.description.is_empty() {
        let description = get_user_input("Enter workout description: ");
        workout_log.update_description(description);
    }

    workout_log.finish();

    LoggingControlFlow::End
}

/// `discard` - ends the logging loop without saving.
fn handle_log_discard(_args: &[&str], _library: &ExerciseLibrary, _workout_log: &mut WorkoutLog) -> LoggingControlFlow {
    LoggingControlFlow::Discard
}
