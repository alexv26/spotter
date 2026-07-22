use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::hash::Hash;
use std::path::Path;
use std::string::String;

use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Force {
    Static,
    Pull,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mechanic {
    Isolation,
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
    // so you can do Muscle:ALL. For example:
    // for muscle in Muscle::ALL
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
    /// a `Muscle`. Case-insensitive, since CLI input shouldn't have to match exactly.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "abdominals" => Ok(Muscle::Abdominals),
            "abductors" => Ok(Muscle::Abductors),
            "adductors" => Ok(Muscle::Adductors),
            "biceps" => Ok(Muscle::Biceps),
            "calves" => Ok(Muscle::Calves),
            "chest" => Ok(Muscle::Chest),
            "forearms" => Ok(Muscle::Forearms),
            "glutes" => Ok(Muscle::Glutes),
            "hamstrings" => Ok(Muscle::Hamstrings),
            "lats" => Ok(Muscle::Lats),
            "lower back" | "lowerback" => Ok(Muscle::LowerBack),
            "middle back" | "middleback" => Ok(Muscle::MiddleBack),
            "neck" => Ok(Muscle::Neck),
            "quadriceps" => Ok(Muscle::Quadriceps),
            "shoulders" => Ok(Muscle::Shoulders),
            "traps" => Ok(Muscle::Traps),
            "triceps" => Ok(Muscle::Triceps),
            other => Err(format!("unknown muscle: {other}")),
        }
    }
}

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

pub struct ExerciseLibrary {
    pub catalog: HashMap<String, Exercise>,
    pub num_exercises: usize,
    pub by_primary_muscle: HashMap<Muscle, Vec<String>>,
    pub by_secondary_muscle: HashMap<Muscle, Vec<String>>,
    pub by_equipment: HashMap<Equipment, Vec<String>>,
    pub by_mechanic: HashMap<Mechanic, Vec<String>>,
    pub by_category: HashMap<Category, Vec<String>>,
    pub by_level: HashMap<Level, Vec<String>>,
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
        let mut results = self.find_by_primary_muscle(muscle);
        for exercise in self.find_by_secondary_muscle(muscle) {
            if !results.iter().any(|e| e.id == exercise.id) {
                results.push(exercise);
            }
        }
        results
    }

    pub fn find_by_equipment(&self, equipment: Equipment) -> Vec<&Exercise> {
        self.resolve(self.by_equipment.get(&equipment))
    }

    pub fn find_by_mechanic(&self, mechanic: Mechanic) -> Vec<&Exercise> {
        self.resolve(self.by_mechanic.get(&mechanic))
    }

    pub fn find_by_category(&self, category: Category) -> Vec<&Exercise> {
        self.resolve(self.by_category.get(&category))
    }

    pub fn find_by_level(&self, level: Level) -> Vec<&Exercise> {
        self.resolve(self.by_level.get(&level))
    }

    pub fn find_by_force(&self, force: Force) -> Vec<&Exercise> {
        self.resolve(self.by_force.get(&force))
    }

    /// Finds all exercises whose name contains the search key.
    pub fn find_by_name(&self, search: &str) -> Vec<&Exercise> {
        let search = search.to_lowercase();
        self.catalog
            .values()
            .filter(|exercise| exercise.name.to_lowercase().contains(&search))
            .collect()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Exercise {
    pub id: String,
    pub name: String,
    pub force: Option<Force>,
    pub level: Level,
    pub mechanic: Option<Mechanic>,
    pub equipment: Option<Equipment>,
    pub primary_muscles: Vec<Muscle>,
    pub secondary_muscles: Vec<Muscle>,
    pub instructions: Vec<String>,
    pub category: Category,
    pub images: Vec<String>,
}

// Exercise methods
impl Exercise {
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

/// Loads every exercise from free-exercise-db's separated .json exercise files`.
/// Path = path to folder containing all .json files.
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

// MATCH TIERS:
// 1. Exact match
// 2. Prefix match: starts with search term
// 3. Whole-word match: search term appears as one of the name's words.
// 4. Substring match: the term appears somewhee in the string.
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

pub struct ScoredExercise<'a> {
    pub exercise: &'a Exercise,
    pub score: u32,
}

impl ExerciseLibrary {
    /// Searches by name, ranking results by match quality (see MATCH TIERS above)
    /// instead of leaving them in whatever order `find_by_name` happened to return.
    pub fn smart_search(&self, search_term: &str) -> Vec<ScoredExercise<'_>> {
        let exercises_found = self.find_by_name(search_term);
        let mut similarity_scores: Vec<ScoredExercise> = Vec::new();

        for exercise in exercises_found {
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
