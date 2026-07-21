use std::collections::HashMap;

use spotter_core::exercise::ExerciseLibrary;

/// Tells the main loop whether to keep reading commands or shut down.
pub enum ControlFlow {
    Continue,
    Quit,
}

pub type Handler = fn(args: &[&str], library: &ExerciseLibrary) -> ControlFlow;

/// Maps a command name (the first word the user types) to its handler.
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
    commands
}

fn handle_info(args: &[&str], library: &ExerciseLibrary) -> ControlFlow {
    todo!()
}

fn handle_search(args: &[&str], library: &ExerciseLibrary) -> ControlFlow {
    todo!()
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

fn handle_quit(args: &[&str], library: &ExerciseLibrary) -> ControlFlow {
    todo!()
}
