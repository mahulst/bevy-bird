use bevy::app::AppExit;

use crate::prelude::*;

pub struct ButtonMaterials {
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
    pressed: Handle<ColorMaterial>,
}

impl FromWorld for ButtonMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            normal: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
            hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
            pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
        }
    }
}

pub fn close_menu(
    mut commands: Commands,
    menu_query: Query<(Entity), (Or<(With<MenuItem>, With<GameOver>)>)>,
) {
    for (entity) in menu_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn menu(
    button_materials: Res<ButtonMaterials>,
    mut state: ResMut<State<AppState>>,
    mut query: Query<
        (
            &Interaction,
            &mut Handle<ColorMaterial>,
            &Children,
            &ActionButton,
        ),
        (Changed<Interaction>, With<ActionButton>),
    >,
    mut text_query: Query<&mut Text>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    for (interaction, mut material, children, action) in query.iter_mut() {
        let mut text = text_query.get_mut(children[0]).expect("Get button text");
        match *interaction {
            Interaction::Clicked => match *action {
                ActionButton(Actions::Start) => {
                    state.set(AppState::restart()).unwrap();
                }
                ActionButton(Actions::Exit) => {
                    app_exit_events.send(AppExit);
                }
            },
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}

pub fn display_game_over(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(UiCameraBundle::default());
    commands
        .spawn_bundle(TextBundle {
            text: Text::with_section(
                "Game Over",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 80.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                    ..Default::default()
                },
                TextAlignment {
                    horizontal: HorizontalAlign::Right,
                    ..Default::default()
                },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(5.0),
                    left: Val::Percent(50.),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(GameOver);
}

pub fn setup_menu(
    mut commands: Commands,
    button_materials: Res<ButtonMaterials>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn_bundle(UiCameraBundle::default());
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.), Val::Px(65.0)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: button_materials.normal.clone(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "Play",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                        ..Default::default()
                    },
                    Default::default(),
                ),
                ..Default::default()
            });
        })
        .insert(ActionButton(Actions::Start))
        .insert(MenuItem);

    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.), Val::Px(65.0)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: button_materials.normal.clone(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "Exit",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                        ..Default::default()
                    },
                    Default::default(),
                ),
                ..Default::default()
            });
        })
        .insert(ActionButton(Actions::Exit))
        .insert(MenuItem);
}

pub fn update_score(mut query: Query<(&mut Text), (With<ScoreText>)>, score: Res<Score>) {
    if let Ok(mut text) = query.single_mut() {
        text.sections[0].value = score.0.to_string();
    }
}