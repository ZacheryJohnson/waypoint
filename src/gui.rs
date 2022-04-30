use iced::{
    button, scrollable, slider, text_input, Button, Checkbox,
    Color, Column, Container, Element, Image, Length, Radio, Row, Sandbox,
    Scrollable, Settings, Slider, Space, Text, TextInput
};

use crate::app::{*};

pub struct WaypointGui {
    app: WaypointApp,
    _tmp_start: button::State,
    _tmp_logs: button::State,
    _tmp_scrl: scrollable::State,
    _tmp_service_id: ServiceId,
}

impl WaypointGui {
    pub fn start() -> iced::Result {
        WaypointGui::run(Settings::default())
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    AddServer,
    StartService,
    ReadLogs,
}

impl Sandbox for WaypointGui {
    type Message = Message;

    fn new() -> WaypointGui {
        WaypointGui {
            app: WaypointApp::new(),
            _tmp_start: Default::default(),
            _tmp_logs: Default::default(),
            _tmp_scrl: Default::default(),
            _tmp_service_id: Default::default(),
        }
    }

    fn title(&self) -> String {
        String::from("Waypoint")
    }

    fn update(&mut self, event: Message) {
        match event {
            Message::AddServer => { println!("AddServer pressed!"); },
            Message::StartService => { 
                let service_id = self.app.start_service("foobar");
                self._tmp_service_id = service_id;
            },
            Message::ReadLogs => {
                if let Some(logs) = self.app.get_service_logs(&self._tmp_service_id) {
                    let logs = &*logs.lock().unwrap();
                    for logline in logs {
                        print!("{}", logline)
                    }
                } else {
                    println!("Failed to get logs...");
                }
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let app = self;

        let scrollable = Scrollable::new(&mut app._tmp_scrl)
            .push(Text::new("label"));

        let content: Element<_> = Column::new()
            .max_width(540)
            .spacing(20)
            .padding(20)
            .push(Button::new(&mut app._tmp_start, Text::new("Start")).on_press(Message::StartService))
            .push(Button::new(&mut app._tmp_logs, Text::new("Read Logs")).on_press(Message::ReadLogs))
            .push(scrollable)
            .into();

        Container::new(content)
            .height(Length::Fill)
            .center_y()
            .into()
    }
}

mod style {
    use iced::{button, Background, Color, Vector};

    pub enum Button {
        Primary,
        Secondary,
    }

    impl button::StyleSheet for Button {
        fn active(&self) -> button::Style {
            button::Style {
                background: Some(Background::Color(match self {
                    Button::Primary => Color::from_rgb(0.11, 0.42, 0.87),
                    Button::Secondary => Color::from_rgb(0.5, 0.5, 0.5),
                })),
                border_radius: 12.0,
                shadow_offset: Vector::new(1.0, 1.0),
                text_color: Color::from_rgb8(0xEE, 0xEE, 0xEE),
                ..button::Style::default()
            }
        }

        fn hovered(&self) -> button::Style {
            button::Style {
                text_color: Color::WHITE,
                shadow_offset: Vector::new(1.0, 2.0),
                ..self.active()
            }
        }
    }
}