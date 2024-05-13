//! This example illustrates how to create UI text and update it in a system.
//!
//! It displays the current FPS in the top left corner, as well as text that changes color
//! in the bottom right. For text within a scene, please see the text2d example.
pub mod input;

use std::any::Any;
use std::time::Duration;

// #![allow(clippy::type_complexity)]
use bevy::window::PrimaryWindow;
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_button_released_plugin::{ButtonReleasedEvent, ButtonsReleasedPlugin};
use input::{
    cell_select_handler, keyboard_input_system, text_input_system, type_handler, SelectEvent,
    TypeEvent,
};

fn main() {
    App::new()
        .add_plugins(ButtonsReleasedPlugin)
        .add_plugins((DefaultPlugins, FrameTimeDiagnosticsPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, (text_update_system, text_color_system))
        .add_event::<SelectEvent>()
        .add_event::<TypeEvent>()
        .add_systems(Update, (keyboard_input_system, cell_select_handler))
        .add_systems(Update, (text_input_system, type_handler))
        .add_systems(Update, button_system)
        // .add_systems(Update, button_system_release)
        // .add_systems(Update, text_input)
        .run();
}

// A unit struct to help identify the FPS UI component, since there may be many Text components
#[derive(Component)]
pub struct FpsText;

// A unit struct to help identify the color-changing Text component
#[derive(Component)]
pub struct ColorText;

#[derive(Component)]
pub struct Position {
    x: f32,
    y: f32,
}

#[derive(Component)]
pub struct Glyph {
    char: char,
    x: f32,
    y: f32,
}

pub struct Grid {
    items: [[Glyph; DEFAULT_COL_COUNT as usize]; DEFAULT_ROW_COUNT as usize],
}

#[derive(Component)]
pub struct Selected;

pub const DEFAULT_CELL_SIZE: f32 = 25.0;
pub const DEFAULT_COL_COUNT: f32 = 10.0;
pub const DEFAULT_ROW_COUNT: f32 = 10.0;
pub const DEFAULT_SCALE: f32 = 1.0;

pub const SELECTED_BG_COLOR: Color = Color::rgb(0.94, 0.97, 1.0);
pub const DEFAUL_BG_COLOR: Color = Color::NONE;

// Name::new("Player"),

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let width = DEFAULT_COL_COUNT * (DEFAULT_CELL_SIZE + 2.0) * DEFAULT_SCALE;
    let height = DEFAULT_ROW_COUNT * (DEFAULT_CELL_SIZE + 2.0) * DEFAULT_SCALE;

    let grid = NodeBundle {
        style: Style {
            display: Display::Grid,
            width: Val::Px(width),
            height: Val::Px(height),

            grid_template_columns: RepeatedGridTrack::flex(10, 1.0),
            grid_template_rows: RepeatedGridTrack::flex(10, 1.0),
            row_gap: Val::Px(1.0),
            column_gap: Val::Px(1.0),

            padding: UiRect::all(Val::Px(10.0)),
            border: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        border_color: Color::GRAY.into(),
        ..default()
    };

    let grid_id = commands.spawn(grid).id();

    for y in 1..=10 {
        for x in 1..=10 {
            let cell = ButtonBundle {
                style: Style {
                    width: Val::Px(DEFAULT_CELL_SIZE),
                    height: Val::Px(DEFAULT_CELL_SIZE),
                    // align_items: AlignItems::Center,
                    // align_content: AlignContent::Center,
                    justify_content: JustifyContent::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                border_color: Color::CRIMSON.into(),
                background_color: Color::NONE.into(),
                ..default()
            };

            let glyph = Glyph {
                char: '.',
                x: x as f32,
                y: y as f32,
            };

            let text = TextBundle::from_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                glyph.char.to_string(),
                TextStyle {
                    // This font is loaded and will be used instead of the default font.
                    font: asset_server.load("fonts/JetBrainsMono-Regular.ttf"),
                    font_size: DEFAULT_CELL_SIZE,
                    ..default()
                },
            )
            .with_text_alignment(TextAlignment::Center);

            let cell_id = commands.spawn((cell, glyph)).id();
            let text_id = commands.spawn(text).id();

            commands.entity(cell_id).push_children(&[text_id]);
            commands.entity(grid_id).push_children(&[cell_id]);
        }
    }
}

// fn button_system_release(
//     mut reader: EventReader<ButtonReleasedEvent>,
//     mut query: Query<(&mut BackgroundColor, &Children)>,
//     mut text_query: Query<&mut Text>,
// ) {
//     for event in reader.read() {
//         if let Ok((mut bg_color, children)) = query.get_mut(**event) {
//             println!("Released");

//             let mut text = text_query.get_mut(children[0]).unwrap();
//             text.sections[0].value = "$".to_string();

//             *bg_color = Color::NONE.into();
//         }
//     }
// }

fn button_system_release(
    mut reader: EventReader<ButtonReleasedEvent>,
    mut query: Query<(&mut BackgroundColor, &Children)>,
    mut text_query: Query<&mut Text>,
) {
    for event in reader.read() {
        // tracing::info!("{:?}", event.);
        println!("Released");
        if let Ok((mut bg_color, children)) = query.get_mut(**event) {
            println!("Released inner");

            // let mut text = text_query.get_mut(children[0]).unwrap();
            // text.sections[0].value = "$".to_string();
            // *bg_color = Color::NONE.into();
        }
    }
}

fn button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            Entity,
            &mut BackgroundColor,
            &Glyph,
            &Children,
        ),
        (Changed<Interaction>, With<Glyph>, Without<Selected>),
    >,
    mut selected_query: Query<(Entity, &mut BackgroundColor), With<Selected>>,
    mut text_query: Query<&mut Text>,
    // mut text_query: Query<(&mut Text, &Parent)>,
    mut commands: Commands,
) {
    for (interaction, entity, mut bg_color, glyph, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        // let (mut text, parent) = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                println!("Pressed");
                println!("{}.{}", glyph.x, glyph.y);

                if let Ok((current_selected, mut current_selected_bg_color)) =
                    selected_query.get_single_mut()
                {
                    *current_selected_bg_color = Color::NONE.into();
                    commands.entity(current_selected).remove::<Selected>();
                }

                commands.entity(entity).insert(Selected);

                text.sections[0].value = "*".to_string();
                *bg_color = SELECTED_BG_COLOR.into();
            }
            Interaction::Hovered => {
                // println!("Hovered");
            }
            Interaction::None => {
                // println!("None");
            }
        }
    }
}

fn text_color_system(time: Res<Time>, mut query: Query<&mut Text, With<Glyph>>) {
    for mut text in &mut query {
        let seconds = time.elapsed_seconds();

        // Update the color of the first and only section.
        text.sections[0].style.color = Color::Rgba {
            red: (1.25 * seconds).sin() / 2.0 + 0.5,
            green: (0.75 * seconds).sin() / 2.0 + 0.5,
            blue: (0.50 * seconds).sin() / 2.0 + 0.5,
            alpha: 1.0,
        };
    }
}

fn text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Update the value of the second section
                text.sections[1].value = format!("{value:.2}");
            }
        }
    }
}

// fn cursor_position(q_windows: Query<&Window, With<PrimaryWindow>>) {
//     // Games typically only have one window (the primary window)
//     if let Some(position) = q_windows.single().cursor_position() {
//         println!("Cursor is inside the primary window, at {:?}", position);
//     } else {
//         println!("Cursor is not in the game window.");
//     }
// }

fn mouse_event(windows: Query<&Window>, buttons: Res<Input<MouseButton>>) {
    let window = windows.single();

    if buttons.just_pressed(MouseButton::Left) {
        // Left button was pressed
        println!("Mouse");
        if let Some(world_position) = window.cursor_position() {
            eprintln!("coords: {}/{}", world_position.x, world_position.y);
        }
    }
    if buttons.just_released(MouseButton::Left) {
        // Left Button was released
    }
    if buttons.pressed(MouseButton::Right) {
        // Right Button is being held down
    }
    // we can check multiple at once with `.any_*`
    if buttons.any_just_pressed([MouseButton::Left, MouseButton::Right]) {
        // Either the left or the right button was just pressed
    }
}

// This system handles changing all buttons color based on mouse interaction
// fn button_system(
//     mut interaction_query: Query<
//         (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
//         (Changed<Interaction>, With<Button>),
//     >,
// ) {

// // This system updates the settings when a new value for a setting is selected, and marks
//     // the button as the one currently selected
//     fn setting_button<T: Resource + Component + PartialEq + Copy>(
//         interaction_query: Query<(&Interaction, &T, Entity), (Changed<Interaction>, With<Button>)>,
//         mut selected_query: Query<(Entity, &mut BackgroundColor), With<SelectedOption>>,
//         mut commands: Commands,
//         mut setting: ResMut<T>,
//     ) {

// fn check_for_collisions(
//     mut commands: Commands,
//     mut scoreboard: ResMut<Scoreboard>,
//     mut ball_query: Query<(&mut Velocity, &Transform), With<Ball>>,
//     collider_query: Query<(Entity, &Transform, Option<&Brick>), With<Collider>>,
//     mut collision_events: EventWriter<CollisionEvent>,
// ) {

// fn my_cursor_system(
//     windows: Query<&Window>,
//     camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
// ) {
//     let window = windows.single();
//     let (camera, camera_transform) = camera_q.single();

//     if let Some(world_position) = window
//         .cursor_position()
//         .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
//     {
//         eprintln!("World coords: {}/{}", world_position.x, world_position.y);
//     }
// }
