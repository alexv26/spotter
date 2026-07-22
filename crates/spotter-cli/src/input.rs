//! Turns raw terminal input into structured command arguments: reading a line
//! ([`getUserInput`]), tokenizing it while respecting quoted phrases
//! ([`parseArgs`], [`nextArg`]), and extracting a specific flag's value out of
//! the result ([`get_flag_value`]). Nothing here knows about specific commands
//! or exercise data - that's `commands.rs`'s job, one layer up.

use std::io::{self, Write};
use std::str::FromStr;

/// Prints `cli_msg` as a prompt (no trailing newline) and reads one line from stdin.
pub fn getUserInput(cli_msg: String) -> String {
    let mut input = String::new();

    print!("{cli_msg}");
    let _ = io::stdout().flush();
    // Read the line from standard input
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    // Remove the trailing newline character (\n)
    input.trim().to_string()
}

/// One parsed command-line argument, already classified so callers don't have
/// to re-inspect a plain `String` to figure out what kind of argument it was.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ArgType {
    /// A bare value with no leading flag - a plain word, or an entire quoted
    /// phrase joined into one (e.g. `curl`, or `Hello there` from `"Hello there"`).
    Positional(String),
    /// A flag with no value attached (e.g. `-v` for verbose).
    Flag(String),
    /// A flag paired with the value that followed it (e.g. `-level beginner`
    /// becomes `flag: "-level"`, `value: "beginner"`), for flags registered as
    /// value-taking via `parseArgs`'s `flags_with_values` parameter.
    Option { flag: String, value: String },
}

/// Finds `name` (e.g. "-level") among already-parsed args and parses its value
/// as `T`. Returns `Ok(None)` if the flag wasn't present at all, and `Err` only
/// if it was present with a value that doesn't parse as `T`. One call per flag
/// replaces the find-then-match-then-parse block a new flag would otherwise need.
pub fn get_flag_value<T: FromStr>(args: &[ArgType], name: &str) -> Result<Option<T>, T::Err> {
    let value = args.iter().find_map(|arg| match arg {
        ArgType::Option { flag, value } if flag == name => Some(value),
        _ => None,
    });

    match value {
        Some(v) => v.parse::<T>().map(Some),
        None => Ok(None),
    }
}

/// Reads the argument at `*i`, updating `*i` to the last token it consumed.
/// A plain word is a single token; a quoted phrase (`"like this"`) can span
/// several whitespace-split tokens, which get joined back into one string here.
/// Callers position `*i` at whatever they want read - e.g. `parseArgs`
/// advances past a flag before calling this, so it reads the flag's value.
pub fn nextArg(args: &[&str], i: &mut usize) -> Result<String, String> {
    let j = *i;
    if j >= args.len() {
        println!("Error: invalid argument provided.");
        return Err(String::from("invalid argument provided"));
    }

    // Argument is a string
    if args[j].starts_with('\"') {
        // Already closed in this same token, e.g. "hello" - nothing more to scan for.
        if args[j].len() > 1 && args[j].ends_with('\"') {
            *i = j;
            return Ok(args[j].trim_matches('\"').to_string());
        }

        let mut new_pos = j;
        let mut terms: Vec<&str> = Vec::new();
        terms.push(args[j].trim_matches('\"'));
        for k in j + 1..args.len() {
            let arg = args[k];
            let word: &str = arg.trim_matches('\"');
            terms.push(word);
            new_pos = k;
            if arg.contains('\"') {
                break;
            }
        }
        let joined_term = terms.join(" ");
        *i = new_pos;
        return Ok(joined_term);
    } else {
        *i = j;
        return Ok(args[j].to_string());
    }
}

/// Classifies each raw, whitespace-split token in `args` into an [`ArgType`]:
/// a `-`-prefixed token becomes a [`ArgType::Flag`], or an [`ArgType::Option`]
/// if it's listed in `flags_with_values` (in which case the following token is
/// consumed as its value); anything else becomes an [`ArgType::Positional`],
/// with quoted multi-word phrases joined into one via [`nextArg`].
///
/// `flags_with_values` is `None` for commands that take no value-taking flags
/// at all (every other `-`-prefixed token is still parsed as a boolean `Flag`).
pub fn parseArgs(
    args: &[&str],
    flags_with_values: Option<&[&str]>,
) -> Result<Vec<ArgType>, String> {
    let mut new_args: Vec<ArgType> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i];

        // Handling flags
        if arg.starts_with('-') {
            // Option 1. Flag with value
            if flags_with_values != None && flags_with_values.unwrap_or_default().contains(&arg) {
                if i + 1 >= args.len() {
                    return Err(format!("Flag {} missing value", &arg));
                } else if args[i + 1].contains('-') {
                    return Err(format!("Flag {} needs an argument", &arg));
                }
                i += 1;
                let next_arg: String = match nextArg(args, &mut i) {
                    Ok(val) => val,
                    Err(err) => {
                        return Err(err);
                    }
                };
                new_args.push(ArgType::Option {
                    flag: arg.to_string(),
                    value: next_arg,
                });
                i += 1;
            } else {
                new_args.push(ArgType::Flag(arg.to_string()));
                i += 1;
            }
        } else {
            // Positional
            let next_arg: String = match nextArg(args, &mut i) {
                Ok(val) => val,
                Err(err) => {
                    return Err(err);
                }
            };
            new_args.push(ArgType::Positional(next_arg));
            i += 1;
        }
    }
    Ok(new_args)
}
