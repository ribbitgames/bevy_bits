use bevy::prelude::*;
use bits_helpers::send_bit_message;
use decoding_board::{AddCharacterAt, RemoveCharacterAt, ResetBoard, SetColorsAt};
use ribbit::MasterMind;

mod decoding_board;
mod ribbit;
mod ui;

const CODE_LENGTH: usize = 4;
const MAX_TURN: usize = 8;

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash, Default)]
enum DecodeResult {
    #[default]
    Invalid,
    Miss,
    Blow,
    Hit,
}

#[derive(Resource)]
struct GameProgress {
    turn: usize,
    index: usize,
    encoded: Vec<char>,
    decoding: Vec<char>,
}

#[derive(Event)]
struct ResetGame;

pub fn run() {
    bits_helpers::get_default_app::<MasterMind>(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
        .insert_resource(GameProgress {
            turn: 0,
            index: 0,
            encoded: vec![' '; CODE_LENGTH],
            decoding: vec![' '; CODE_LENGTH],
        })
        .add_event::<ResetGame>()
        .add_plugins(decoding_board::DecodingBoardPlugin {
            row: MAX_TURN,
            column: CODE_LENGTH,
        })
        .add_plugins(ui::UIPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (reset_game, ui_input, keyboard_input))
        .run();
}

fn setup(mut commands: Commands, mut event: EventWriter<ResetGame>) {
    commands.spawn(Camera2d);
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Center),
        Text::from(format!("Guess {CODE_LENGTH} digit number")),
        TextFont {
            font_size: 24.0,
            ..default()
        },
    ));
    event.send(ResetGame);
}

fn reset_game(
    mut game_progress: ResMut<GameProgress>,
    mut event: EventWriter<ResetBoard>,
    mut events: EventReader<ResetGame>,
) {
    for _event in events.read() {
        encode_characters(&mut game_progress);
        event.send(ResetBoard);
        game_progress.index = 0;
        game_progress.turn = 0;
    }
}

fn encode_characters(game_progress: &mut ResMut<GameProgress>) {
    let code: Vec<char> = (0..CODE_LENGTH)
        .map(|_x| char::from_digit(fastrand::u32(0..10), 10).expect(""))
        .collect();
    game_progress.encoded = code;
    print!("Encoded Code = ");
    for c in &game_progress.encoded {
        print!("{c}");
    }
    println!(".");
}

// codemaker turn, giving feedback
fn decode_characters(game_progress: &mut ResMut<GameProgress>) -> (bool, Vec<Color>) {
    let mut count: usize = 0;
    let mut checked: [bool; CODE_LENGTH] = [false; CODE_LENGTH];
    let mut result: [DecodeResult; CODE_LENGTH] = [DecodeResult::Invalid; CODE_LENGTH];
    for (index, decoding_char) in game_progress.decoding.iter().enumerate() {
        let Some(encoded_char) = game_progress.encoded.get(index) else {
            continue;
        };
        if *decoding_char == *encoded_char {
            count += 1;
            let Some(c) = checked.get_mut(index) else {
                continue;
            };
            *c = true;
            let Some(res) = result.get_mut(index) else {
                continue;
            };
            *res = DecodeResult::Hit;
        }
    }
    for (index, decoding_char) in game_progress.decoding.iter().enumerate() {
        let Some(res) = result.get_mut(index) else {
            continue;
        };
        if *res != DecodeResult::Invalid {
            continue;
        }
        for (index2, encoded_char) in game_progress.encoded.iter().enumerate() {
            if index == index2 {
                continue;
            }
            let Some(c) = checked.get_mut(index2) else {
                continue;
            };
            if *decoding_char == *encoded_char && *res == DecodeResult::Invalid && !(*c) {
                *res = DecodeResult::Blow;
                *c = true;
                break;
            }
        }
        if *res == DecodeResult::Invalid {
            *res = DecodeResult::Miss;
        }
    }
    print!("Decoding Code = ");
    for c in &game_progress.decoding {
        print!("{c}");
    }
    println!(".");

    // following things might be outside of this func and in anothor func, and this func returns result
    if count == CODE_LENGTH {
        println!("Decoded!");
        // good!
        return (true, vec![decoding_board::FRAME_COLOR_HIT; CODE_LENGTH]);
    }
    game_progress.index = 0;
    game_progress.turn += 1;
    let mut colors = Vec::new();
    print!("Decoding Code = ");
    for c in &result {
        match *c {
            DecodeResult::Hit => {
                print!("H");
                colors.push(decoding_board::FRAME_COLOR_HIT);
            }
            DecodeResult::Blow => {
                print!("B");
                colors.push(decoding_board::FRAME_COLOR_BLOW);
            }
            DecodeResult::Miss => {
                print!("M");
                colors.push(decoding_board::FRAME_COLOR_MISS);
            }
            DecodeResult::Invalid => {
                colors.push(Color::BLACK);
            }
        }
    }
    println!(".");
    (false, colors)
}

// codebraker turn, guessing a pattern of X code characters
fn enter_character(c: char, game_progress: &mut ResMut<GameProgress>) -> (bool, Vec<Color>) {
    let index = game_progress.index;
    if let Some(ch) = game_progress.decoding.get_mut(index) {
        *ch = c;
        if game_progress.index < CODE_LENGTH - 1 {
            game_progress.index += 1;
            return (false, Vec::new());
        }
        let result = decode_characters(game_progress);
        return result;
    }
    (false, Vec::new())
}

fn delete_character(game_progress: &mut ResMut<GameProgress>) -> bool {
    let index = game_progress.index;
    if index == 0 {
        return false;
    }
    if let Some(ch) = game_progress.decoding.get_mut(index - 1) {
        *ch = ' ';
        game_progress.index -= 1;
        return true;
    }
    false
}

fn common_input(
    c: char,
    game_progress: &mut ResMut<GameProgress>,
    add_event: &mut EventWriter<decoding_board::AddCharacterAt>,
    board_event: &mut EventWriter<decoding_board::SetColorsAt>,
) {
    let row = game_progress.turn;
    let result = enter_character(c, game_progress);
    add_event.send(AddCharacterAt {
        x: game_progress.index,
        y: game_progress.turn,
        c,
    });

    let (res, colors) = result;
    if colors.len() == CODE_LENGTH {
        board_event.send(SetColorsAt { row, colors });
        if res {
            send_bit_message(ribbit_bits::BitMessage::End(
                ribbit_bits::BitResult::Success,
            ));
        } else if row == MAX_TURN - 1 {
            send_bit_message(ribbit_bits::BitMessage::End(
                ribbit_bits::BitResult::Failure,
            ));
        }
    }
}

fn ui_input(
    mut button_events: EventReader<ui::ButtonEvent>,
    mut game_progress: ResMut<GameProgress>,
    mut add_event: EventWriter<decoding_board::AddCharacterAt>,
    mut remove_event: EventWriter<decoding_board::RemoveCharacterAt>,
    mut board_event: EventWriter<decoding_board::SetColorsAt>,
) {
    for event in button_events.read() {
        let mut c = ' ';
        match event.key_code {
            KeyCode::Digit1 => {
                c = '1';
            }
            KeyCode::Digit2 => {
                c = '2';
            }
            KeyCode::Digit3 => {
                c = '3';
            }
            KeyCode::Digit4 => {
                c = '4';
            }
            KeyCode::Digit5 => {
                c = '5';
            }
            KeyCode::Digit6 => {
                c = '6';
            }
            KeyCode::Digit7 => {
                c = '7';
            }
            KeyCode::Digit8 => {
                c = '8';
            }
            KeyCode::Digit9 => {
                c = '9';
            }
            KeyCode::Digit0 => {
                c = '0';
            }
            KeyCode::Backspace => {
                if delete_character(&mut game_progress) {
                    remove_event.send(RemoveCharacterAt {
                        x: game_progress.index,
                        y: game_progress.turn,
                    });
                }
            }
            _ => {}
        }
        if c != ' ' {
            common_input(c, &mut game_progress, &mut add_event, &mut board_event);
        }
    }
}

// for dev purpose
fn keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut game_progress: ResMut<GameProgress>,
    mut add_event: EventWriter<decoding_board::AddCharacterAt>,
    mut remove_event: EventWriter<decoding_board::RemoveCharacterAt>,
    mut board_event: EventWriter<decoding_board::SetColorsAt>,
) {
    if keyboard_input.just_pressed(KeyCode::Minus)
        || keyboard_input.just_pressed(KeyCode::Backspace)
    {
        if delete_character(&mut game_progress) {
            remove_event.send(RemoveCharacterAt {
                x: game_progress.index,
                y: game_progress.turn,
            });
        }
        return;
    }
    let mut c = ' ';
    if keyboard_input.just_pressed(KeyCode::Digit0) || keyboard_input.just_pressed(KeyCode::Numpad0)
    {
        c = '0';
    } else if keyboard_input.just_pressed(KeyCode::Digit1)
        || keyboard_input.just_pressed(KeyCode::Numpad1)
    {
        c = '1';
    } else if keyboard_input.just_pressed(KeyCode::Digit2)
        || keyboard_input.just_pressed(KeyCode::Numpad2)
    {
        c = '2';
    } else if keyboard_input.just_pressed(KeyCode::Digit3)
        || keyboard_input.just_pressed(KeyCode::Numpad3)
    {
        c = '3';
    } else if keyboard_input.just_pressed(KeyCode::Digit4)
        || keyboard_input.just_pressed(KeyCode::Numpad4)
    {
        c = '4';
    } else if keyboard_input.just_pressed(KeyCode::Digit5)
        || keyboard_input.just_pressed(KeyCode::Numpad5)
    {
        c = '5';
    } else if keyboard_input.just_pressed(KeyCode::Digit6)
        || keyboard_input.just_pressed(KeyCode::Numpad6)
    {
        c = '6';
    } else if keyboard_input.just_pressed(KeyCode::Digit7)
        || keyboard_input.just_pressed(KeyCode::Numpad7)
    {
        c = '7';
    } else if keyboard_input.just_pressed(KeyCode::Digit8)
        || keyboard_input.just_pressed(KeyCode::Numpad8)
    {
        c = '8';
    } else if keyboard_input.just_pressed(KeyCode::Digit9)
        || keyboard_input.just_pressed(KeyCode::Numpad9)
    {
        c = '9';
    }
    if c != ' ' {
        common_input(c, &mut game_progress, &mut add_event, &mut board_event);
    }
}
