use bevy::prelude::*;
use bevy::time::Timer;
use bits_helpers::welcome_screen::{WelcomeScreenElement, despawn_welcome_screen};
use bits_helpers::{FONT, send_bit_message};
use ribbit::MathQuiz;
use ribbit_bits::{BitMessage, BitResult};

mod ribbit;

const ANSWER_BOX_SIZE: Vec2 = Vec2::new(80.0, 40.0);
const QUESTION_FONT_SIZE: f32 = 40.0;
const ANSWER_FONT_SIZE: f32 = 30.0;
const TIMER_FONT_SIZE: f32 = 24.0;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Welcome,
    Playing,
    GameOver,
}

#[derive(Resource)]
struct GameData {
    current_stage: u32,
    current_question: String,
    correct_answer: i32,
    timer: Timer,
    stage_entity: Option<Entity>,
    feedback_timer: Option<Timer>,
    last_answer_correct: bool,
    waiting_for_feedback: bool,
}

#[derive(Component)]
struct AnswerBox {
    value: i32,
}

#[derive(Component)]
struct TimerText;

#[derive(Component)]
struct CleanupMarker;

#[derive(Component)]
struct StageText;

impl Default for GameData {
    fn default() -> Self {
        Self {
            current_stage: 1,
            current_question: String::new(),
            correct_answer: 0,
            timer: Timer::from_seconds(60.0, TimerMode::Once),
            stage_entity: None,
            feedback_timer: None,
            last_answer_correct: false,
            waiting_for_feedback: false,
        }
    }
}

pub fn run() {
    bits_helpers::get_default_app::<MathQuiz>(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
        .init_state::<GameState>()
        .init_resource::<GameData>()
        .add_systems(Startup, setup)
        .add_systems(OnEnter(GameState::Welcome), spawn_welcome_screen)
        .add_systems(OnExit(GameState::Welcome), despawn_welcome_screen)
        .add_systems(OnEnter(GameState::Playing), spawn_game_elements)
        .add_systems(OnExit(GameState::Playing), cleanup_marked_entities)
        .add_systems(
            Update,
            (
                handle_welcome_input.run_if(in_state(GameState::Welcome)),
                (update_timer, check_answer, handle_feedback_timer)
                    .run_if(in_state(GameState::Playing)),
            ),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_welcome_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Text::new("Math Quiz"),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 40.0,
            ..default()
        },
        Node {
            top: Val::Percent(5.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Center),
        WelcomeScreenElement,
    ));

    commands.spawn((
        Text::new("Solve math problems as quickly as you can!"),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 40.0,
            ..default()
        },
        Node {
            top: Val::Percent(30.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Center),
        WelcomeScreenElement,
    ));
}

fn handle_welcome_input(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) || touch_input.any_just_pressed() {
        next_state.set(GameState::Playing);
    }
}

fn spawn_game_elements(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut game_data: ResMut<GameData>,
    stage_text_query: Query<&mut Text, With<StageText>>,
) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            CleanupMarker,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Time: 0.0"),
                TextFont {
                    font: asset_server.load(FONT),
                    font_size: TIMER_FONT_SIZE,
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(10.0),
                    right: Val::Percent(5.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                TimerText,
            ));

            parent.spawn((
                Text::new(format!("Stage: {}", game_data.current_stage)),
                TextFont {
                    font: asset_server.load(FONT),
                    font_size: TIMER_FONT_SIZE,
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(10.0),
                    left: Val::Px(10.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                StageText,
            ));
        });

    game_data.timer.reset();
    game_data.current_stage = 1;

    generate_question(
        &mut commands,
        &asset_server,
        &mut game_data,
        stage_text_query,
    );
}

fn generate_question(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    game_data: &mut GameData,
    mut stage_text_query: Query<&mut Text, With<StageText>>,
) {
    if let Some(entity) = game_data.stage_entity.take() {
        commands.entity(entity).despawn_recursive();
    }

    match game_data.current_stage {
        1 => {
            let a = fastrand::i32(2..10);
            let b = fastrand::i32(2..10);
            game_data.current_question = format!("{a} x {b}");
            game_data.correct_answer = a * b;
        }
        2 => {
            let a = fastrand::i32(10..100);
            let b = fastrand::i32(10..100);
            game_data.current_question = format!("{a} + {b}");
            game_data.correct_answer = a + b;
        }
        3 => {
            let a = fastrand::i32(10..100);
            let b = fastrand::i32(2..10);
            game_data.current_question = format!("{a} x {b}");
            game_data.correct_answer = a * b;
        }
        4 => {
            let a = fastrand::i32(10..100);
            let b = fastrand::i32(1..10);
            let c = fastrand::i32(1..10);
            let op1 = if fastrand::bool() { "+" } else { "-" };
            let op2 = if fastrand::bool() { "+" } else { "-" };
            game_data.current_question = format!("{a} {op1} {b}({b} {op2} {c})");
            game_data.correct_answer = if op1 == "+" {
                a + b * (if op2 == "+" { b + c } else { b - c })
            } else {
                a - b * (if op2 == "+" { b + c } else { b - c })
            };
        }
        _ => {}
    }

    let correct_final_digit = game_data.correct_answer % 10;
    let mut answers = vec![game_data.correct_answer];

    // Ensure at least one other answer has the same final digit
    let mut attempts = 0;
    while answers.len() < 2 && attempts < 100 {
        let wrong_answer = game_data.correct_answer + fastrand::i32(-10..10);
        if wrong_answer != game_data.correct_answer
            && !answers.contains(&wrong_answer)
            && wrong_answer % 10 == correct_final_digit
        {
            answers.push(wrong_answer);
        }
        attempts += 1;
    }

    // Generate remaining wrong answers
    while answers.len() < 4 {
        let wrong_answer = game_data.correct_answer + fastrand::i32(-10..10);
        if wrong_answer != game_data.correct_answer && !answers.contains(&wrong_answer) {
            answers.push(wrong_answer);
        }
    }

    // Manual Fisher-Yates shuffle implementation
    for i in (1..answers.len()).rev() {
        let j = fastrand::usize(..=i);
        answers.swap(i, j);
    }

    let stage_entity = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            CleanupMarker,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(&game_data.current_question),
                TextFont {
                    font: asset_server.load(FONT),
                    font_size: QUESTION_FONT_SIZE,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Auto,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                })
                .with_children(|parent| {
                    for answer in answers {
                        parent
                            .spawn((
                                Node {
                                    width: Val::Px(ANSWER_BOX_SIZE.x),
                                    height: Val::Px(ANSWER_BOX_SIZE.y),
                                    margin: UiRect::horizontal(Val::Px(5.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::BLACK),
                                Button,
                                AnswerBox { value: answer },
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    Text::new(answer.to_string()),
                                    TextFont {
                                        font: asset_server.load(FONT),
                                        font_size: ANSWER_FONT_SIZE,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    }
                });
        })
        .id();

    game_data.stage_entity = Some(stage_entity);
    game_data.waiting_for_feedback = false;

    if let Ok(mut text) = stage_text_query.get_single_mut() {
        text.0 = format!("Stage: {}", game_data.current_stage);
    }
}

fn check_answer(
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    interaction_query: Query<
        (Entity, &Interaction, &AnswerBox),
        (Changed<Interaction>, With<Button>),
    >,
) {
    if game_data.waiting_for_feedback {
        return;
    }

    for (entity, interaction, answer_box) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            let is_correct = answer_box.value == game_data.correct_answer;
            let color = if is_correct {
                Color::srgb(0.0, 1.0, 0.0)
            } else {
                Color::srgb(1.0, 0.0, 0.0)
            };

            commands.entity(entity).insert(BackgroundColor(color));

            game_data.last_answer_correct = is_correct;
            if is_correct {
                game_data.current_stage += 1;
            }

            game_data.feedback_timer = Some(Timer::from_seconds(1.0, TimerMode::Once));
            game_data.waiting_for_feedback = true;

            break;
        }
    }
}

fn handle_feedback_timer(
    mut game_data: ResMut<GameData>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
    stage_text_query: Query<&mut Text, With<StageText>>,
) {
    if let Some(ref mut timer) = game_data.feedback_timer {
        timer.tick(time.delta());

        if timer.finished() {
            if game_data.last_answer_correct {
                if game_data.current_stage > 4 {
                    next_state.set(GameState::GameOver);
                    send_bit_message(BitMessage::End(BitResult::FastestDuration(
                        game_data.timer.elapsed(),
                    )));
                } else {
                    generate_question(
                        &mut commands,
                        &asset_server,
                        &mut game_data,
                        stage_text_query,
                    );
                }
            } else {
                next_state.set(GameState::GameOver);
                send_bit_message(BitMessage::End(BitResult::Failure));
            }
            game_data.feedback_timer = None;
            game_data.waiting_for_feedback = false;
        }
    }
}

fn update_timer(
    time: Res<Time>,
    mut game_data: ResMut<GameData>,
    mut timer_query: Query<&mut Text, With<TimerText>>,
) {
    game_data.timer.tick(time.delta());
    if let Ok(mut text) = timer_query.get_single_mut() {
        text.0 = format!("Time: {:.1}", game_data.timer.elapsed_secs());
    }
}

fn cleanup_marked_entities(mut commands: Commands, query: Query<Entity, With<CleanupMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
