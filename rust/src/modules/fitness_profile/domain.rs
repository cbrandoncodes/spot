use std::{collections::HashSet, str::FromStr};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const QUESTIONNAIRE_VERSION: i32 = 1;
pub const SNAPSHOT_SCHEMA_VERSION: i32 = 1;
pub const PRIVACY_NOTICE_VERSION: &str = "fitness-data-v1";

macro_rules! string_enum {
    ($name:ident, $field:literal, { $($variant:ident => $value:literal),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $value),+
                }
            }
        }

        impl FromStr for $name {
            type Err = ValidationError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match value {
                    $($value => Ok(Self::$variant),)+
                    _ => Err(ValidationError::new($field, "contains an unsupported value")),
                }
            }
        }
    };
}

string_enum!(
    Goal,
    "goals",
    {
        GeneralFitness => "general_fitness",
        BuildMuscle => "build_muscle",
        GetStronger => "get_stronger",
        ImproveConditioning => "improve_conditioning",
        ImproveMobility => "improve_mobility",
        AthleticPerformance => "athletic_performance",
    }
);

string_enum!(
    ExperienceLevel,
    "experienceLevel",
    {
        NeverTrained => "never_trained",
        Beginner => "beginner",
        Intermediate => "intermediate",
        Advanced => "advanced",
    }
);

string_enum!(
    ActivityLevel,
    "currentActivityLevel",
    {
        MostlyInactive => "mostly_inactive",
        LightlyActive => "lightly_active",
        Active => "active",
        VeryActive => "very_active",
    }
);

string_enum!(
    TrainingEnvironment,
    "trainingEnvironment",
    {
        Gym => "gym",
        Home => "home",
        Mixed => "mixed",
    }
);

string_enum!(
    Equipment,
    "availableEquipment",
    {
        Bodyweight => "bodyweight",
        Dumbbells => "dumbbells",
        AdjustableBench => "adjustable_bench",
        Barbell => "barbell",
        WeightPlates => "weight_plates",
        SquatRack => "squat_rack",
        CableStation => "cable_station",
        LatPulldown => "lat_pulldown",
        LegPress => "leg_press",
        SmithMachine => "smith_machine",
        ResistanceBands => "resistance_bands",
        Kettlebells => "kettlebells",
        PullUpBar => "pull_up_bar",
        CardioMachines => "cardio_machines",
    }
);

string_enum!(
    WorkoutType,
    "preferredWorkoutTypes",
    {
        Strength => "strength",
        Hypertrophy => "hypertrophy",
        Conditioning => "conditioning",
        Bodyweight => "bodyweight",
        Mobility => "mobility",
        Mixed => "mixed",
    }
);

string_enum!(
    CoachingStyle,
    "coachingStyle",
    {
        CalmSupportive => "calm_supportive",
        Motivational => "motivational",
        Direct => "direct",
        Technical => "technical",
        Balanced => "balanced",
    }
);

string_enum!(
    StartingIntensity,
    "startingIntensity",
    {
        Gentle => "gentle",
        Balanced => "balanced",
        Challenging => "challenging",
    }
);

string_enum!(
    AvoidancePreference,
    "dislikedExercises",
    {
        PreferAlternative => "prefer_alternative",
        NeverInclude => "never_include",
    }
);

string_enum!(
    BodyRegion,
    "bodyRegion",
    {
        Neck => "neck",
        Shoulder => "shoulder",
        Elbow => "elbow",
        WristHand => "wrist_hand",
        UpperBack => "upper_back",
        LowerBack => "lower_back",
        Hip => "hip",
        Knee => "knee",
        AnkleFoot => "ankle_foot",
        Other => "other",
    }
);

string_enum!(
    MovementPattern,
    "movementPattern",
    {
        Squat => "squat",
        Hinge => "hinge",
        Lunge => "lunge",
        HorizontalPush => "horizontal_push",
        VerticalPush => "vertical_push",
        HorizontalPull => "horizontal_pull",
        VerticalPull => "vertical_pull",
        Carry => "carry",
        Rotation => "rotation",
        Running => "running",
        Jumping => "jumping",
    }
);

string_enum!(
    ConstraintImpact,
    "impact",
    {
        Monitor => "monitor",
        Modify => "modify",
        Avoid => "avoid",
    }
);

string_enum!(
    UnitSystem,
    "unitSystem",
    {
        Metric => "metric",
        Imperial => "imperial",
    }
);

string_enum!(
    Weekday,
    "preferredWeekdays",
    {
        Monday => "monday",
        Tuesday => "tuesday",
        Wednesday => "wednesday",
        Thursday => "thursday",
        Friday => "friday",
        Saturday => "saturday",
        Sunday => "sunday",
    }
);

impl Weekday {
    pub const fn number(self) -> i16 {
        match self {
            Self::Monday => 1,
            Self::Tuesday => 2,
            Self::Wednesday => 3,
            Self::Thursday => 4,
            Self::Friday => 5,
            Self::Saturday => 6,
            Self::Sunday => 7,
        }
    }

    pub fn from_number(value: i16) -> Result<Self, ValidationError> {
        Self::ALL
            .iter()
            .copied()
            .find(|weekday| weekday.number() == value)
            .ok_or_else(|| ValidationError::new("preferredWeekdays", "contains an invalid day"))
    }
}

pub const EXERCISE_OPTIONS: &[(&str, &str)] = &[
    ("barbell_back_squat", "Barbell back squat"),
    ("deadlift", "Deadlift"),
    ("bench_press", "Bench press"),
    ("overhead_press", "Overhead press"),
    ("pull_up", "Pull-up"),
    ("push_up", "Push-up"),
    ("lunge", "Lunge"),
    ("hip_thrust", "Hip thrust"),
    ("burpee", "Burpee"),
    ("running", "Running"),
];

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ExerciseCode(String);

impl ExerciseCode {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn parse_supported(value: &str) -> Result<Self, ValidationError> {
        let code = Self::from_str(value)?;
        if EXERCISE_OPTIONS
            .iter()
            .any(|(supported, _)| *supported == value)
        {
            Ok(code)
        } else {
            Err(ValidationError::new(
                "dislikedExercises",
                "contains an unsupported exercise",
            ))
        }
    }
}

impl FromStr for ExerciseCode {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if !value.is_empty()
            && value.len() <= 64
            && value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
            && !value.starts_with('_')
            && !value.ends_with('_')
            && !value.contains("__")
        {
            Ok(Self(value.to_owned()))
        } else {
            Err(ValidationError::new(
                "dislikedExercises",
                "contains an invalid exercise code",
            ))
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalsExperienceAnswers {
    pub primary_goal: Goal,
    pub secondary_goals: Vec<Goal>,
    pub experience_level: ExperienceLevel,
    pub current_activity_level: ActivityLevel,
}

impl GoalsExperienceAnswers {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.secondary_goals.len() > 2 {
            return Err(ValidationError::new(
                "secondaryGoals",
                "must contain at most two goals",
            ));
        }
        let unique = self.secondary_goals.iter().copied().collect::<HashSet<_>>();
        if unique.len() != self.secondary_goals.len() || unique.contains(&self.primary_goal) {
            return Err(ValidationError::new(
                "secondaryGoals",
                "must be unique and different from the primary goal",
            ));
        }
        Ok(())
    }

    pub fn ranked_goals(&self) -> impl Iterator<Item = (Goal, i16)> + '_ {
        std::iter::once((self.primary_goal, 1)).chain(
            self.secondary_goals
                .iter()
                .copied()
                .enumerate()
                .map(|(index, goal)| (goal, i16::try_from(index).unwrap_or(0) + 2)),
        )
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleSetupAnswers {
    pub days_per_week: i16,
    pub minimum_duration_minutes: i16,
    pub preferred_duration_minutes: i16,
    pub maximum_duration_minutes: i16,
    pub training_environment: TrainingEnvironment,
    pub available_equipment: Vec<Equipment>,
    pub preferred_weekdays: Vec<Weekday>,
}

impl ScheduleSetupAnswers {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if !(1..=7).contains(&self.days_per_week) {
            return Err(ValidationError::new(
                "daysPerWeek",
                "must be between 1 and 7",
            ));
        }
        if !(15..=180).contains(&self.minimum_duration_minutes) {
            return Err(ValidationError::new(
                "minimumDurationMinutes",
                "must be between 15 and 180",
            ));
        }
        if !(15..=180).contains(&self.preferred_duration_minutes) {
            return Err(ValidationError::new(
                "preferredDurationMinutes",
                "must be between 15 and 180",
            ));
        }
        if !(15..=180).contains(&self.maximum_duration_minutes) {
            return Err(ValidationError::new(
                "maximumDurationMinutes",
                "must be between 15 and 180",
            ));
        }

        if !(&self.minimum_duration_minutes <= &self.preferred_duration_minutes) {
            return Err(ValidationError::new(
                "minimum_duration_minutes",
                "must be less than the preferred duration minutes",
            ));
        }

        if !(&self.preferred_duration_minutes <= &self.maximum_duration_minutes) {
            return Err(ValidationError::new(
                "preferredDurationMinutes",
                "must be less than the maximum duration minutes",
            ));
        }

        validate_unique_nonempty("availableEquipment", &self.available_equipment)?;
        validate_unique("preferredWeekdays", &self.preferred_weekdays)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExerciseAvoidance {
    pub exercise_code: ExerciseCode,
    pub preference: AvoidancePreference,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainingPreferencesAnswers {
    pub preferred_workout_types: Vec<WorkoutType>,
    pub disliked_exercises: Vec<ExerciseAvoidance>,
    pub coaching_style: CoachingStyle,
    pub starting_intensity: StartingIntensity,
    pub consistency_barrier: Option<String>,
}

impl TrainingPreferencesAnswers {
    pub fn validate(&mut self) -> Result<(), ValidationError> {
        validate_unique_nonempty("preferredWorkoutTypes", &self.preferred_workout_types)?;
        let unique = self
            .disliked_exercises
            .iter()
            .map(|avoidance| avoidance.exercise_code.as_str())
            .collect::<HashSet<_>>();
        if unique.len() != self.disliked_exercises.len() {
            return Err(ValidationError::new(
                "dislikedExercises",
                "cannot contain the same exercise more than once",
            ));
        }
        self.consistency_barrier =
            normalize_optional_text("consistencyBarrier", self.consistency_barrier.take(), 160)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MovementConstraint {
    pub body_region: BodyRegion,
    pub movement_pattern: Option<MovementPattern>,
    pub impact: ConstraintImpact,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MovementWellnessAnswers {
    pub movement_constraints: Vec<MovementConstraint>,
    pub accessibility_needs: Option<String>,
    pub height_mm: Option<i32>,
    pub weight_grams: Option<i32>,
    pub unit_system: UnitSystem,
    pub acknowledge_sensitive_data: bool,
}

impl MovementWellnessAnswers {
    pub fn validate(&mut self) -> Result<(), ValidationError> {
        if self.movement_constraints.len() > 12 {
            return Err(ValidationError::new(
                "movementConstraints",
                "must contain at most 12 entries",
            ));
        }
        for constraint in &mut self.movement_constraints {
            constraint.notes = normalize_optional_text("notes", constraint.notes.take(), 240)?;
        }
        self.accessibility_needs =
            normalize_optional_text("accessibilityNeeds", self.accessibility_needs.take(), 240)?;
        if self
            .height_mm
            .is_some_and(|value| !(500..=2750).contains(&value))
        {
            return Err(ValidationError::new(
                "heightMm",
                "must be between 500 and 2750 millimetres",
            ));
        }
        if self
            .weight_grams
            .is_some_and(|value| !(20_000..=500_000).contains(&value))
        {
            return Err(ValidationError::new(
                "weightGrams",
                "must be between 20000 and 500000 grams",
            ));
        }
        if self.contains_sensitive_data() && !self.acknowledge_sensitive_data {
            return Err(ValidationError::new(
                "acknowledgeSensitiveData",
                "must be accepted before saving health-related information",
            ));
        }
        Ok(())
    }

    pub fn contains_sensitive_data(&self) -> bool {
        !self.movement_constraints.is_empty()
            || self.accessibility_needs.is_some()
            || self.height_mm.is_some()
            || self.weight_grams.is_some()
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingAnswersV1 {
    pub goals_experience: Option<GoalsExperienceAnswers>,
    pub schedule_setup: Option<ScheduleSetupAnswers>,
    pub training_preferences: Option<TrainingPreferencesAnswers>,
    pub movement_wellness: Option<MovementWellnessAnswers>,
}

impl OnboardingAnswersV1 {
    pub fn completed_groups(&self) -> Vec<OnboardingGroup> {
        let mut groups = Vec::with_capacity(4);
        if self.goals_experience.is_some() {
            groups.push(OnboardingGroup::GoalsExperience);
        }
        if self.schedule_setup.is_some() {
            groups.push(OnboardingGroup::ScheduleSetup);
        }
        if self.training_preferences.is_some() {
            groups.push(OnboardingGroup::TrainingPreferences);
        }
        if self.movement_wellness.is_some() {
            groups.push(OnboardingGroup::MovementWellness);
        }
        groups
    }

    pub fn complete(self) -> Result<CompleteFitnessProfile, ValidationError> {
        let goals_experience = self
            .goals_experience
            .ok_or_else(|| ValidationError::new("goalsExperience", "must be completed"))?;
        let schedule_setup = self
            .schedule_setup
            .ok_or_else(|| ValidationError::new("scheduleSetup", "must be completed"))?;
        let mut training_preferences = self
            .training_preferences
            .ok_or_else(|| ValidationError::new("trainingPreferences", "must be completed"))?;
        let mut movement_wellness = self
            .movement_wellness
            .ok_or_else(|| ValidationError::new("movementWellness", "must be completed"))?;

        goals_experience.validate()?;
        schedule_setup.validate()?;
        training_preferences.validate()?;
        movement_wellness.validate()?;

        Ok(CompleteFitnessProfile {
            goals_experience,
            schedule_setup,
            training_preferences,
            movement_wellness,
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteFitnessProfile {
    pub goals_experience: GoalsExperienceAnswers,
    pub schedule_setup: ScheduleSetupAnswers,
    pub training_preferences: TrainingPreferencesAnswers,
    pub movement_wellness: MovementWellnessAnswers,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OnboardingGroup {
    GoalsExperience,
    ScheduleSetup,
    TrainingPreferences,
    MovementWellness,
    Complete,
}

impl OnboardingGroup {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GoalsExperience => "goals_experience",
            Self::ScheduleSetup => "schedule_setup",
            Self::TrainingPreferences => "training_preferences",
            Self::MovementWellness => "movement_wellness",
            Self::Complete => "complete",
        }
    }
}

impl FromStr for OnboardingGroup {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "goals_experience" => Ok(Self::GoalsExperience),
            "schedule_setup" => Ok(Self::ScheduleSetup),
            "training_preferences" => Ok(Self::TrainingPreferences),
            "movement_wellness" => Ok(Self::MovementWellness),
            "complete" => Ok(Self::Complete),
            _ => Err(ValidationError::new(
                "currentGroup",
                "contains an unsupported value",
            )),
        }
    }
}

fn validate_unique<T>(field: &'static str, values: &[T]) -> Result<(), ValidationError>
where
    T: Copy + Eq + std::hash::Hash,
{
    if values.iter().copied().collect::<HashSet<_>>().len() != values.len() {
        return Err(ValidationError::new(
            field,
            "cannot contain duplicate values",
        ));
    }
    Ok(())
}

fn validate_unique_nonempty<T>(field: &'static str, values: &[T]) -> Result<(), ValidationError>
where
    T: Copy + Eq + std::hash::Hash,
{
    if values.is_empty() {
        return Err(ValidationError::new(
            field,
            "must contain at least one value",
        ));
    }
    validate_unique(field, values)
}

fn normalize_optional_text(
    field: &'static str,
    value: Option<String>,
    maximum: usize,
) -> Result<Option<String>, ValidationError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    if value.chars().count() > maximum {
        return Err(ValidationError::new(
            field,
            format!("must not exceed {maximum} characters"),
        ));
    }
    Ok(Some(value.to_owned()))
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("{field} {message}")]
pub struct ValidationError {
    pub field: &'static str,
    pub message: String,
}

impl ValidationError {
    pub fn new(field: &'static str, message: impl Into<String>) -> Self {
        Self {
            field,
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ActivityLevel, Equipment, ExperienceLevel, Goal, GoalsExperienceAnswers,
        MovementWellnessAnswers, OnboardingAnswersV1, ScheduleSetupAnswers, TrainingEnvironment,
        UnitSystem,
    };

    #[test]
    fn ranked_goals_reject_duplicates() {
        let answers = GoalsExperienceAnswers {
            primary_goal: Goal::BuildMuscle,
            secondary_goals: vec![Goal::BuildMuscle],
            experience_level: ExperienceLevel::Beginner,
            current_activity_level: ActivityLevel::LightlyActive,
        };
        assert!(answers.validate().is_err());
    }

    #[test]
    fn sensitive_data_requires_acknowledgement() {
        let mut answers = MovementWellnessAnswers {
            movement_constraints: Vec::new(),
            accessibility_needs: None,
            height_mm: None,
            weight_grams: Some(80_000),
            unit_system: UnitSystem::Metric,
            acknowledge_sensitive_data: false,
        };
        assert!(answers.validate().is_err());
    }

    #[test]
    fn exercise_options_reject_unknown_codes_but_stored_codes_remain_parseable() {
        assert!(super::ExerciseCode::parse_supported("bench_press").is_ok());
        assert!(super::ExerciseCode::parse_supported("future_exercise").is_err());
        assert!("future_exercise".parse::<super::ExerciseCode>().is_ok());
        assert!("Future Exercise".parse::<super::ExerciseCode>().is_err());
    }

    #[test]
    fn schedule_rejects_duplicate_equipment() {
        let answers = ScheduleSetupAnswers {
            days_per_week: 3,
            minimum_duration_minutes: 45,
            preferred_duration_minutes: 45,
            maximum_duration_minutes: 45,
            training_environment: TrainingEnvironment::Gym,
            available_equipment: vec![Equipment::Dumbbells, Equipment::Dumbbells],
            preferred_weekdays: Vec::new(),
        };

        assert_eq!(
            answers
                .validate()
                .expect_err("duplicate equipment must be rejected")
                .field,
            "availableEquipment"
        );
    }

    #[test]
    fn completion_reports_the_first_missing_group() {
        assert_eq!(
            OnboardingAnswersV1::default()
                .complete()
                .expect_err("an empty draft cannot be completed")
                .field,
            "goalsExperience"
        );
    }
}
