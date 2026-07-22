# Spotter 🏋️ — A Gym Companion

Most workout trackers (Strong, Hevy, Fitbod, ...) are great at logging sets and
reps, but they assume you already know what you're doing. Spotter is built for
the lifter who doesn't — someone new to the gym who needs guidance and a
reason to keep showing up, not just a spreadsheet. The long-term goal is a
tracker that teaches as you use it, and gamifies progress (leveling up, not
just numbers going up) so early progress feels rewarding even before it's
visible in the mirror.

Currently a CLI, built in Rust, with Python planned for future
recommendation/analysis features.

## Project layout

```
spotter/
├── crates/
│   ├── spotter-core/   # domain logic: Exercise data model, ExerciseLibrary lookups/search
│   └── spotter-cli/    # the CLI itself: input loop + command dispatch
└── data/
    └── free-exercise-db/   # exercise data (git submodule)
```

`spotter-core` has no CLI-specific code in it — it's a plain library crate, so
future interfaces (a web/mobile API server, for instance) can reuse the same
exercise data model and search logic without depending on the CLI at all.

## Setup

Clone with submodules (the exercise data lives in a separate repo):

```sh
git clone --recurse-submodules <this-repo-url>
```

Already cloned without `--recurse-submodules`? Pull the data in after the fact:

```sh
git submodule update --init
```

## Running it

From the repository root (the app loads exercise data via a path relative to
the repo root, so it won't find it if you `cd` into a crate directory first):

```sh
cargo run -p spotter-cli
```

This drops you into an interactive prompt (`$`). A few commands that work today:

| Command | What it does |
|---|---|
| `search <term>` | Finds exercises by name, best matches first. Wrap multi-word terms in quotes: `search "barbell curl"` |
| `random` | Shows a random exercise. `random <muscle>` (e.g. `random biceps`) narrows it to one muscle |
| `clear` | Clears the terminal |
| `quit` / `exit` | Exits |

Several other commands (`info`, `muscle`, `equipment`, `category`, `level`,
`help`) are wired into the command table but not implemented yet — typing them
will panic with a `not yet implemented` message. That's expected; they're next.

## Documentation

Every public type and function in `spotter-core` and `spotter-cli` has doc
comments. Build and open them in your browser with:

```sh
cargo doc --no-deps --open
```

(`--no-deps` skips generating docs for every third-party dependency too, so you
just get Spotter's own docs.)

## Building

```sh
cargo build
```

Builds every crate in the workspace. Run `cargo build -p spotter-core` or
`cargo build -p spotter-cli` to build just one.
