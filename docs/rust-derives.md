# Common Derive Macros in Rust

"Deriving" a trait means the compiler auto-generates the implementation for
you, based purely on the shape of your struct/enum, instead of you writing it
by hand. You do it by listing traits inside `#[derive(...)]` right above the
type definition:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Muscle { ... }
```

Below are the ones you'll run into constantly, roughly ordered from
"almost always include this" to "situational."

## Debug

Enables `{:?}` (and `{:#?}` for pretty/multi-line) formatting in
`println!`/`format!`/etc. Prints the type name plus every field's value,
recursively.

```rust
println!("{:?}", Time { hours: 1, minutes: 30, seconds: 0 });
// Time { hours: 1, minutes: 30, seconds: 0 }
```

- Without it, `{:?}` on that type is a compile error — there's no default.
- `assert_eq!` requires **both** `PartialEq` (to compare) and `Debug` (to
  print `left`/`right` in the failure message) on the compared type.
- It's essentially free and doesn't change real behavior — include it on
  almost everything you define.
- Contrast with `Display` (`{}`, enabled by hand-writing `impl fmt::Display`,
  not derivable): `Debug` is for *you*, the programmer, inspecting a value;
  `Display` is for the *user* reading your program's actual output. That's
  why `Exercise` has a hand-written `Display` for its nicely formatted block,
  but would still separately benefit from a derived `Debug`.

## Clone

Adds a `.clone()` method that produces an independent, owned copy of the
value. Every field must itself implement `Clone` (a struct made of
`Clone` fields can derive it automatically).

- Needed whenever you want a second, independent copy of something instead
  of moving/losing access to the original — e.g. `exercise.id.clone()`,
  used constantly in `ExerciseLibrary::from_catalog` because the same `id`
  needs to end up in several different index `Vec<String>`s.

## Copy

Marks a type as "silently duplicated on assignment/move" instead of moved —
so using a `Copy` value doesn't consume the original.

- Requires `Clone` (`Copy` is really just "and also, do this implicitly").
- **Every field must also be `Copy`.** You cannot derive `Copy` on anything
  containing a `String`, `Vec`, or other heap-owned data — only plain,
  fixed-size, stack-only data (integers, bools, chars, small structs/enums
  built entirely from `Copy` fields).
- This is exactly why `Muscle`, `Force`, `Level`, etc. all derive `Copy` in
  this project: it's what lets code like
  `by_primary_muscle.entry(*muscle)...` work — dereferencing `*muscle`
  copies the `Muscle` out of the shared reference instead of trying
  (illegally) to move it out.

## PartialEq

Enables `==` and `!=`. Compares field-by-field (structs) or variant-by-variant
(enums) by default.

- "Partial" because equality isn't guaranteed to be total for every possible
  value — the classic example is floating point `NaN != NaN`. Any type
  containing an `f32`/`f64` field can still derive `PartialEq`, just not the
  stricter `Eq` below.
- Needed for `.contains()`, `assert_eq!`, and `==` generally.

## Eq

A marker trait (no methods) that promises `PartialEq` is *fully* reflexive —
every value equals itself, no exceptions.

- Because of that guarantee, **you cannot derive `Eq` on a type containing
  `f32`/`f64`** — `NaN` breaks the promise. This is precisely why, in this
  project, `SetLog` (which holds `weight: f32`) only derives `Debug`, while
  `Time` and `SetType` (no floats) derive the full
  `Debug, Clone, Copy, PartialEq, Eq` set.
- Required (alongside `Hash`) to use a type as a `HashMap`/`HashSet` key.

## Hash

Lets the compiler compute a hash for the type — required to use it as a key
in `HashMap`/`HashSet`, or store it in a `HashSet`.

- Needs `PartialEq`/`Eq` alongside it (equal values must hash identically).
- Why `Muscle`, `Equipment`, `Category`, `Level`, `Force`, `Mechanic` all
  derive `Hash` in this project: they're every one of them used as
  `HashMap` keys in `ExerciseLibrary`'s `by_*` indices
  (`HashMap<Muscle, Vec<String>>`, etc.).

## PartialOrd / Ord

Enable `<`, `>`, `<=`, `>=` (`PartialOrd`) and a full, total ordering
(`Ord`) — needed to `.sort()` a `Vec` of the type directly, or use it in a
`BTreeMap`/`BTreeSet`.

- `Ord` requires `Eq` + `PartialOrd` (same float caveat as `Eq` applies).
- Handy detail: a derived `Ord` on an enum orders variants by **declaration
  order** — the first variant listed is the "smallest." `Level`'s variants
  are already declared `Beginner, Intermediate, Expert` — deriving `Ord`
  on it would give you `Beginner < Intermediate < Expert` sorting for free,
  with zero extra code.

## Default

Adds a `Type::default()` function that builds a reasonable "zero value" —
`0` for numbers, `""` for `String`, an empty `Vec`, `None` for `Option`, and
so on, recursively for structs. For enums, one variant needs an explicit
`#[default]` attribute.

- Useful for "give me a blank starting value" without listing every field
  by hand — e.g. a fresh, empty `WorkoutLog` builder.

## Serialize / Deserialize (from the `serde` crate, not `std`)

Not a standard-library derive — comes from the external `serde` crate
(already a dependency here). Requires `use serde::{Serialize, Deserialize};`.

- **`Deserialize`**: lets a type be *built from* an external format. This is
  how `Exercise` gets populated straight from free-exercise-db's JSON files
  via `serde_json::from_str`.
- **`Serialize`**: the reverse — converts a type *into* an external format
  (e.g. saving a `WorkoutLog` to a JSON file). Not used yet in this project,
  but it's the derive you'd reach for once workout logs need to persist
  between runs.
- Both are customizable with attributes you've already used a lot here —
  `#[serde(rename = "...")]` and `#[serde(rename_all = "...")]` on the
  `Equipment`/`Muscle`/`Category` enums, to map Rust variant names onto the
  exact strings the JSON data uses.

## Quick rules of thumb

- `Copy` requires `Clone`.
- `Eq` requires `PartialEq`.
- `Ord` requires `PartialOrd` + `Eq`.
- `Hash` is almost always paired with `Eq` (`HashMap` keys need both).
- Floating point fields (`f32`/`f64`) block `Eq`, `Ord`, and `Hash` — only
  `Debug`, `Clone`, `Copy`, and `PartialEq`/`PartialOrd` are available for
  types that contain them.
