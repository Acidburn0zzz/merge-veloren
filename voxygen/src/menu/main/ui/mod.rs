mod connecting;
//mod disclaimer;
mod login;
mod servers;

use crate::{
    i18n::{i18n_asset_key, Localization},
    render::Renderer,
    ui::{
        self,
        fonts::IcedFonts as Fonts,
        ice::{style, widget, Element, Font, IcedUi as Ui},
        img_ids::{ImageGraphic, VoxelGraphic},
        Graphic,
    },
    GlobalState,
};
use iced::{text_input, Column, Container, HorizontalAlignment, Length};
//ImageFrame, Tooltip,
use crate::settings::Settings;
use common::assets::Asset;
use image::DynamicImage;
use rand::{seq::SliceRandom, thread_rng};
use std::time::Duration;

// TODO: what is this? (showed up in rebase)
//const COL1: Color = Color::Rgba(0.07, 0.1, 0.1, 0.9);

// UI Color-Theme
/*const UI_MAIN: Color = Color::Rgba(0.61, 0.70, 0.70, 1.0); // Greenish Blue
const UI_HIGHLIGHT_0: Color = Color::Rgba(0.79, 1.09, 1.09, 1.0);*/

pub const TEXT_COLOR: iced::Color = iced::Color::from_rgb(1.0, 1.0, 1.0);
pub const DISABLED_TEXT_COLOR: iced::Color = iced::Color::from_rgba(1.0, 1.0, 1.0, 0.2);

image_ids_ice! {
    struct Imgs {
        <VoxelGraphic>
        v_logo: "voxygen.element.v_logo",

        <ImageGraphic>
        bg: "voxygen.background.bg_main",
        banner: "voxygen.element.frames.banner",
        banner_bottom: "voxygen.element.frames.banner_bottom",
        banner_top: "voxygen.element.frames.banner_top",
        button: "voxygen.element.buttons.button",
        button_hover: "voxygen.element.buttons.button_hover",
        button_press: "voxygen.element.buttons.button_press",
        input_bg: "voxygen.element.misc_bg.textbox",
        loading_art: "voxygen.element.frames.loading_screen.loading_bg",
        loading_art_l: "voxygen.element.frames.loading_screen.loading_bg_l",
        loading_art_r: "voxygen.element.frames.loading_screen.loading_bg_r",
    }
}

// Randomly loaded background images
const BG_IMGS: [&str; 16] = [
    "voxygen.background.bg_1",
    "voxygen.background.bg_2",
    "voxygen.background.bg_3",
    "voxygen.background.bg_4",
    "voxygen.background.bg_5",
    "voxygen.background.bg_6",
    "voxygen.background.bg_7",
    "voxygen.background.bg_8",
    "voxygen.background.bg_9",
    //"voxygen.background.bg_10",
    "voxygen.background.bg_11",
    //"voxygen.background.bg_12",
    "voxygen.background.bg_13",
    //"voxygen.background.bg_14",
    "voxygen.background.bg_15",
    "voxygen.background.bg_16",
];

pub enum Event {
    LoginAttempt {
        username: String,
        password: String,
        server_address: String,
    },
    CancelLoginAttempt,
    #[cfg(feature = "singleplayer")]
    StartSingleplayer,
    Quit,
    //DisclaimerClosed, TODO: remove all traces?
    //DisclaimerAccepted,
    AuthServerTrust(String, bool),
}

pub struct LoginInfo {
    pub username: String,
    pub password: String,
    pub server: String,
}

enum ConnectionState {
    InProgress {
        status: String,
    },
    AuthTrustPrompt {
        auth_server: String,
        msg: String,
        // To remember when we switch back
        status: String,
    },
}
impl ConnectionState {
    fn take_status_string(&mut self) -> String {
        std::mem::take(match self {
            Self::InProgress { status } => status,
            Self::AuthTrustPrompt { status, .. } => status,
        })
    }
}

enum Screen {
    /*Disclaimer {
        screen: disclaimer::Screen,
    },*/
    Login {
        screen: login::Screen,
        // Error to display in a box
        error: Option<String>,
    },
    Servers {
        screen: servers::Screen,
    },
    Connecting {
        screen: connecting::Screen,
        connection_state: ConnectionState,
    },
}

struct Controls {
    fonts: Fonts,
    imgs: Imgs,
    bg_img: widget::image::Handle,
    i18n: std::sync::Arc<Localization>,
    // Voxygen version
    version: String,

    selected_server_index: Option<usize>,
    login_info: LoginInfo,

    time: f32,

    screen: Screen,
}

#[derive(Clone)]
enum Message {
    Quit,
    Back,
    ShowServers,
    #[cfg(feature = "singleplayer")]
    Singleplayer,
    Multiplayer,
    Username(String),
    Password(String),
    Server(String),
    ServerChanged(usize),
    FocusPassword,
    CancelConnect,
    TrustPromptAdd,
    TrustPromptCancel,
    CloseError,
    /*CloseDisclaimer,
     *AcceptDisclaimer, */
}

impl Controls {
    fn new(
        fonts: Fonts,
        imgs: Imgs,
        bg_img: widget::image::Handle,
        i18n: std::sync::Arc<Localization>,
        settings: &Settings,
    ) -> Self {
        let version = format!(
            "{}-{}",
            env!("CARGO_PKG_VERSION"),
            common::util::GIT_VERSION.to_string()
        );

        let screen = /* if settings.show_disclaimer {
            Screen::Disclaimer {
                screen: disclaimer::Screen::new(),
            }
        } else { */
            Screen::Login {
                screen: login::Screen::new(),
                error: None,
            };
        //};

        let login_info = LoginInfo {
            username: settings.networking.username.clone(),
            password: String::new(),
            server: settings.networking.default_server.clone(),
        };
        let selected_server_index = settings
            .networking
            .servers
            .iter()
            .position(|f| f == &login_info.server);

        Self {
            fonts,
            imgs,
            bg_img,
            i18n,
            version,

            selected_server_index,
            login_info,

            time: 0.0,

            screen,
        }
    }

    fn view(&mut self, settings: &Settings, dt: f32) -> Element<Message> {
        self.time = self.time + dt;

        // TODO: consider setting this as the default in the renderer
        let button_style = style::button::Style::new(self.imgs.button)
            .hover_image(self.imgs.button_hover)
            .press_image(self.imgs.button_press)
            .text_color(TEXT_COLOR)
            .disabled_text_color(DISABLED_TEXT_COLOR);

        let version = iced::Text::new(&self.version)
            .size(self.fonts.cyri.scale(15))
            .width(Length::Fill)
            .horizontal_alignment(HorizontalAlignment::Right);

        let bg_img = if matches!(&self.screen, Screen::Connecting {..}) {
            self.bg_img
        } else {
            self.imgs.bg
        };

        // TODO: make any large text blocks scrollable so that if the area is to
        // small they can still be read
        let content = match &mut self.screen {
            //Screen::Disclaimer { screen } => screen.view(&self.fonts, &self.i18n, button_style),
            Screen::Login { screen, error } => screen.view(
                &self.fonts,
                &self.imgs,
                &self.login_info,
                error.as_deref(),
                &self.i18n,
                button_style,
            ),
            Screen::Servers { screen } => screen.view(
                &self.fonts,
                &settings.networking.servers,
                self.selected_server_index,
                &self.i18n,
                button_style,
            ),
            Screen::Connecting {
                screen,
                connection_state,
            } => screen.view(
                &self.fonts,
                &connection_state,
                self.time,
                &self.i18n,
                button_style,
            ),
        };

        Container::new(
            Column::with_children(vec![version.into(), content])
                .spacing(3)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .style(style::container::Style::image(bg_img))
        .padding(3)
        .into()
    }

    fn update(&mut self, message: Message, events: &mut Vec<Event>, settings: &Settings) {
        let servers = &settings.networking.servers;

        match message {
            Message::Quit => events.push(Event::Quit),
            Message::Back => {
                self.screen = Screen::Login {
                    screen: login::Screen::new(),
                    error: None,
                };
            },
            Message::ShowServers => {
                if matches!(&self.screen, Screen::Login {..}) {
                    self.selected_server_index =
                        servers.iter().position(|f| f == &self.login_info.server);
                    self.screen = Screen::Servers {
                        screen: servers::Screen::new(),
                    };
                }
            },
            #[cfg(feature = "singleplayer")]
            Message::Singleplayer => {
                self.screen = Screen::Connecting {
                    screen: connecting::Screen::new(),
                    connection_state: ConnectionState::InProgress {
                        status: [self.i18n.get("main.creating_world"), "..."].concat(),
                    },
                };

                events.push(Event::StartSingleplayer);
            },
            Message::Multiplayer => {
                self.screen = Screen::Connecting {
                    screen: connecting::Screen::new(),
                    connection_state: ConnectionState::InProgress {
                        status: [self.i18n.get("main.connecting"), "..."].concat(),
                    },
                };

                events.push(Event::LoginAttempt {
                    username: self.login_info.username.clone(),
                    password: self.login_info.password.clone(),
                    server_address: self.login_info.server.clone(),
                });
            },
            Message::Username(new_value) => self.login_info.username = new_value,
            Message::Password(new_value) => self.login_info.password = new_value,
            Message::Server(new_value) => {
                self.login_info.server = new_value;
            },
            Message::ServerChanged(new_value) => {
                self.selected_server_index = Some(new_value);
                self.login_info.server = servers[new_value].clone();
            },
            Message::FocusPassword => {
                if let Screen::Login { screen, .. } = &mut self.screen {
                    screen.banner.password = text_input::State::focused();
                    screen.banner.username = text_input::State::new();
                }
            },
            Message::CancelConnect => {
                self.exit_connect_screen();
                events.push(Event::CancelLoginAttempt);
            },
            msg @ Message::TrustPromptAdd | msg @ Message::TrustPromptCancel => {
                if let Screen::Connecting {
                    connection_state, ..
                } = &mut self.screen
                {
                    if let ConnectionState::AuthTrustPrompt {
                        auth_server,
                        status,
                        ..
                    } = connection_state
                    {
                        let auth_server = std::mem::take(auth_server);
                        let status = std::mem::take(status);
                        let added = matches!(msg, Message::TrustPromptAdd);

                        *connection_state = ConnectionState::InProgress { status };
                        events.push(Event::AuthServerTrust(auth_server, added));
                    }
                }
            },
            Message::CloseError => {
                if let Screen::Login { error, .. } = &mut self.screen {
                    *error = None;
                }
            },
            //Message::CloseDisclaimer => {
            //   events.push(Event::DisclaimerClosed);
            //},
            /*Message::AcceptDisclaimer => {
                if let Screen::Disclaimer { .. } = &self.screen {
                    events.push(Event::DisclaimerAccepted);
                    self.screen = Screen::Login {
                        screen: login::Screen::new(),
                        error: None,
                    };
                }
            },*/
        }
    }

    // Connection successful of failed
    fn exit_connect_screen(&mut self) {
        if matches!(&self.screen, Screen::Connecting {..}) {
            self.screen = Screen::Login {
                screen: login::Screen::new(),
                error: None,
            }
        }
    }

    fn auth_trust_prompt(&mut self, auth_server: String) {
        if let Screen::Connecting {
            connection_state, ..
        } = &mut self.screen
        {
            let msg = format!(
                "Warning: The server you are trying to connect to has provided this \
                 authentication server address:\n\n{}\n\nbut it is not in your list of trusted \
                 authentication servers.\n\nMake sure that you trust this site and owner to not \
                 try and bruteforce your password!",
                &auth_server
            );

            *connection_state = ConnectionState::AuthTrustPrompt {
                auth_server,
                msg,
                status: connection_state.take_status_string(),
            };
        }
    }

    fn connection_error(&mut self, error: String) {
        if matches!(&self.screen, Screen::Connecting {..}) {
            self.screen = Screen::Login {
                screen: login::Screen::new(),
                error: Some(error),
            }
        }
    }
}

pub struct MainMenuUi {
    ui: Ui,
    // TODO: re add this
    // tip_no: u16,
    controls: Controls,
}

impl<'a> MainMenuUi {
    pub fn new(global_state: &mut GlobalState) -> Self {
        // Load language
        let i18n = Localization::load_expect(&i18n_asset_key(
            &global_state.settings.language.selected_language,
        ));

        // TODO: don't add default font twice
        let font = {
            use std::io::Read;
            let mut buf = Vec::new();
            common::assets::load_file("voxygen.font.haxrcorp_4089_cyrillic_altgr_extended", &[
                "ttf",
            ])
            .unwrap()
            .read_to_end(&mut buf)
            .unwrap();
            Font::try_from_vec(buf).unwrap()
        };

        let mut ui = Ui::new(&mut global_state.window, font).unwrap();

        let fonts = Fonts::load(&i18n.fonts, &mut ui).expect("Impossible to load fonts");

        let bg_img_spec = BG_IMGS.choose(&mut thread_rng()).unwrap();

        let controls = Controls::new(
            fonts,
            Imgs::load(&mut ui).expect("Failed to load images"),
            ui.add_graphic(Graphic::Image(DynamicImage::load_expect(bg_img_spec), None)),
            i18n,
            &global_state.settings,
        );

        Self { ui, controls }
    }

    pub fn auth_trust_prompt(&mut self, auth_server: String) {
        self.controls.auth_trust_prompt(auth_server);
    }

    pub fn show_info(&mut self, msg: String) { self.controls.connection_error(msg); }

    pub fn connected(&mut self) { self.controls.exit_connect_screen(); }

    pub fn cancel_connection(&mut self) { self.controls.exit_connect_screen(); }

    pub fn handle_event(&mut self, event: ui::ice::Event) { self.ui.handle_event(event); }

    pub fn maintain(&mut self, global_state: &mut GlobalState, dt: Duration) -> Vec<Event> {
        let mut events = Vec::new();

        let (messages, _) = self.ui.maintain(
            self.controls.view(&global_state.settings, dt.as_secs_f32()),
            global_state.window.renderer_mut(),
        );

        messages.into_iter().for_each(|message| {
            self.controls
                .update(message, &mut events, &global_state.settings)
        });

        events
    }

    pub fn render(&self, renderer: &mut Renderer) { self.ui.render(renderer); }
}
