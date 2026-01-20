#![windows_subsystem = "windows"]

mod submission_summary;

use crate::submission_summary::{
    ArtDerSequenzierung, CheckedValue, StringValue, SubmissionSummary,
};
use iced::border::Radius;
use iced::font::Weight;
use iced::widget::{button, column, container, row, rule, text, text_input, Row};
use iced::window::Event;
use iced::{
    alignment, application, color, window, Background, Border, Color, Element, Font, Pixels, Task,
};
use iced::{Length, Settings};
use std::cmp::PartialEq;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

fn main() -> iced::Result {
    application(Ui::new, Ui::update, Ui::view)
        .title("Submission Summary Viewer")
        .settings(Settings {
            default_text_size: Pixels::from(13),
            ..Settings::default()
        })
        .resizable(false)
        .window_size((800, 600))
        .subscription(Ui::subscription)
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    PickFile,
    ClearFile,
    ReadFile(Result<PathBuf, ()>),
    Empty,
}

#[derive(Debug, Clone, PartialEq)]
enum Status {
    NoFile,
    FileLoaded,
    ParseError,
}

struct Ui {
    file_path: Option<PathBuf>,
    status: Status,
    submission_summary: Option<SubmissionSummary>,
}

impl Ui {
    fn new() -> Self {
        Self {
            file_path: None,
            status: Status::NoFile,
            submission_summary: None,
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ReadFile(file) => {
                if let Ok(path) = file {
                    self.file_path = Some(path);
                    if let Ok(summary) = self.parse_file() {
                        self.submission_summary = Some(summary);
                        self.status = Status::FileLoaded;
                    } else {
                        self.submission_summary = None;
                        self.status = Status::ParseError;
                    }
                }
                Task::none()
            }
            Message::ClearFile => {
                self.file_path = None;
                self.status = Status::NoFile;
                self.submission_summary = None;
                Task::none()
            }
            Message::PickFile => Task::perform(Self::pick_file(), Message::ReadFile),
            Message::Empty => Task::none(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        fn colored_content_line<'a>(
            name: &str,
            content: &impl CheckedValue,
            color: Color,
        ) -> Row<'a, Message> {
            row![
                text(name.to_string()).width(160),
                text_input(name, &content.to_string())
                    .font(Font::MONOSPACE)
                    .style(move |theme, status| text_input::Style {
                        background: Background::Color(color),
                        placeholder: color!(0x888888),
                        value: color!(0x333333),
                        ..text_input::default(theme, status)
                    })
            ]
            .align_y(alignment::Vertical::Center)
        }

        fn content_line<'a>(name: &str, content: &impl CheckedValue) -> Row<'a, Message> {
            if content.is_invalid() {
                return colored_content_line(name, content, color!(0xFFFFCC));
            }
            colored_content_line(name, content, Color::WHITE)
        }

        let drop_container =
            container(text("Datei hier fallen lassen oder oben auswählen").color(color!(0x777777)))
                .center(Length::Fill)
                .style(|_| container::Style {
                    border: Border {
                        width: 1.0,
                        color: color!(0xCCCCCC),
                        radius: Radius::new(40),
                    },
                    ..container::Style::default()
                });

        column![
            container(
                row![
                    text("Meldebestätigung"),
                    match &self.file_path {
                        Some(path) => match self.status {
                            Status::ParseError =>
                                text(path.to_str().unwrap_or_default()).color(color!(0xFF3333)),
                            _ => text(path.to_str().unwrap_or_default()).color(color!(0x333333)),
                        },
                        _ => text("Keine Datei geladen").color(color!(0x777777)),
                    }
                    .font(Font::MONOSPACE)
                    .width(Length::Fill),
                    match &self.status {
                        Status::FileLoaded => button("x")
                            .style(button::danger)
                            .on_press(Message::ClearFile),
                        _ => button("..").on_press(Message::PickFile),
                    },
                ]
                .spacing(12)
                .align_y(alignment::Vertical::Center)
            )
            .padding(12)
            .style(|_| container::Style {
                background: Some(Background::Color(color!(0xEEEEEE))),
                ..container::Style::default()
            }),
            rule::horizontal(1),
            match &self.submission_summary {
                Some(submission_summary) => {
                    column![
                        container(text("Inhalt der Meldebestätigung").font(Font {
                            weight: Weight::Bold,
                            ..Font::default()
                        })),
                        content_line("Tan", &submission_summary.tan),
                        content_line("Code", &submission_summary.code),
                        row![
                            content_line("Datum", &submission_summary.date),
                            content_line("Laufende Nummer", &submission_summary.counter)
                        ]
                        .spacing(80),
                        content_line("Leistungserbringer", &submission_summary.ik),
                        content_line("Datenknoten", &submission_summary.datacenter),
                        content_line("Typ der Meldung", &submission_summary.typ_der_meldung),
                        content_line("Indikationsbereich", &submission_summary.indikationsbereich),
                        content_line("Kostenträger", &submission_summary.kostentraeger),
                        content_line("Art der Daten", &submission_summary.art_der_daten),
                        if submission_summary
                            .art_der_sequenzierung
                            .eq(&ArtDerSequenzierung::Keine)
                        {
                            colored_content_line(
                                "Art der Sequenzierung",
                                &submission_summary.art_der_sequenzierung,
                                color!(0xFFFFCC),
                            )
                        } else {
                            content_line(
                                "Art der Sequenzierung",
                                &submission_summary.art_der_sequenzierung,
                            )
                        },
                        colored_content_line(
                            "Qualitätskontrolle",
                            &StringValue::new_valid(if submission_summary.accepted {
                                "bestanden"
                            } else {
                                "nicht bestanden"
                            }),
                            if submission_summary.accepted {
                                color!(0xCCFFCC)
                            } else {
                                color!(0xFFCCCC)
                            }
                        ),
                        colored_content_line(
                            "Sha256-Hash",
                            &submission_summary.hash_wert,
                            if submission_summary.valid_hash() {
                                color!(0xCCFFCC)
                            } else {
                                color!(0xFFCCCC)
                            }
                        ),
                    ]
                    .padding(12)
                    .spacing(8)
                }
                _ => match &self.status {
                    Status::ParseError => column![
                        container(text("Fehler beim Lesen der Datei").color(color!(0xFF3333)),)
                            .center(Length::Fill),
                        drop_container
                    ]
                    .padding(80),
                    _ => column![drop_container].padding(80),
                },
            }
        ]
        .into()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        window::events().map(|(_, event)| match event {
            Event::FileDropped(file) => Message::ReadFile(Ok(file)),
            _ => Message::Empty,
        })
    }

    //

    async fn pick_file() -> Result<PathBuf, ()> {
        let path = rfd::AsyncFileDialog::new()
            .set_title("Open file...")
            .pick_file()
            .await
            .ok_or(())?;

        Ok(path.into())
    }

    fn parse_file(&self) -> Result<SubmissionSummary, ()> {
        match fs::read_to_string(self.file_path.clone().unwrap_or_default()).map_err(|_| ()) {
            Ok(content) => Ok(SubmissionSummary::from_str(&content)?),
            Err(()) => Err(()),
        }
    }
}
