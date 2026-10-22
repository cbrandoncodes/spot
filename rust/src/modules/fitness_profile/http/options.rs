use serde::Serialize;

use super::super::domain::EXERCISE_OPTIONS;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct OnboardingOptionsResponse {
    goals: Vec<Choice>,
    experience_levels: Vec<Choice>,
    activity_levels: Vec<Choice>,
    training_environments: Vec<Choice>,
    equipment: Vec<Choice>,
    workout_types: Vec<Choice>,
    exercises: Vec<Choice>,
    coaching_styles: Vec<Choice>,
    starting_intensities: Vec<Choice>,
    avoidance_preferences: Vec<Choice>,
    body_regions: Vec<Choice>,
    movement_patterns: Vec<Choice>,
    constraint_impacts: Vec<Choice>,
    weekdays: Vec<Choice>,
    unit_systems: Vec<Choice>,
}

impl OnboardingOptionsResponse {
    pub(super) fn new() -> Self {
        Self {
            goals: choices(&[
                ("general_fitness", "General fitness"),
                ("build_muscle", "Build muscle"),
                ("get_stronger", "Get stronger"),
                ("improve_conditioning", "Improve conditioning"),
                ("improve_mobility", "Improve mobility"),
                ("athletic_performance", "Athletic performance"),
            ]),
            experience_levels: choices(&[
                ("never_trained", "New to structured training"),
                ("beginner", "Beginner"),
                ("intermediate", "Intermediate"),
                ("advanced", "Advanced"),
            ]),
            activity_levels: choices(&[
                ("mostly_inactive", "Mostly inactive"),
                ("lightly_active", "Lightly active"),
                ("active", "Active"),
                ("very_active", "Very active"),
            ]),
            training_environments: choices(&[
                ("gym", "Gym"),
                ("home", "Home"),
                ("mixed", "Gym and home"),
            ]),
            equipment: choices(&[
                ("bodyweight", "Bodyweight"),
                ("dumbbells", "Dumbbells"),
                ("adjustable_bench", "Adjustable bench"),
                ("barbell", "Barbell"),
                ("weight_plates", "Weight plates"),
                ("squat_rack", "Squat rack"),
                ("cable_station", "Cable station"),
                ("lat_pulldown", "Lat pulldown"),
                ("leg_press", "Leg press"),
                ("smith_machine", "Smith machine"),
                ("resistance_bands", "Resistance bands"),
                ("kettlebells", "Kettlebells"),
                ("pull_up_bar", "Pull-up bar"),
                ("cardio_machines", "Cardio machines"),
            ]),
            workout_types: choices(&[
                ("strength", "Strength"),
                ("hypertrophy", "Muscle building"),
                ("conditioning", "Conditioning"),
                ("bodyweight", "Bodyweight"),
                ("mobility", "Mobility"),
                ("mixed", "Mixed"),
            ]),
            exercises: choices(EXERCISE_OPTIONS),
            coaching_styles: choices(&[
                ("calm_supportive", "Calm and supportive"),
                ("motivational", "Motivational"),
                ("direct", "Direct"),
                ("technical", "Technical"),
                ("balanced", "Balanced"),
            ]),
            starting_intensities: choices(&[
                ("gentle", "Ease me in"),
                ("balanced", "Balanced"),
                ("challenging", "Challenge me"),
            ]),
            avoidance_preferences: choices(&[
                ("prefer_alternative", "Prefer an alternative"),
                ("never_include", "Never include"),
            ]),
            body_regions: choices(&[
                ("neck", "Neck"),
                ("shoulder", "Shoulder"),
                ("elbow", "Elbow"),
                ("wrist_hand", "Wrist or hand"),
                ("upper_back", "Upper back"),
                ("lower_back", "Lower back"),
                ("hip", "Hip"),
                ("knee", "Knee"),
                ("ankle_foot", "Ankle or foot"),
                ("other", "Other"),
            ]),
            movement_patterns: choices(&[
                ("squat", "Squatting"),
                ("hinge", "Hip hinging"),
                ("lunge", "Lunging"),
                ("horizontal_push", "Horizontal pushing"),
                ("vertical_push", "Overhead pushing"),
                ("horizontal_pull", "Horizontal pulling"),
                ("vertical_pull", "Vertical pulling"),
                ("carry", "Carrying"),
                ("rotation", "Rotation"),
                ("running", "Running"),
                ("jumping", "Jumping"),
            ]),
            constraint_impacts: choices(&[
                ("monitor", "Monitor"),
                ("modify", "Modify"),
                ("avoid", "Avoid"),
            ]),
            weekdays: choices(&[
                ("monday", "Monday"),
                ("tuesday", "Tuesday"),
                ("wednesday", "Wednesday"),
                ("thursday", "Thursday"),
                ("friday", "Friday"),
                ("saturday", "Saturday"),
                ("sunday", "Sunday"),
            ]),
            unit_systems: choices(&[("metric", "Metric"), ("imperial", "Imperial")]),
        }
    }
}

#[derive(Serialize)]
struct Choice {
    code: &'static str,
    label: &'static str,
}

fn choices(values: &[(&'static str, &'static str)]) -> Vec<Choice> {
    values
        .iter()
        .map(|(code, label)| Choice { code, label })
        .collect()
}
