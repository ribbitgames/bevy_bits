use bevy::prelude::*;
use bevy::utils::Duration;

#[derive(Resource, Default)]
pub struct ScoreUI {
    score: u32,
    digit: usize,
    visibility: Visibility,
    is_dirty: bool,
}

impl ScoreUI {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, new_score: u32) {
        self.score = new_score;
        self.is_dirty = true;
    }

    pub fn set_visiblity(&mut self, new_visiblity: Visibility) {
        self.visibility = new_visiblity;
        self.is_dirty = true;
    }

    pub fn set_digit(&mut self, new_digit: usize) {
        self.digit = new_digit;
        self.is_dirty = true;
    }
}

#[derive(Resource, Default)]
pub struct TimeUI {
    time: Duration,
    visibility: Visibility,
    is_dirty: bool,
}

impl TimeUI {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, new_time: Duration) {
        self.time = new_time;
        self.is_dirty = true;
    }

    pub fn set_visiblity(&mut self, new_visibility: Visibility) {
        self.visibility = new_visibility;
        self.is_dirty = true;
    }
}

#[derive(Resource, Default)]
pub struct CenterTextUI {
    text: String,
    visibility: Visibility,
    is_dirty: bool,
}

impl CenterTextUI {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, new_text: String) {
        self.text = new_text;
        self.is_dirty = true;
    }

    pub fn set_visiblity(&mut self, new_visibility: Visibility) {
        self.visibility = new_visibility;
        self.is_dirty = true;
    }
}

#[derive(Resource, Default)]
pub struct BottomTextUI {
    text: String,
    visibility: Visibility,
    is_dirty: bool,
}

impl BottomTextUI {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, new_text: String) {
        self.text = new_text;
        self.is_dirty = true;
    }

    pub fn set_visiblity(&mut self, new_visibility: Visibility) {
        self.visibility = new_visibility;
        self.is_dirty = true;
    }
}

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct TimeText;

#[derive(Component)]
struct CenterText;

#[derive(Component)]
struct BottomText;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ScoreUI::new())
            .insert_resource(TimeUI::new())
            .insert_resource(CenterTextUI::new())
            .insert_resource(BottomTextUI::new())
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    update_score,
                    update_time,
                    update_center_text,
                    update_bottom_text,
                ),
            );
    }
}

fn setup(mut commands: Commands) {
    commands
        .spawn(Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            display: Display::Grid,
            grid_template_rows: RepeatedGridTrack::fr(3, 1.),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn(Node {
                    display: Display::Grid,
                    grid_template_columns: RepeatedGridTrack::fr(2, 1.),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            align_self: AlignSelf::Start,
                            justify_self: JustifySelf::Start,
                            ..default()
                        },
                        Text::new(""),
                        Visibility::Hidden,
                        ScoreText,
                    ));
                    parent.spawn((
                        Node {
                            align_self: AlignSelf::Start,
                            justify_self: JustifySelf::End,
                            ..default()
                        },
                        Text::new(""),
                        Visibility::Hidden,
                        TimeText,
                    ));
                });
            parent.spawn((
                Node {
                    align_self: AlignSelf::Center,
                    justify_self: JustifySelf::Center,
                    ..default()
                },
                Text::new(""),
                Visibility::Hidden,
                TextColor(Color::srgba(1., 0., 0., 1.)),
                BackgroundColor(Color::srgba(0.25, 0.25, 0.25, 0.75)),
                CenterText,
            ));
            parent.spawn((
                Node {
                    align_self: AlignSelf::End,
                    justify_self: JustifySelf::Center,
                    ..default()
                },
                Text::new(""),
                Visibility::Hidden,
                BottomText,
            ));
        });
}

fn update_score(
    mut score: ResMut<ScoreUI>,
    mut query: Query<(&ScoreText, &mut Text, &mut Visibility)>,
) {
    if score.is_dirty {
        for (_, mut text, mut visibility) in &mut query {
            *text = Text::new(format!(
                "SCORE {:0digit$}",
                score.score,
                digit = score.digit
            ));
            *visibility = score.visibility;
        }
        score.is_dirty = false;
    }
}

fn update_time(
    mut time: ResMut<TimeUI>,
    mut query: Query<(&TimeText, &mut Text, &mut Visibility)>,
) {
    if time.is_dirty {
        for (_, mut text, mut visibility) in &mut query {
            *text = Text::new(format!(
                "{:02}:{:02}",
                time.time.as_secs(),
                time.time.subsec_millis() / 10
            ));
            *visibility = time.visibility;
        }
        time.is_dirty = false;
    }
}

fn update_center_text(
    mut center_text: ResMut<CenterTextUI>,
    mut query: Query<(&CenterText, &mut Text, &mut Visibility)>,
) {
    if center_text.is_dirty {
        for (_, mut text, mut visibility) in &mut query {
            *text = Text::new(center_text.text.clone());
            *visibility = center_text.visibility;
        }
        center_text.is_dirty = false;
    }
}

fn update_bottom_text(
    mut bottom_text: ResMut<BottomTextUI>,
    mut query: Query<(&BottomText, &mut Text, &mut Visibility)>,
) {
    if bottom_text.is_dirty {
        for (_, mut text, mut visibility) in &mut query {
            *text = Text::new(bottom_text.text.clone());
            *visibility = bottom_text.visibility;
        }
        bottom_text.is_dirty = false;
    }
}
