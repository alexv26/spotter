//! Exercise data model and lookup/search over the free-exercise-db catalog.
//!
//! [`Exercise`] mirrors one entry from the free-exercise-db JSON files, with its
//! enum-valued fields (`force`, `level`, `mechanic`, `equipment`, `category`,
//! and the two muscle lists) parsed via `serde` into the enums defined here
//! ([`Force`], [`Level`], [`Mechanic`], [`Equipment`], [`Muscle`], [`Category`])
//! instead of being left as raw strings. [`ExerciseLibrary`] loads the whole
//! catalog once and builds indices over those enums so lookups like
//! "every exercise for `Muscle::Biceps`" don't require scanning every exercise.

use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::hash::Hash;
use std::path::Path;
use std::string::String;

use serde::Deserialize;

/// The type of muscular contraction an exercise trains: pushing, pulling, or holding still.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Force {
    /// The muscle contracts without the joint moving (e.g. a plank).
    Static,
    /// The resistance moves toward the body (e.g. a row or curl).
    Pull,
    /// The resistance moves away from the body (e.g. a press or squat).
    Push,
}

impl Force {
    pub fn as_str(&self) -> &'static str {
        match self {
            Force::Static => "static",
            Force::Pull => "pull",
            Force::Push => "push",
        }
    }
}

impl fmt::Display for Force {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Force {
    type Err = String;

    /// The reverse of `as_str`: turns user-typed text like "push" back into a `Force`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "static" => Ok(Force::Static),
            "pull" => Ok(Force::Pull),
            "push" => Ok(Force::Push),
            other => Err(format!("unknown force: {other}")),
        }
    }
}

/// How much lifting experience an exercise assumes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Beginner,
    Intermediate,
    Expert,
}

impl Level {
    pub fn as_str(&self) -> &'static str {
        match self {
            Level::Beginner => "beginner",
            Level::Intermediate => "intermediate",
            Level::Expert => "expert",
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Level {
    type Err = String;

    /// The reverse of `as_str`: turns user-typed text like "beginner" back into a `Level`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "beginner" => Ok(Level::Beginner),
            "intermediate" => Ok(Level::Intermediate),
            "expert" => Ok(Level::Expert),
            other => Err(format!("unknown level: {other}")),
        }
    }
}

/// Whether an exercise moves one joint (isolation) or several at once (compound).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mechanic {
    /// Moves a single joint, targeting one muscle group (e.g. a bicep curl).
    Isolation,
    /// Moves multiple joints at once, targeting several muscle groups (e.g. a squat).
    Compound,
}

impl Mechanic {
    pub fn as_str(&self) -> &'static str {
        match self {
            Mechanic::Isolation => "isolation",
            Mechanic::Compound => "compound",
        }
    }
}

impl fmt::Display for Mechanic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Mechanic {
    type Err = String;

    /// The reverse of `as_str`: turns user-typed text like "compound" back into a `Mechanic`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "isolation" => Ok(Mechanic::Isolation),
            "compound" => Ok(Mechanic::Compound),
            other => Err(format!("unknown mechanic: {other}")),
        }
    }
}

/// The equipment (if any) an exercise requires.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum Equipment {
    #[serde(rename = "medicine ball")]
    MedicineBall,
    #[serde(rename = "dumbbell")]
    Dumbbell,
    #[serde(rename = "body only")]
    BodyOnly,
    #[serde(rename = "bands")]
    Bands,
    #[serde(rename = "kettlebells")]
    Kettlebells,
    #[serde(rename = "foam roll")]
    FoamRoll,
    #[serde(rename = "cable")]
    Cable,
    #[serde(rename = "machine")]
    Machine,
    #[serde(rename = "barbell")]
    Barbell,
    #[serde(rename = "exercise ball")]
    ExerciseBall,
    #[serde(rename = "e-z curl bar")]
    EZCurlBar,
    #[serde(rename = "other")]
    Other,
}

impl Equipment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Equipment::MedicineBall => "medicine ball",
            Equipment::Dumbbell => "dumbbell",
            Equipment::BodyOnly => "body only",
            Equipment::Bands => "bands",
            Equipment::Kettlebells => "kettlebells",
            Equipment::FoamRoll => "foam roll",
            Equipment::Cable => "cable",
            Equipment::Machine => "machine",
            Equipment::Barbell => "barbell",
            Equipment::ExerciseBall => "exercise ball",
            Equipment::EZCurlBar => "e-z curl bar",
            Equipment::Other => "other",
        }
    }
}

impl fmt::Display for Equipment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Equipment {
    type Err = String;

    /// The reverse of `as_str`: turns user-typed text like "barbell" back into an `Equipment`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "medicine ball" => Ok(Equipment::MedicineBall),
            "dumbbell" => Ok(Equipment::Dumbbell),
            "body only" => Ok(Equipment::BodyOnly),
            "bands" => Ok(Equipment::Bands),
            "kettlebells" => Ok(Equipment::Kettlebells),
            "foam roll" => Ok(Equipment::FoamRoll),
            "cable" => Ok(Equipment::Cable),
            "machine" => Ok(Equipment::Machine),
            "barbell" => Ok(Equipment::Barbell),
            "exercise ball" => Ok(Equipment::ExerciseBall),
            "e-z curl bar" => Ok(Equipment::EZCurlBar),
            "other" => Ok(Equipment::Other),
            other => Err(format!("unknown equipment: {other}")),
        }
    }
}

/// A muscle group an exercise can target, as either a primary or secondary mover.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum Muscle {
    #[serde(rename = "abdominals")]
    Abdominals,
    #[serde(rename = "abductors")]
    Abductors,
    #[serde(rename = "adductors")]
    Adductors,
    #[serde(rename = "biceps")]
    Biceps,
    #[serde(rename = "calves")]
    Calves,
    #[serde(rename = "chest")]
    Chest,
    #[serde(rename = "forearms")]
    Forearms,
    #[serde(rename = "glutes")]
    Glutes,
    #[serde(rename = "hamstrings")]
    Hamstrings,
    #[serde(rename = "lats")]
    Lats,
    #[serde(rename = "lower back")]
    LowerBack,
    #[serde(rename = "middle back")]
    MiddleBack,
    #[serde(rename = "neck")]
    Neck,
    #[serde(rename = "quadriceps")]
    Quadriceps,
    #[serde(rename = "shoulders")]
    Shoulders,
    #[serde(rename = "traps")]
    Traps,
    #[serde(rename = "triceps")]
    Triceps,
}

impl Muscle {
    /// Every `Muscle` variant, for iterating over all of them, e.g. `for muscle in Muscle::ALL`.
    /// Hand-maintained: adding or removing a variant means updating this list (and its length) too.
    pub const ALL: [Muscle; 17] = [
        Muscle::Abdominals,
        Muscle::Abductors,
        Muscle::Adductors,
        Muscle::Biceps,
        Muscle::Calves,
        Muscle::Chest,
        Muscle::Forearms,
        Muscle::Glutes,
        Muscle::Hamstrings,
        Muscle::Lats,
        Muscle::LowerBack,
        Muscle::MiddleBack,
        Muscle::Neck,
        Muscle::Quadriceps,
        Muscle::Shoulders,
        Muscle::Traps,
        Muscle::Triceps,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Muscle::Abdominals => "abdominals",
            Muscle::Abductors => "abductors",
            Muscle::Adductors => "adductors",
            Muscle::Biceps => "biceps",
            Muscle::Calves => "calves",
            Muscle::Chest => "chest",
            Muscle::Forearms => "forearms",
            Muscle::Glutes => "glutes",
            Muscle::Hamstrings => "hamstrings",
            Muscle::Lats => "lats",
            Muscle::LowerBack => "lower back",
            Muscle::MiddleBack => "middle back",
            Muscle::Neck => "neck",
            Muscle::Quadriceps => "quadriceps",
            Muscle::Shoulders => "shoulders",
            Muscle::Traps => "traps",
            Muscle::Triceps => "triceps",
        }
    }
}

impl fmt::Display for Muscle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Muscle {
    type Err = String;

    /// The reverse of `as_str`: turns user-typed text like "biceps" back into
    /// a `Muscle`. Case-insensitive, and tolerates a trailing "s" mismatch
    /// either way via `words_match` - e.g. "bicep" also matches `Muscle::Biceps`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_lowercase();

        for muscle in Muscle::ALL {
            if words_match(&lower, muscle.as_str()) {
                return Ok(muscle);
            }
        }

        match lower.as_str() {
            "lowerback" => Ok(Muscle::LowerBack),
            "middleback" => Ok(Muscle::MiddleBack),
            _ => Err(format!("unknown muscle: {s}")),
        }
    }
}

/// The broad style of training an exercise belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum Category {
    #[serde(rename = "powerlifting")]
    Powerlifting,
    #[serde(rename = "strength")]
    Strength,
    #[serde(rename = "stretching")]
    Stretching,
    #[serde(rename = "cardio")]
    Cardio,
    #[serde(rename = "olympic weightlifting")]
    OlympicWeightlifting,
    #[serde(rename = "strongman")]
    Strongman,
    #[serde(rename = "plyometrics")]
    Plyometrics,
}

impl Category {
    pub fn as_str(&self) -> &'static str {
        match self {
            Category::Powerlifting => "powerlifting",
            Category::Strength => "strength",
            Category::Stretching => "stretching",
            Category::Cardio => "cardio",
            Category::OlympicWeightlifting => "olympic weightlifting",
            Category::Strongman => "strongman",
            Category::Plyometrics => "plyometrics",
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Category {
    type Err = String;

    /// The reverse of `as_str`: turns user-typed text like "strength" back into a `Category`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "powerlifting" => Ok(Category::Powerlifting),
            "strength" => Ok(Category::Strength),
            "stretching" => Ok(Category::Stretching),
            "cardio" => Ok(Category::Cardio),
            "olympic weightlifting" => Ok(Category::OlympicWeightlifting),
            "strongman" => Ok(Category::Strongman),
            "plyometrics" => Ok(Category::Plyometrics),
            other => Err(format!("unknown category: {other}")),
        }
    }
}

/// The full exercise catalog plus indices for looking it up by enum field
/// (muscle, equipment, category, ...) without scanning every exercise.
///
/// Each index stores exercise `id`s rather than the exercises themselves or a
/// shared pointer to them — looking a result up costs one extra `catalog` hash
/// lookup, but avoids either cloning every `Exercise` into every index it
/// belongs to, or the self-referential-struct problem a `&Exercise`/`Rc<Exercise>`
/// index would run into by pointing back into `catalog` on the same struct.
pub struct ExerciseLibrary {
    /// Every exercise, keyed by its `id`. The source of truth all the `by_*` indices resolve through.
    pub catalog: HashMap<String, Exercise>,
    /// `catalog.len()`, cached so callers don't need to compute it themselves.
    pub num_exercises: usize,
    /// Exercise ids, keyed by each `Muscle` they train as a *primary* mover.
    pub by_primary_muscle: HashMap<Muscle, Vec<String>>,
    /// Exercise ids, keyed by each `Muscle` they train as a *secondary/assisting* mover.
    pub by_secondary_muscle: HashMap<Muscle, Vec<String>>,
    /// Exercise ids, keyed by required `Equipment`.
    pub by_equipment: HashMap<Equipment, Vec<String>>,
    /// Exercise ids, keyed by `Mechanic` (isolation vs. compound).
    pub by_mechanic: HashMap<Mechanic, Vec<String>>,
    /// Exercise ids, keyed by `Category` (strength, cardio, ...).
    pub by_category: HashMap<Category, Vec<String>>,
    /// Exercise ids, keyed by `Level` (beginner, intermediate, expert).
    pub by_level: HashMap<Level, Vec<String>>,
    /// Exercise ids, keyed by `Force` (push, pull, static).
    pub by_force: HashMap<Force, Vec<String>>,
}

impl ExerciseLibrary {
    /// Loads every exercise from free-exercise-db's separated .json exercise files
    /// and builds the search indices alongside the catalog.
    pub fn load(path: &Path) -> Result<ExerciseLibrary, Box<dyn std::error::Error>> {
        let catalog = load_exercises(path)?;
        Ok(ExerciseLibrary::from_catalog(catalog))
    }

    /// Builds a library (catalog + search indices) from an already-loaded catalog.
    /// Each index stores the exercise's `id`, resolved back through `catalog`
    /// when a lookup happens, rather than a shared pointer to the exercise itself.
    pub fn from_catalog(catalog: HashMap<String, Exercise>) -> ExerciseLibrary {
        let mut by_primary_muscle: HashMap<Muscle, Vec<String>> = HashMap::new();
        let mut by_secondary_muscle: HashMap<Muscle, Vec<String>> = HashMap::new();
        let mut by_equipment: HashMap<Equipment, Vec<String>> = HashMap::new();
        let mut by_mechanic: HashMap<Mechanic, Vec<String>> = HashMap::new();
        let mut by_category: HashMap<Category, Vec<String>> = HashMap::new();
        let mut by_level: HashMap<Level, Vec<String>> = HashMap::new();
        let mut by_force: HashMap<Force, Vec<String>> = HashMap::new();

        for exercise in catalog.values() {
            for muscle in &exercise.primary_muscles {
                by_primary_muscle
                    .entry(*muscle)
                    .or_default()
                    .push(exercise.id.clone());
            }
            for muscle in &exercise.secondary_muscles {
                by_secondary_muscle
                    .entry(*muscle)
                    .or_default()
                    .push(exercise.id.clone());
            }
            if let Some(equipment) = exercise.equipment {
                by_equipment
                    .entry(equipment)
                    .or_default()
                    .push(exercise.id.clone());
            }
            if let Some(mechanic) = exercise.mechanic {
                by_mechanic
                    .entry(mechanic)
                    .or_default()
                    .push(exercise.id.clone());
            }
            by_category
                .entry(exercise.category)
                .or_default()
                .push(exercise.id.clone());
            by_level
                .entry(exercise.level)
                .or_default()
                .push(exercise.id.clone());
            if let Some(force) = exercise.force {
                by_force.entry(force).or_default().push(exercise.id.clone());
            }
        }

        let num_exercises = catalog.len();

        ExerciseLibrary {
            catalog,
            num_exercises,
            by_primary_muscle,
            by_secondary_muscle,
            by_equipment,
            by_mechanic,
            by_category,
            by_level,
            by_force,
        }
    }

    /// Turns a list of exercise ids (as stored in the `by_*` indices) into the
    /// actual `Exercise`s, via `catalog`. `None` (index key not present) and an
    /// empty list are both treated as "no results" rather than an error.
    fn resolve(&self, ids: Option<&Vec<String>>) -> Vec<&Exercise> {
        ids.map(|ids| ids.iter().filter_map(|id| self.catalog.get(id)).collect())
            .unwrap_or_default()
    }

    /// Every exercise that trains `muscle` as a primary mover.
    pub fn find_by_primary_muscle(&self, muscle: Muscle) -> Vec<&Exercise> {
        self.resolve(self.by_primary_muscle.get(&muscle))
    }

    /// Every exercise that trains `muscle` as a secondary/assisting mover.
    pub fn find_by_secondary_muscle(&self, muscle: Muscle) -> Vec<&Exercise> {
        self.resolve(self.by_secondary_muscle.get(&muscle))
    }

    /// Every exercise that trains `muscle`, primary or secondary.
    pub fn find_by_muscle(&self, muscle: Muscle) -> Vec<&Exercise> {
        self.resolve(self.by_primary_muscle.get(&muscle))
    }

    /// Every exercise that requires `equipment`.
    pub fn find_by_equipment(&self, equipment: Equipment) -> Vec<&Exercise> {
        self.resolve(self.by_equipment.get(&equipment))
    }

    /// Every exercise with the given `mechanic` (isolation vs. compound).
    pub fn find_by_mechanic(&self, mechanic: Mechanic) -> Vec<&Exercise> {
        self.resolve(self.by_mechanic.get(&mechanic))
    }

    /// Every exercise in the given `category` (strength, cardio, ...).
    pub fn find_by_category(&self, category: Category) -> Vec<&Exercise> {
        self.resolve(self.by_category.get(&category))
    }

    /// Every exercise at the given `level` (beginner, intermediate, expert).
    pub fn find_by_level(&self, level: Level) -> Vec<&Exercise> {
        self.resolve(self.by_level.get(&level))
    }

    /// Every exercise with the given `force` (push, pull, static).
    pub fn find_by_force(&self, force: Force) -> Vec<&Exercise> {
        self.resolve(self.by_force.get(&force))
    }

    /// Finds all exercises whose name contains the search key (case-insensitive).
    /// Results aren't ranked by relevance - see [`ExerciseLibrary::smart_search`] for that.
    pub fn find_by_name(&self, search: &str) -> Vec<&Exercise> {
        let search = search.to_lowercase();
        self.catalog
            .values()
            .filter(|exercise| exercise.name.to_lowercase().contains(&search))
            .collect()
    }
}

/// One exercise, deserialized directly from a free-exercise-db JSON entry.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Exercise {
    /// Stable identifier matching the exercise's filename in free-exercise-db (e.g. `"Barbell_Curl"`).
    pub id: String,
    /// Human-readable name (e.g. `"Barbell Curl"`).
    pub name: String,
    /// Push, pull, or static - `None` if the source data didn't specify one.
    pub force: Option<Force>,
    pub level: Level,
    /// Isolation vs. compound - `None` if the source data didn't specify one.
    pub mechanic: Option<Mechanic>,
    /// Required equipment, if any.
    pub equipment: Option<Equipment>,
    pub primary_muscles: Vec<Muscle>,
    /// Muscles trained as assisting movers; can be empty.
    pub secondary_muscles: Vec<Muscle>,
    /// Step-by-step instructions, in order.
    pub instructions: Vec<String>,
    pub category: Category,
    /// Relative image paths (e.g. `"Barbell_Curl/0.jpg"`), as shipped by free-exercise-db.
    pub images: Vec<String>,
}

impl Exercise {
    /// A single-line summary (name, primary muscles, level) for listing many
    /// exercises at once - e.g. search results - where the full boxed `Display`
    /// output would be too long to scan.
    pub fn short_display(&self) -> String {
        let primary_muscles = self
            .primary_muscles
            .iter()
            .map(Muscle::as_str)
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "{:<60} Primary Muscles: {:<20} Level: {}",
            self.name, primary_muscles, self.level
        )
    }
}

impl fmt::Display for Exercise {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let force = self.force.map(|f| f.as_str()).unwrap_or("none");
        let mechanic = self.mechanic.map(|m| m.as_str()).unwrap_or("none");
        let equipment = self.equipment.map(|e| e.as_str()).unwrap_or("none");
        let primary_muscles = self
            .primary_muscles
            .iter()
            .map(Muscle::as_str)
            .collect::<Vec<_>>()
            .join(", ");
        let secondary_muscles = if self.secondary_muscles.is_empty() {
            "none".to_string()
        } else {
            self.secondary_muscles
                .iter()
                .map(Muscle::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        };
        let instructions = self
            .instructions
            .iter()
            .enumerate()
            .map(|(i, step)| format!("  {}. {}", i + 1, step))
            .collect::<Vec<_>>()
            .join("\n");

        let title = format!(" {} ", self.name);
        let border = "═".repeat(title.chars().count().max(24));

        writeln!(f, "{border}")?;
        writeln!(f, "{title}")?;
        writeln!(f, "{border}")?;
        writeln!(f, "{:<11}{}", "Level:", self.level)?;
        writeln!(f, "{:<11}{}", "Category:", self.category)?;
        writeln!(f, "{:<11}{}", "Force:", force)?;
        writeln!(f, "{:<11}{}", "Mechanic:", mechanic)?;
        writeln!(f, "{:<11}{}", "Equipment:", equipment)?;
        writeln!(f, "{:<11}{}", "Primary:", primary_muscles)?;
        writeln!(f, "{:<11}{}", "Secondary:", secondary_muscles)?;
        writeln!(f)?;
        writeln!(f, "Instructions:")?;
        write!(f, "{instructions}")
    }
}

/// Loads every exercise from free-exercise-db's per-exercise `.json` files.
///
/// `path` is the directory containing those files (e.g. free-exercise-db's
/// `exercises/` folder) - non-`.json` entries and subdirectories are skipped.
/// Exercises are keyed by their `id` field, not their filename.
fn load_exercises(path: &Path) -> Result<HashMap<String, Exercise>, Box<dyn std::error::Error>> {
    let mut exercises: HashMap<String, Exercise> = HashMap::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_path = entry.path();

        // Skip directories
        if !file_path.is_file() {
            continue;
        }

        // Only process .json files
        // .extension: "file.json" -> "json"
        if file_path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let raw = fs::read_to_string(&file_path)?;
        let exercise: Exercise = serde_json::from_str(&raw)?;

        exercises.insert(exercise.id.clone(), exercise);
    }
    Ok(exercises)
}

/// Treats two words as equal even if one is just the other with a trailing
/// "s" added, so a search for "curl" also recognizes "curls" as the same word.
fn words_match(a: &str, b: &str) -> bool {
    a == b || format!("{a}s") == b || format!("{b}s") == a
}

/// Scores how well `e_name` matches search term `n`, lower is a better match:
///
/// 1. Exact match - `e_name` equals `n` outright.
/// 2. Prefix match - `e_name` starts with `n`, word for word.
/// 3. Whole-word match - `n` appears as a run of consecutive whole words
///    somewhere in `e_name` (not necessarily at the start).
/// 4. Substring match - `n` appears somewhere in `e_name`, but not aligned to
///    word boundaries (everything [`ExerciseLibrary::find_by_name`] treats as
///    equally good already gets this tier).
///
/// Word comparisons tolerate a trailing "s" mismatch in either direction (see
/// the private `words_match` helper below), so plurals like "curls" count as
/// the same word as "curl".
pub fn search_similarity_score(n: &str, e_name: &str) -> u32 {
    let search_term: String = n.to_lowercase();
    let e_name: String = e_name.to_lowercase();
    let split_ename: Vec<&str> = e_name.split_whitespace().collect();
    let split_search_term: Vec<&str> = search_term.split_whitespace().collect();
    let len_search_term = split_search_term.len();

    // 1. Exact match
    if search_term == e_name {
        return 1;
    }

    // 2. Prefix match:
    let mut pos = 0;
    let mut words_equal = 0;
    for word in &split_ename {
        if pos < len_search_term && words_match(word, split_search_term[pos]) {
            words_equal += 1;
        } else {
            break;
        }

        if words_equal == len_search_term {
            return 2;
        }

        pos += 1;
    }

    // whole-word match: does the search phrase appear as a run of consecutive
    // whole words starting at ANY position in the name, not just position 0?
    if len_search_term > 0
        && split_ename.windows(len_search_term).any(|window| {
            window
                .iter()
                .zip(split_search_term.iter())
                .all(|(a, b)| words_match(a, b))
        })
    {
        return 3;
    }

    return 4;
}

/// One [`ExerciseLibrary::smart_search`] result: an exercise paired with its
/// [`search_similarity_score`] (lower is a better match).
pub struct ScoredExercise<'a> {
    pub exercise: &'a Exercise,
    pub score: u32,
}

impl ExerciseLibrary {
    /// Searches by name, ranking results by match quality (see MATCH TIERS above)
    /// instead of leaving them in whatever order `find_by_name` happened to return.
    /// `filter` decides whether a candidate exercise stays in the results at all,
    /// before match-quality scoring - e.g. `|e| e.level == Level::Beginner`.
    /// This never needs to change to support a new filterable flag: whoever
    /// calls `smart_search` builds whatever filter logic it needs and passes it
    /// in, so this function's signature stays the same no matter how many
    /// flags get added on the caller's side.
    pub fn smart_search(
        &self,
        search_term: &str,
        filter: impl Fn(&Exercise) -> bool,
    ) -> Vec<ScoredExercise<'_>> {
        let exercises_found = self.find_by_name(search_term);
        let mut similarity_scores: Vec<ScoredExercise> = Vec::new();

        for exercise in exercises_found {
            if !filter(exercise) {
                continue;
            }

            let score = search_similarity_score(search_term, &exercise.name);
            similarity_scores.push(ScoredExercise { exercise, score });
        }
        similarity_scores.sort_by_key(|scored| scored.score);
        similarity_scores
    }
}

/// Counts how many exercises target each primary muscle.
pub fn count_by_primary_muscle(exercises: &[Exercise]) -> HashMap<Muscle, usize> {
    let mut counts = HashMap::new();
    for exercise in exercises {
        for muscle in &exercise.primary_muscles {
            *counts.entry(*muscle).or_insert(0) += 1;
        }
    }
    counts
}

/// Returns every exercise that needs no equipment at all.
pub fn bodyweight_only(exercises: &[Exercise]) -> Vec<&Exercise> {
    exercises
        .iter()
        .filter(|e| e.equipment == Some(Equipment::BodyOnly))
        .collect()
}
