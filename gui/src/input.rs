// #![allow(clippy::type_complexity)]
use bevy::window::PrimaryWindow;
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_button_released_plugin::{ButtonReleasedEvent, ButtonsReleasedPlugin};

use crate::{
    Glyph, Selected, DEFAULT_COL_COUNT, DEFAULT_ROW_COUNT, DEFAUL_BG_COLOR, SELECTED_BG_COLOR,
};

#[derive(Event)]
pub struct SelectEvent {
    selected: Entity,
}

#[derive(Event)]
pub struct TypeEvent {
    char: char,
}

// time: Res<Time<Real>>,
pub fn keyboard_input_system(
    input: Res<Input<KeyCode>>,
    mut writer: EventWriter<SelectEvent>,
    mut selected_query: Query<&Glyph, With<Selected>>,
    mut unselected_query: Query<(Entity, &Glyph), Without<Selected>>,
) {
    let shift = input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let ctrl = input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);

    if ctrl && shift && input.just_pressed(KeyCode::A) {
        info!("Just pressed Ctrl + Shift + A!");
    }

    if input.any_pressed([KeyCode::Left, KeyCode::Right, KeyCode::Up, KeyCode::Down]) {
        if let Ok(glyph) = selected_query.get_single_mut() {
            let x_delta: f32 = if input.just_pressed(KeyCode::Left) {
                -1.0
            } else if input.just_pressed(KeyCode::Right) {
                1.0
            } else {
                0.0
            };

            let y_delta: f32 = if input.just_pressed(KeyCode::Up) {
                -1.0
            } else if input.just_pressed(KeyCode::Down) {
                1.0
            } else {
                0.0
            };

            // println!("glyph {}.{}", glyph.x, glyph.y);
            // println!("delta {}.{}", x_delta, y_delta);

            if x_delta != 0.0 || y_delta != 0.0 {
                let x = (glyph.x + x_delta).clamp(1.0, DEFAULT_COL_COUNT);
                let y = (glyph.y + y_delta).clamp(1.0, DEFAULT_ROW_COUNT);

                // println!("{}.{}", x, y);

                for (selected, glyph) in &mut unselected_query {
                    if glyph.x == x && glyph.y == y {
                        writer.send(SelectEvent { selected });
                    }
                }
            }
        }
    }
}

pub fn cell_select_handler(
    mut reader: EventReader<SelectEvent>,
    mut commands: Commands,
    mut selected_query: Query<(Entity, &mut BackgroundColor), With<Selected>>,
    mut unselected_query: Query<&mut BackgroundColor, Without<Selected>>,
) {
    for event in reader.read() {
        eprintln!("Entity {:?} selected!", event.selected);

        if let Ok((current, mut bg_color)) = selected_query.get_single_mut() {
            commands.entity(current).remove::<Selected>();
            *bg_color = DEFAUL_BG_COLOR.into();
        }

        if let Ok(mut bg_color) = unselected_query.get_mut(event.selected) {
            *bg_color = SELECTED_BG_COLOR.into();
            commands.entity(event.selected).insert(Selected);
        }
    }
}

pub fn type_handler(
    mut reader: EventReader<TypeEvent>,
    mut writer: EventWriter<SelectEvent>,
    mut selected_query: Query<(&mut Glyph, &Children), With<Selected>>,
    mut unselected_query: Query<(Entity, &Glyph), Without<Selected>>,
    mut text_query: Query<&mut Text>,
) {
    for event in reader.read() {
        if let Ok((mut selected_glyph, children)) = selected_query.get_single_mut() {
            let mut text = text_query.get_mut(children[0]).unwrap();

            selected_glyph.char = event.char;
            text.sections[0].value = event.char.to_string();

            // Shift left 1
            let x_delta = 1.0;

            let x = (selected_glyph.x + x_delta).clamp(1.0, DEFAULT_COL_COUNT);

            // Find the unselected glyph at the new position
            for (selected, glyph) in &mut unselected_query {
                if x == glyph.x && selected_glyph.y == glyph.y {
                    writer.send(SelectEvent { selected });
                }
            }
        }
    }
}

pub fn text_input_system(
    _input: Res<Input<KeyCode>>,
    mut reader: EventReader<ReceivedCharacter>,
    mut writer: EventWriter<TypeEvent>,
    // mut string: Local<String>,
) {
    // if input.just_pressed(KeyCode::Return) {
    //     println!("Text input: {}", &*string);
    //     string.clear();
    // }
    // if input.just_pressed(KeyCode::Back) {
    //     string.pop();
    // }
    for received in reader.read() {
        if !received.char.is_control() {
            println!("{}", received.char);
            writer.send(TypeEvent {
                char: received.char,
            });
        }
    }
}
