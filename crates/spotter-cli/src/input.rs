use std::io::{self, Write};

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ArgType {
    Positional(String),
    Flag(String),
    Option { flag: String, value: String },
}

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

pub fn parseArgs(args: &[&str], flags_with_values: Option<&[&str]>) -> Result<Vec<ArgType>, String> {
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
