/// Menu is based on `bevy_quickmenu` with `Screens` and `Actions` around YourGame Configuration
/// struct `GameCfg`
use crate::GameState;
use bevy::window::PrimaryWindow;
use bevy::{app::AppExit, prelude::*};
use bevy_quickmenu::{style::Stylesheet, *};

/// `Screens` will hold different menu structures. This decides what will be shown in the menu
/// panel. Atleast one of them will be present at any given time.
/// - During `GameState::Game` `Game` screen will be active
/// - During `GameState::Menu`
///   - `NewGame` at the start of the game
///   - `Pause` when a game is still going on in background
///   - `GameOver` when a game is over
/// - During any state, and for some of the screens, sub-screens like `Num` might be active
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
enum Screens {
    Game,
    Pause,
    NewGame,
    GameOver,
    /// Sub screens
    Num,
}

/// `Actions` will hold button actions
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Actions {
    Resume,
    Pause,
    #[cfg(not(target_arch = "wasm32"))]
    Quit,
    NewGame,
    SetBoolean,
    SetNum(u8),
}

impl ActionTrait for Actions {
    type State = GameCfg;
    type Event = Self;
    fn handle(&self, state: &mut Self::State, event_writer: &mut EventWriter<Self::Event>) {
        match self {
            Self::Pause | Self::Resume => event_writer.send(*self),
            #[cfg(not(target_arch = "wasm32"))]
            Self::Quit => event_writer.send(*self),
            Self::NewGame => {
                state.new_game = true;
                event_writer.send(*self)
            }
            Self::SetBoolean => state.boolean ^= true,
            Self::SetNum(x) => state.num = *x,
        }
    }
}
impl ScreenTrait for Screens {
    type Action = Actions;
    type State = GameCfg;
    fn resolve(
        &self,
        state: &<<Self as ScreenTrait>::Action as bevy_quickmenu::ActionTrait>::State,
    ) -> bevy_quickmenu::Menu<Self> {
        let num_actions =
            |n| MenuItem::action(format!("{n}"), Actions::SetNum(n)).checked(state.num == n);
        Menu::new(
            format!("{self:?}"),
            match self {
                Self::Pause => vec![
                    MenuItem::headline("Paused"),
                    MenuItem::action("Resume", Actions::Resume),
                    MenuItem::screen("New Game", Screens::NewGame),
                    #[cfg(not(target_arch = "wasm32"))]
                    MenuItem::action("Quit", Actions::Quit),
                ],
                Self::Game => vec![MenuItem::action("Pause", Actions::Pause)],
                Self::GameOver => vec![
                    MenuItem::headline("Game Over"),
                    MenuItem::screen("New Game", Screens::NewGame),
                    #[cfg(not(target_arch = "wasm32"))]
                    MenuItem::action("Quit", Actions::Quit),
                ],
                Self::NewGame => vec![
                    MenuItem::headline("YourGame"),
                    MenuItem::action("Start a New Game", Actions::NewGame),
                    MenuItem::label("Configuration"),
                    MenuItem::action("Boolean", Actions::SetBoolean).checked(state.boolean),
                    MenuItem::screen("Num", Screens::Num),
                ],
                Self::Num => [MenuItem::headline("Num")]
                    .into_iter()
                    .chain((3..6).map(|x| num_actions(x)))
                    .collect(),
            },
        )
    }
}

/// Resource to hold the Configurations for `YourGame`
#[derive(Resource, Clone, Copy)]
pub struct GameCfg {
    pub boolean: bool,
    pub new_game: bool,
    pub outcome: Option<bool>,
    pub num: u8,
}
impl Default for GameCfg {
    fn default() -> Self {
        Self {
            boolean: true,
            new_game: false,
            outcome: None,
            num: 3,
        }
    }
}

/// Sets `Screens` for the quickmenu, window title
fn menu(
    mut commands: Commands,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    cfg: Res<GameCfg>,
    state: Res<State<GameState>>,
) {
    let mut window = window.get_single_mut().unwrap();

    let (title, screen, position_type) = if state.0 == GameState::Game {
        ("YourGame", Screens::Game, PositionType::Absolute)
    } else if cfg.outcome.is_some() {
        ("YourGame - GameOver", Screens::GameOver, default())
    } else {
        ("YourGame - Paused", Screens::Pause, default())
    };

    window.title = title.to_string();
    let sheet = Stylesheet::default()
        .with_background(BackgroundColor(Color::BLACK))
        .with_style(Style {
            position_type,
            ..default()
        });

    commands.insert_resource(MenuState::new(*cfg, screen, Some(sheet)))
}
fn handle_events(
    mut action_event: EventReader<Actions>,
    #[cfg(not(target_arch = "wasm32"))] mut app_event: EventWriter<AppExit>,
    mut commands: Commands,
    menu_state: Option<Res<MenuState<Screens>>>,
) {
    if let Some(menu_state) = menu_state {
        if !action_event.is_empty() {
            commands.insert_resource(*menu_state.state());
        }
    }
    for event in action_event.iter() {
        match event {
            Actions::Resume | Actions::NewGame => {
                commands.insert_resource(NextState(Some(GameState::Game)))
            }
            Actions::Pause => commands.insert_resource(NextState(Some(GameState::Menu))),
            #[cfg(not(target_arch = "wasm32"))]
            Actions::Quit => app_event.send(AppExit),
            _ => (),
        }
    }
}

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(QuickMenuPlugin::<Screens>::new())
            .add_event::<Actions>()
            .insert_resource(MenuState::new(
                GameCfg::default(),
                Screens::NewGame,
                Some(Stylesheet::default().with_background(BackgroundColor(Color::BLACK))),
            ))
            // For the Quick Menu
            .add_startup_system(|mut commands: Commands| {
                commands.spawn(Camera2dBundle::default());
            })
            .add_system(menu.in_schedule(OnEnter(GameState::Game)))
            .add_system(menu.in_schedule(OnExit(GameState::Game)))
            .add_system(handle_events);
    }
}
