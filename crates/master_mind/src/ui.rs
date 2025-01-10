use bevy::prelude::*;
use bevy::utils::Duration;

const BUTTON_NORMAL: Color = Color::srgb(0.15, 0.15, 0.15);
const BUTTON_HOVERED: Color = Color::srgb(0.25, 0.25, 0.25);
const BUTTON_PRESSED: Color = Color::srgb(0.35, 0.35, 0.35);
const BUTTON_FRAME_NORMAL: Color = Color::srgb(0.25, 0.25, 0.25);
const BUTTON_FRAME_HOVERED: Color = Color::srgb(0.5, 0.5, 0.5);
const BUTTON_FRAME_PRESSED: Color = Color::srgb(1., 1., 1.);

#[derive(Event)]
pub struct ButtonEvent {
    pub key_code: KeyCode,
}

#[derive(Event)]
pub struct ShowMessageEvent {
    pub message: String,
}

#[derive(Event)]
pub struct EndMessageEvent;

#[derive(Component)]
struct MessageTimer {
    timer: Timer,
}

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ButtonEvent>()
            .add_event::<ShowMessageEvent>()
            .add_event::<EndMessageEvent>()
            .add_systems(Startup, setup)
            .add_systems(Update, (button_system, show_message_system, update_message));
    }
}

fn setup(mut commands: Commands) {
    let button = (
        Button,
        Node {
            width: Val::Px(96.0),
            height: Val::Px(48.0),
            border: UiRect::all(Val::Px(5.0)),
            // horizontally center child text
            justify_content: JustifyContent::Center,
            // vertically center child text
            align_items: AlignItems::Center,

            ..default()
        },
        BorderColor(Color::srgb(0.5, 0.5, 0.5)),
        BorderRadius::MAX,
        BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
    );
    let text_bundle = |c| {
        (
            Text::new(c),
            TextFont {
                font_size: 32.0,
                ..default()
            },
        )
    };
    commands
        .spawn(Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            align_items: AlignItems::End,
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(button.clone()).with_children(|parent| {
                parent.spawn(text_bundle("1"));
            });
        })
        .with_children(|parent| {
            parent.spawn(button.clone()).with_children(|parent| {
                parent.spawn(text_bundle("2"));
            });
        })
        .with_children(|parent| {
            parent.spawn(button.clone()).with_children(|parent| {
                parent.spawn(text_bundle("3"));
            });
        })
        .with_children(|parent| {
            parent.spawn(button.clone()).with_children(|parent| {
                parent.spawn(text_bundle("4"));
            });
        })
        .with_children(|parent| {
            parent.spawn(button.clone()).with_children(|parent| {
                parent.spawn(text_bundle("5"));
            });
        })
        .with_children(|parent| {
            parent.spawn(button.clone()).with_children(|parent| {
                parent.spawn(text_bundle("6"));
            });
        })
        .with_children(|parent| {
            parent.spawn(button.clone()).with_children(|parent| {
                parent.spawn(text_bundle("7"));
            });
        })
        .with_children(|parent| {
            parent.spawn(button.clone()).with_children(|parent| {
                parent.spawn(text_bundle("8"));
            });
        })
        .with_children(|parent| {
            parent.spawn(button.clone()).with_children(|parent| {
                parent.spawn(text_bundle("9"));
            });
        })
        .with_children(|parent| {
            parent.spawn(button.clone()).with_children(|parent| {
                parent.spawn(text_bundle("0"));
            });
        })
        .with_children(|parent| {
            parent.spawn(button.clone()).with_children(|parent| {
                parent.spawn(text_bundle("BS"));
            });
        });
}

fn button_system(
    //mut commands: Commands,
    mut query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    text_query: Query<&Text>,
    mut button_event: EventWriter<ButtonEvent>,
) {
    for (interaction, mut color, mut border_color, children) in &mut query {
        let Some(child) = children.first() else {
            continue;
        };
        match *interaction {
            Interaction::Pressed => {
                if let Ok(key) = text_query.get(*child).map(|a| a.0.as_str()) {
                    match key {
                        "1" => {
                            button_event.send(ButtonEvent {
                                key_code: KeyCode::Digit1,
                            });
                            //println!("1");
                        }
                        "2" => {
                            button_event.send(ButtonEvent {
                                key_code: KeyCode::Digit2,
                            });
                            //println!("2");
                        }
                        "3" => {
                            button_event.send(ButtonEvent {
                                key_code: KeyCode::Digit3,
                            });
                            //println!("3");
                        }
                        "4" => {
                            button_event.send(ButtonEvent {
                                key_code: KeyCode::Digit4,
                            });
                            //println!("4");
                        }
                        "5" => {
                            button_event.send(ButtonEvent {
                                key_code: KeyCode::Digit5,
                            });
                            //println!("5");
                        }
                        "6" => {
                            button_event.send(ButtonEvent {
                                key_code: KeyCode::Digit6,
                            });
                            //println!("6");
                        }
                        "7" => {
                            button_event.send(ButtonEvent {
                                key_code: KeyCode::Digit7,
                            });
                            //println!("7");
                        }
                        "8" => {
                            button_event.send(ButtonEvent {
                                key_code: KeyCode::Digit8,
                            });
                            //println!("8");
                        }
                        "9" => {
                            button_event.send(ButtonEvent {
                                key_code: KeyCode::Digit9,
                            });
                            //println!("9");
                        }
                        "0" => {
                            button_event.send(ButtonEvent {
                                key_code: KeyCode::Digit0,
                            });
                            //println!("0");
                        }
                        "BS" => {
                            button_event.send(ButtonEvent {
                                key_code: KeyCode::Backspace,
                            });
                            //println!("BS");
                        }
                        _ => {}
                    }
                };
                *color = BUTTON_PRESSED.into();
                border_color.0 = BUTTON_FRAME_PRESSED;
            }
            Interaction::Hovered => {
                *color = BUTTON_HOVERED.into();
                border_color.0 = BUTTON_FRAME_HOVERED;
            }
            Interaction::None => {
                *color = BUTTON_NORMAL.into();
                border_color.0 = BUTTON_FRAME_NORMAL;
            }
        }
    }
}

fn show_message_system(mut commands: Commands, mut events: EventReader<ShowMessageEvent>) {
    for event in events.read() {
        commands
            .spawn((
                Text::from(event.message.clone()),
                TextFont {
                    font_size: 32.,
                    ..default()
                },
                Transform::from_translation(Vec3::new(0., 0., 10.)),
            ))
            .insert(MessageTimer {
                timer: Timer::new(Duration::from_secs(2), TimerMode::Once),
            });
    }
}

fn update_message(
    mut commands: Commands,
    mut query: Query<(Entity, &mut MessageTimer)>,
    time: Res<Time>,
    mut event: EventWriter<EndMessageEvent>,
) {
    for (entity, mut message) in &mut query {
        message.timer.tick(time.delta());
        if message.timer.finished() {
            commands.entity(entity).despawn();
            event.send(EndMessageEvent);
        }
    }
}
