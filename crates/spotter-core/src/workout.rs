//! Workout logging

use std::fmt;
use std::time::{SystemTime, Duration};

#[derive(Debug)]
pub struct WorkoutLog {
    pub name: String,
    pub description: String,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub duration: Duration,
    pub exercises: Vec<LoggedExercise>,
    /// Index into `exercises` that commands like `set`/`complete`/`note` act on
    /// by default, so the CLI (and eventually a UI) doesn't need to name an
    /// exercise on every single command. `None` only when `exercises` is empty.
    pub current_exercise: Option<usize>,
}

/// Methods for WorkoutLog:
/// 1. Add exercise(s)
/// 2. Remove exercise(s)
/// 3. Add workout description
/// 4. Update duration
/// 5. Name workout
/// 6. Discard

#[derive(Debug, PartialEq, Eq)]
pub enum WorkoutState {
    Continue,
    End,
}

impl WorkoutLog {
    pub fn new() -> WorkoutLog {
        WorkoutLog {
            name: String::new(),
            description: String::new(),
            start_time: SystemTime::now(),
            end_time: None,
            duration: Duration::ZERO,
            exercises: Vec::new(),
            current_exercise: None,
        }
    }

    pub fn finish(&mut self) -> WorkoutState {
        if self.exercises.is_empty() {
            return WorkoutState::Continue;
        }
        let now = SystemTime::now();
        self.end_time = Some(now);
        self.duration = now.duration_since(self.start_time).unwrap_or(Duration::ZERO);
        WorkoutState::End
    }

    pub fn print_statistics(&self) {
        println!("Name: {}", self.name);
        println!("Description: {}", self.description);
        let total_secs = self.duration.as_secs();
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        println!("Duration: {hours}h {minutes}m");
    }

    /// Discards the workout. Consumes `self` so the log can't be used or saved afterward.
    /// Unlike `finish`, this always succeeds - there's no precondition to fail.
    pub fn discard(self) -> WorkoutState {
        WorkoutState::End
    }

    /// Adds the exercise and focuses it, so a bare `set`/`note` command right
    /// after `add` acts on the exercise that was just added.
    pub fn add_exercise(&mut self, exercise_id : String) {
        self.exercises.push(LoggedExercise::new(exercise_id));
        self.current_exercise = Some(self.exercises.len() - 1);
    }

    pub fn add_exercises(&mut self, exercise_ids : Vec<String>) {
        for eid in exercise_ids {
            self.add_exercise(eid);
        }
    }

    //todo: Note: will not allow an exercise to be logged twice. if you log it twice and delete it will delete both.
    pub fn remove_exercise(&mut self, exercise_id : String) {
        self.exercises.retain(|e| e.exercise_id != exercise_id);
        self.clamp_current();
    }

    pub fn remove_exercises(&mut self, exercise_ids : Vec<String>) {
        for eid in exercise_ids {
            self.remove_exercise(eid);
        }
    }

    /// Removes the exercise at `index` (0-based). Preferred over
    /// `remove_exercise` when the caller already knows the position (e.g. from
    /// a numbered `list` display), since that sidesteps the duplicate-name
    /// ambiguity `remove_exercise` has. Returns `false` if `index` is out of bounds.
    pub fn remove_exercise_at(&mut self, index: usize) -> bool {
        if index >= self.exercises.len() {
            return false;
        }
        self.exercises.remove(index);
        self.clamp_current();
        true
    }

    /// Keeps `current_exercise` pointing at a valid index after the exercise
    /// list shrinks - `None` if it's now empty, otherwise clamped to the last
    /// remaining exercise if it pointed past the end.
    fn clamp_current(&mut self) {
        if self.exercises.is_empty() {
            self.current_exercise = None;
        } else if let Some(i) = self.current_exercise {
            if i >= self.exercises.len() {
                self.current_exercise = Some(self.exercises.len() - 1);
            }
        }
    }

    /// Finds the logged exercise with the given id, if it's in this workout,
    /// so callers can log/delete sets on it (e.g. `log.get_exercise_mut(id).map(|e| e.log_set(...))`).
    pub fn get_exercise_mut(&mut self, exercise_id: &str) -> Option<&mut LoggedExercise> {
        self.exercises.iter_mut().find(|e| e.exercise_id == exercise_id)
    }

    /// 0-based index of the exercise that bare `set`/`note`/`complete` commands
    /// act on. `None` only when the workout has no exercises yet.
    pub fn current_index(&self) -> Option<usize> {
        self.current_exercise
    }

    pub fn current_exercise_mut(&mut self) -> Option<&mut LoggedExercise> {
        self.current_exercise.and_then(move |i| self.exercises.get_mut(i))
    }

    /// Focuses the exercise at `index` (0-based). Returns `false` (leaving the
    /// cursor unchanged) if `index` is out of bounds.
    pub fn set_current(&mut self, index: usize) -> bool {
        if index < self.exercises.len() {
            self.current_exercise = Some(index);
            true
        } else {
            false
        }
    }

    /// Moves focus to the next exercise. Returns `false` (leaving the cursor
    /// unchanged) if already on the last one.
    pub fn next_exercise(&mut self) -> bool {
        match self.current_exercise {
            Some(i) if i + 1 < self.exercises.len() => {
                self.current_exercise = Some(i + 1);
                true
            }
            _ => false,
        }
    }

    /// Moves focus to the previous exercise. Returns `false` (leaving the
    /// cursor unchanged) if already on the first one.
    pub fn prev_exercise(&mut self) -> bool {
        match self.current_exercise {
            Some(i) if i > 0 => {
                self.current_exercise = Some(i - 1);
                true
            }
            _ => false,
        }
    }

    pub fn update_description(&mut self, description: String) {
        self.description = description;
    }

    pub fn update_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn update_duration(&mut self) {
        let now = SystemTime::now();
        let duration: Duration = now.duration_since(self.start_time).unwrap_or(Duration::ZERO);
        self.duration = duration;
    }
}

#[derive(Debug)]
pub struct LoggedExercise {
    pub exercise_id: String,
    pub sets: Vec<SetLog>,
    pub note: String,

}

impl LoggedExercise {
    pub fn new(exercise_id: String) -> LoggedExercise {
        LoggedExercise {
            exercise_id,
            sets: Vec::new(),
            note: String::new(),
        }
    }

    /// Records one performed set with real values, rather than a blank placeholder.
    pub fn log_set(&mut self, weight: f32, reps: u32, kind: SetType) {
        let new_set = SetLog::new(weight, reps, kind);
        self.sets.push(new_set);
    }

    /// Deletes set number `set_num` (1-based, matching how sets are shown to
    /// the user - "set 1" is `self.sets[0]`). Returns `false` instead of
    /// panicking if `set_num` doesn't refer to an existing set.
    pub fn delete_set(&mut self, set_num: usize) -> bool {
        if set_num == 0 || set_num > self.sets.len() {
            return false;
        }
        self.sets.remove(set_num - 1);
        true
    }

    /// Set number `set_num` (1-based, see `delete_set`), if it exists.
    pub fn get_set_mut(&mut self, set_num: usize) -> Option<&mut SetLog> {
        if set_num == 0 {
            return None;
        }
        self.sets.get_mut(set_num - 1)
    }

    /// Marks set `set_num` completed, optionally overwriting its weight/reps
    /// with what was actually performed (vs. whatever it was logged/planned
    /// with). Returns `false` if `set_num` doesn't refer to an existing set.
    pub fn complete_set(&mut self, set_num: usize, weight: Option<f32>, reps: Option<u32>) -> bool {
        match self.get_set_mut(set_num) {
            Some(set) => {
                if let Some(w) = weight {
                    set.weight = w;
                }
                if let Some(r) = reps {
                    set.reps = r;
                }
                set.completed = true;
                true
            }
            None => false,
        }
    }

    /// Undoes `complete_set` - marks set `set_num` as not yet performed.
    pub fn uncomplete_set(&mut self, set_num: usize) -> bool {
        match self.get_set_mut(set_num) {
            Some(set) => {
                set.completed = false;
                true
            }
            None => false,
        }
    }

    pub fn update_note(&mut self, note: String) {
        self.note = note;
    }
}

#[derive(Debug)]
pub struct SetLog {
    pub weight: f32,
    pub reps: u32,
    pub time: SetTime,
    pub kind: SetType,
    /// Whether this set has actually been performed, vs. still being a
    /// logged target waiting on `WorkoutLog::complete_set`/`LoggedExercise::complete_set`.
    pub completed: bool,
}

impl SetLog {
    pub fn new(weight: f32, reps: u32, kind: SetType) -> SetLog {
        SetLog {
            weight,
            reps,
            time: SetTime { hours: 0, minutes: 0, seconds: 0 },
            kind,
            completed: false,
        }
    }
}

impl fmt::Display for SetLog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let check = if self.completed { "x" } else { " " };
        write!(f, "[{check}] {:>6.1} x {:>3} ({})", self.weight, self.reps, self.kind)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetTime {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SetType {
    Warmup,
    Failure,
    Drop,
    Normal,
}

impl SetType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SetType::Warmup => "warmup",
            SetType::Failure => "failure",
            SetType::Drop => "drop",
            SetType::Normal => "normal",
        }
    }
}

impl fmt::Display for SetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for SetType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "warmup" => Ok(SetType::Warmup),
            "failure" => Ok(SetType::Failure),
            "drop" => Ok(SetType::Drop),
            "normal" => Ok(SetType::Normal),
            other => Err(format!("unknown set type: {other}")),
        }
    }
}
