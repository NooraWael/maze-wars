use bevy::prelude::*;
use shared::server::ServerMessage;

use crate::network::network::{
    connect_to_server, IncomingMessages, NetworkEvent, NetworkState, OutgoingMessages,
};

// Plugin for menu functionality
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuState>()
            .init_state::<GameState>()
            .add_event::<MenuEvent>()
            .add_systems(
                OnEnter(GameState::MainMenu),
                (setup_main_menu, setup_ui_camera),
            )
            .add_systems(
                OnEnter(GameState::ConnectMenu),
                (setup_connect_menu, setup_ui_camera),
            )
            .add_systems(
                OnEnter(GameState::WaitingMenu),
                (setup_waiting_menu, setup_ui_camera),
            )
            .add_systems(OnEnter(GameState::InGame), (cleanup_ui, cleanup_ui_camera))
            .add_systems(OnExit(GameState::MainMenu), (cleanup_ui, cleanup_ui_camera))
            .add_systems(
                OnExit(GameState::ConnectMenu),
                (cleanup_ui, cleanup_ui_camera),
            )
            .add_systems(
                OnExit(GameState::WaitingMenu),
                (cleanup_ui, cleanup_ui_camera),
            )
            .add_systems(
                Update,
                (
                    handle_button_interactions,
                    handle_text_input,
                    handle_menu_events,
                    handle_connection_status,
                    initialize_input_field,
                    update_waiting_menu_ui, // Add the new system here
                )
                    .run_if(
                        in_state(GameState::MainMenu)
                            .or(in_state(GameState::ConnectMenu))
                            .or(in_state(GameState::WaitingMenu)),
                    ),
            );
    }
}

// Main game states
#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    MainMenu,
    ConnectMenu,
    WaitingMenu,
    InGame,
}

// Menu-specific events
#[derive(Event)]
pub enum MenuEvent {
    PlayClicked,
    ConnectClicked,
    BackClicked,
    IpAddressChanged(String),
    UsernameChanged(String),
}

// Current state of the menu
#[derive(Resource, Default)]
pub struct MenuState {
    pub ip_address: String,
    pub username: String,
    pub connection_status: ConnectionStatus,
    pub error_message: Option<String>,
    pub player_list: Vec<String>,
    pub player_count: u32,
}

// Status of the connection attempt
#[derive(Default, PartialEq)]
pub enum ConnectionStatus {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Failed,
}

// Tag component for menu entities
#[derive(Component)]
struct MenuUI;

// Component for input fields
#[derive(Component, PartialEq, Clone, Copy, Debug)]
enum InputField {
    IpAddress,
    Username,
}

// Component for menu buttons
#[derive(Component)]
enum MenuButton {
    Play,
    Connect,
    Back,
}

#[derive(Component)]
pub struct UiCamera;

#[derive(Component)]
struct FocusedField;

// Add these components to identify UI elements that need updating
#[derive(Component)]
struct PlayerCountText;

#[derive(Component)]
struct PlayerListContainer;

// Function to add to your MenuPlugin implementation
fn setup_ui_camera(mut commands: Commands) {
    // Spawn a 2D camera specifically for the UI
    commands.spawn((Camera2dBundle::default(), UiCamera));
}

// System to setup the main menu
fn setup_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Root node
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            MenuUI,
        ))
        .with_children(|parent| {
            // Game title
            parent.spawn((
                Text::new("MAZE WARS"),
                TextFont {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 80.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(50.0)),
                    ..default()
                },
            ));

            // Play button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(250.0),
                        height: Val::Px(65.0),
                        margin: UiRect::all(Val::Px(10.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    MenuButton::Play,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("PLAY"),
                        TextFont {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 40.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
        });
}

// System to setup the connection menu
fn setup_connect_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    menu_state: Res<MenuState>,
) {
    // Root node
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            MenuUI,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("CONNECT TO SERVER"),
                TextFont {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 50.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(50.0)),
                    ..default()
                },
            ));

            // Form container
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    width: Val::Px(400.0),
                    ..default()
                },))
                .with_children(|parent| {
                    // IP Address label
                    parent.spawn((
                        Text::new("IP Address:"),
                        TextFont {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        Node {
                            margin: UiRect::bottom(Val::Px(5.0)),
                            align_self: AlignSelf::Start,
                            ..default()
                        },
                    ));

                    // IP Address input
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(50.0),
                                border: UiRect::all(Val::Px(2.0)),
                                padding: UiRect::all(Val::Px(10.0)),
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BorderColor(Color::srgb(0.5, 0.5, 0.5)),
                            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                            InputField::IpAddress,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new(if !menu_state.ip_address.is_empty() {
                                    menu_state.ip_address.clone()
                                } else {
                                    "127.0.0.1:2025".to_string()
                                }),
                                TextFont {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 24.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                            ));
                        });

                    // Spacer
                    parent.spawn(Node {
                        height: Val::Px(20.0),
                        ..default()
                    });

                    // Username label
                    parent.spawn((
                        Text::new("Username:"),
                        TextFont {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        Node {
                            margin: UiRect::bottom(Val::Px(5.0)),
                            align_self: AlignSelf::Start,
                            ..default()
                        },
                    ));

                    // Username input
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(50.0),
                                border: UiRect::all(Val::Px(2.0)),
                                padding: UiRect::all(Val::Px(10.0)),
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BorderColor(Color::srgb(0.5, 0.5, 0.5)),
                            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                            InputField::Username,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new(if !menu_state.username.is_empty() {
                                    menu_state.username.clone()
                                } else {
                                    "Player".to_string()
                                }),
                                TextFont {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 24.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                            ));
                        });

                    // Spacer
                    parent.spawn(Node {
                        height: Val::Px(30.0),
                        ..default()
                    });

                    // Error message (if any)
                    if let Some(error) = &menu_state.error_message {
                        parent.spawn((
                            Text::new(error),
                            TextFont {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 0.3, 0.3)),
                            Node {
                                margin: UiRect::bottom(Val::Px(20.0)),
                                ..default()
                            },
                        ));
                    }

                    // Buttons container
                    parent
                        .spawn((Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        },))
                        .with_children(|parent| {
                            // Back button
                            parent
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(120.0),
                                        height: Val::Px(50.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.5, 0.2, 0.2)),
                                    MenuButton::Back,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new("BACK"),
                                        TextFont {
                                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                            font_size: 24.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                    ));
                                });

                            // Connect button
                            parent
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(150.0),
                                        height: Val::Px(50.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.2, 0.5, 0.2)),
                                    MenuButton::Connect,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new("CONNECT"),
                                        TextFont {
                                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                            font_size: 24.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                    ));
                                });
                        });
                });
        });
}

// System to handle button interactions
fn handle_button_interactions(
    mut interaction_query: Query<(&Interaction, &MenuButton), (Changed<Interaction>, With<Button>)>,
    mut menu_events: EventWriter<MenuEvent>,
) {
    for (interaction, button) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            match button {
                MenuButton::Play => {
                    menu_events.send(MenuEvent::PlayClicked);
                }
                MenuButton::Connect => {
                    menu_events.send(MenuEvent::ConnectClicked);
                }
                MenuButton::Back => {
                    menu_events.send(MenuEvent::BackClicked);
                }
            }
        }
    }
}

// Simplified text input handling with key_to_char approach instead of ReceivedCharacter
// Improved text input handling
fn handle_text_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut text_query: Query<(&InputField, &Children)>,
    mut text_components: Query<&mut Text>,
    mut menu_events: EventWriter<MenuEvent>,
    mut menu_state: ResMut<MenuState>,
    mut input_fields: Query<
        (Entity, &InputField, &Interaction, Option<&FocusedField>),
        (With<Button>, Changed<Interaction>),
    >,
    focused_fields: Query<(Entity, &InputField), With<FocusedField>>,
    mut commands: Commands,
) {
    // First, handle focus changes
    for (entity, field, interaction, focused) in input_fields.iter_mut() {
        if *interaction == Interaction::Pressed {
            // This field was just clicked
            info!("Field clicked: {:?}", field);

            // If it's not already focused, give it focus
            if focused.is_none() {
                // Remove focus from all other fields
                for (other_entity, _) in focused_fields.iter() {
                    commands.entity(other_entity).remove::<FocusedField>();
                }

                // Add focus to this field
                commands.entity(entity).insert(FocusedField);
                info!("Focus set to {:?}", field);
            }
        }
    }

    // Now process keyboard input for the focused field
    if let Ok((_, focused_field)) = focused_fields.get_single() {
        let mut changed = false;

        // Handle backspace
        if keyboard_input.just_pressed(KeyCode::Backspace) {
            match focused_field {
                InputField::IpAddress => {
                    if !menu_state.ip_address.is_empty() {
                        menu_state.ip_address.pop();
                        changed = true;
                    }
                }
                InputField::Username => {
                    if !menu_state.username.is_empty() {
                        menu_state.username.pop();
                        changed = true;
                    }
                }
            }
        }

        // Handle character input using key_to_char
        for key in keyboard_input.get_just_pressed() {
            if let Some(c) = key_to_char(*key) {
                match focused_field {
                    InputField::IpAddress => {
                        // For IP address, only allow numbers, dots and colons
                        if c.is_digit(10) || c == '.' || c == ':' {
                            menu_state.ip_address.push(c);
                            changed = true;
                        }
                    }
                    InputField::Username => {
                        // For username, allow alphanumeric and some special chars
                        if c.is_alphanumeric() || c == '_' || c == '-' || c == ' ' {
                            menu_state.username.push(c);
                            changed = true;
                        }
                    }
                }
            }
        }

        // Send events if content changed
        if changed {
            match focused_field {
                InputField::IpAddress => {
                    info!("IP address changed: {}", menu_state.ip_address);
                    menu_events.send(MenuEvent::IpAddressChanged(menu_state.ip_address.clone()));
                }
                InputField::Username => {
                    info!("Username changed: {}", menu_state.username);
                    menu_events.send(MenuEvent::UsernameChanged(menu_state.username.clone()));
                }
            }
        }

        // Update the displayed text
        for (field_type, children) in text_query.iter() {
            if *field_type == *focused_field {
                for &child in children.iter() {
                    if let Ok(mut text) = text_components.get_mut(child) {
                        let new_text = match focused_field {
                            InputField::IpAddress => {
                                if menu_state.ip_address.is_empty() && !changed {
                                    "127.0.0.1:2025".to_string()
                                } else {
                                    menu_state.ip_address.clone()
                                }
                            }
                            InputField::Username => {
                                if menu_state.username.is_empty() && !changed {
                                    "Player".to_string()
                                } else {
                                    menu_state.username.clone()
                                }
                            }
                        };

                        text.0 = new_text;
                    }
                }
            }
        }
    }
}

fn initialize_input_field(
    mut interaction_query: Query<
        (&Interaction, &InputField, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_components: Query<&mut Text>,
    mut menu_state: ResMut<MenuState>,
) {
    for (interaction, field, children) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            // Only initialize if the field is empty
            match field {
                InputField::IpAddress => {
                    if menu_state.ip_address.is_empty() {
                        menu_state.ip_address = "127.0.0.1:2025".to_string();
                    }
                }
                InputField::Username => {
                    if menu_state.username.is_empty() {
                        menu_state.username = "Player".to_string();
                    }
                }
            }

            // Update the displayed text
            for &child in children.iter() {
                if let Ok(mut text) = text_components.get_mut(child) {
                    let new_text = match field {
                        InputField::IpAddress => menu_state.ip_address.clone(),
                        InputField::Username => menu_state.username.clone(),
                    };

                    text.0 = new_text;
                }
            }
        }
    }
}

// System to handle menu events
fn handle_menu_events(
    mut menu_events: EventReader<MenuEvent>,
    mut game_state: ResMut<NextState<GameState>>,
    mut menu_state: ResMut<MenuState>,
    mut outgoing_msgs: ResMut<OutgoingMessages>,
    mut incoming_msgs: ResMut<IncomingMessages>,
    mut network_state: ResMut<NetworkState>,
) {
    for event in menu_events.read() {
        match event {
            MenuEvent::PlayClicked => {
                info!("Play button clicked - transitioning to ConnectMenu");
                game_state.set(GameState::ConnectMenu);
            }
            MenuEvent::ConnectClicked => {
                info!("Connect button clicked");

                // Validate input
                if menu_state.ip_address.is_empty() {
                    info!("IP address is empty");
                    menu_state.error_message = Some("IP address cannot be empty".to_string());
                    return;
                }

                if menu_state.username.is_empty() {
                    info!("Username is empty");
                    menu_state.error_message = Some("Username cannot be empty".to_string());
                    return;
                }

                info!(
                    "Attempting to connect to server at {} with username {}",
                    menu_state.ip_address, menu_state.username
                );

                // Try to connect to server
                menu_state.connection_status = ConnectionStatus::Connecting;

                match connect_to_server(
                    &menu_state.ip_address,
                    menu_state.username.clone(),
                    &mut outgoing_msgs,
                    &mut incoming_msgs,
                    &mut network_state,
                ) {
                    Ok(_) => {
                        // Connection initiated successfully
                        info!("Connection initiated successfully");
                        menu_state.connection_status = ConnectionStatus::Connected;
                        menu_state.error_message = None;

                        // We no longer immediately transition to InGame
                        // Just wait for server confirmation
                        info!("Waiting for server confirmation...");
                    }
                    Err(e) => {
                        // Connection failed
                        error!("Connection failed: {}", e);
                        menu_state.connection_status = ConnectionStatus::Failed;
                        menu_state.error_message = Some(format!("Connection failed: {}", e));

                        // Force update the UI to show the error message
                        let error_msg = format!("Connection failed: {}", e);
                        menu_state.error_message = Some(error_msg.clone());
                        info!("Set error message: {}", error_msg);
                    }
                }
            }
            MenuEvent::BackClicked => {
                info!("Back button clicked - transitioning to MainMenu");
                game_state.set(GameState::MainMenu);
                menu_state.error_message = None;
            }
            MenuEvent::IpAddressChanged(address) => {
                menu_state.ip_address = address.clone();
            }
            MenuEvent::UsernameChanged(username) => {
                menu_state.username = username.clone();
            }
        }
    }
}

// System to handle connection status changes
fn handle_connection_status(
    mut network_events: EventReader<NetworkEvent>,
    mut menu_state: ResMut<MenuState>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for event in network_events.read() {
        match event {
            NetworkEvent::Connected => {
                menu_state.connection_status = ConnectionStatus::Connected;
                menu_state.error_message = None;
            }
            NetworkEvent::Disconnected => {
                menu_state.connection_status = ConnectionStatus::Disconnected;
                game_state.set(GameState::MainMenu);
                menu_state.error_message = Some("Disconnected from server".to_string());
            }
            NetworkEvent::Error(message) => {
                menu_state.connection_status = ConnectionStatus::Failed;
                menu_state.error_message = Some(format!("Error: {}", message));
            }
            NetworkEvent::ReceivedMessage(msg) => {
                // Handle received messages from the server
                match msg {
                    ServerMessage::JoinGameError { message } => {
                        // Handle join game error
                        menu_state.connection_status = ConnectionStatus::Failed;
                        menu_state.error_message = Some(format!("Join failed: {}", message));
                        info!("Join game error: {}", message);
                    }
                    ServerMessage::PlayersInLobby {
                        player_count,
                        players,
                    } => {
                        // Update lobby state and show waiting screen
                        menu_state.connection_status = ConnectionStatus::Connected;
                        menu_state.player_list = players.clone();
                        menu_state.player_count = *player_count;

                        // Transition to waiting menu if not already there
                        if !matches!(
                            *game_state,
                            bevy::prelude::NextState::Pending(GameState::WaitingMenu)
                        ) {
                            game_state.set(GameState::WaitingMenu);
                            info!("Waiting in lobby with {} players", player_count);
                        }
                    }
                    ServerMessage::GameStart => {
                        // Successful join - transition to game
                        info!("Game start confirmed by server");
                        game_state.set(GameState::InGame);
                    }
                    // Handle other relevant server messages
                    _ => {}
                }
            }
        }
    }
}

// System to clean up UI when leaving menus
fn cleanup_ui(mut commands: Commands, query: Query<Entity, With<MenuUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn cleanup_ui_camera(mut commands: Commands, camera_query: Query<Entity, With<UiCamera>>) {
    for camera in camera_query.iter() {
        commands.entity(camera).despawn();
    }
}

// Helper function to convert keys to characters
fn key_to_char(key: KeyCode) -> Option<char> {
    match key {
        KeyCode::KeyA => Some('a'),
        KeyCode::KeyB => Some('b'),
        KeyCode::KeyC => Some('c'),
        KeyCode::KeyD => Some('d'),
        KeyCode::KeyE => Some('e'),
        KeyCode::KeyF => Some('f'),
        KeyCode::KeyG => Some('g'),
        KeyCode::KeyH => Some('h'),
        KeyCode::KeyI => Some('i'),
        KeyCode::KeyJ => Some('j'),
        KeyCode::KeyK => Some('k'),
        KeyCode::KeyL => Some('l'),
        KeyCode::KeyM => Some('m'),
        KeyCode::KeyN => Some('n'),
        KeyCode::KeyO => Some('o'),
        KeyCode::KeyP => Some('p'),
        KeyCode::KeyQ => Some('q'),
        KeyCode::KeyR => Some('r'),
        KeyCode::KeyS => Some('s'),
        KeyCode::KeyT => Some('t'),
        KeyCode::KeyU => Some('u'),
        KeyCode::KeyV => Some('v'),
        KeyCode::KeyW => Some('w'),
        KeyCode::KeyX => Some('x'),
        KeyCode::KeyY => Some('y'),
        KeyCode::KeyZ => Some('z'),
        KeyCode::Digit0 => Some('0'),
        KeyCode::Digit1 => Some('1'),
        KeyCode::Digit2 => Some('2'),
        KeyCode::Digit3 => Some('3'),
        KeyCode::Digit4 => Some('4'),
        KeyCode::Digit5 => Some('5'),
        KeyCode::Digit6 => Some('6'),
        KeyCode::Digit7 => Some('7'),
        KeyCode::Digit8 => Some('8'),
        KeyCode::Digit9 => Some('9'),
        KeyCode::Period => Some('.'),
        KeyCode::Semicolon => Some(':'),
        KeyCode::Space => Some(' '),
        KeyCode::Minus => Some('-'),
        KeyCode::Equal => Some('='),
        _ => None,
    }
}

// System to setup the waiting menu
fn setup_waiting_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    menu_state: Res<MenuState>,
) {
    // Root node
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            MenuUI,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("WAITING FOR GAME START"),
                TextFont {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 50.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));

            // Player count - add PlayerCountText tag
            parent.spawn((
                Text::new(format!("Players in lobby: {}", menu_state.player_count)),
                TextFont {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.9, 0.7)),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
                PlayerCountText,
            ));

            // Player list container - add PlayerListContainer tag
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        width: Val::Px(400.0),
                        padding: UiRect::all(Val::Px(20.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BorderColor(Color::srgb(0.3, 0.3, 0.3)),
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    PlayerListContainer,
                ))
                .with_children(|parent| {
                    // Player list title
                    parent.spawn((
                        Text::new("Players:"),
                        TextFont {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        Node {
                            margin: UiRect::bottom(Val::Px(10.0)),
                            ..default()
                        },
                    ));

                    // Player entries
                    for player in &menu_state.player_list {
                        parent.spawn((
                            Text::new(player),
                            TextFont {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.8, 0.8, 1.0)),
                            Node {
                                margin: UiRect::bottom(Val::Px(5.0)),
                                ..default()
                            },
                        ));
                    }
                });

            // Waiting message
            parent.spawn((
                Text::new("Game will start automatically when ready..."),
                TextFont {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    margin: UiRect::all(Val::Px(30.0)),
                    ..default()
                },
            ));
        });
}

// System to update the player list UI when new data is received
fn update_waiting_menu_ui(
    mut commands: Commands,
    menu_state: Res<MenuState>,
    asset_server: Res<AssetServer>,
    mut player_count_text: Query<&mut Text, With<PlayerCountText>>,
    player_list_container: Query<Entity, With<PlayerListContainer>>,
    game_state: Res<State<GameState>>,
) {
    // Only update if we're in the waiting menu state
    if *game_state.get() == GameState::WaitingMenu {
        // Update player count text
        if let Ok(mut text) = player_count_text.get_single_mut() {
            text.0 = format!("Players in lobby: {}", menu_state.player_count);
        }

        // Update player list
        if let Ok(container) = player_list_container.get_single() {
            // Clear current entries but keep the container
            commands.entity(container).despawn_descendants();

            // Rebuild the player list
            commands.entity(container).with_children(|parent| {
                // Player list title
                parent.spawn((
                    Text::new("Players:"),
                    TextFont {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    Node {
                        margin: UiRect::bottom(Val::Px(10.0)),
                        ..default()
                    },
                ));

                // Player entries
                for player in &menu_state.player_list {
                    parent.spawn((
                        Text::new(player),
                        TextFont {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 1.0)),
                        Node {
                            margin: UiRect::bottom(Val::Px(5.0)),
                            ..default()
                        },
                    ));
                }
            });
        }
    }
}
