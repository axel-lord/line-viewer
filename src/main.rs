#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Context;
use clap::Parser;
use iced::{
    alignment::Horizontal,
    theme,
    widget::{button, container, horizontal_rule, scrollable, text, Column},
    window::{self, icon::Icon},
    Color, Element, Length, Padding, Sandbox, Settings,
};
use std::{fmt::Write, fs::File, io, path::PathBuf, process::Command};
use tap::Pipe;

#[derive(Parser)]
struct Cli {
    file_path: PathBuf,
}

#[derive(Default)]
struct Action {
    prefix: Vec<String>,
    suffix: Vec<String>,
}

struct App(anyhow::Result<(Vec<String>, Action)>, String);

impl App {
    fn parse(body: &str) -> (Vec<String>, Action) {
        let mut action = Action::default();
        let mut lines = Vec::new();

        for line in body.lines() {
            if let Some(line) = line.strip_prefix('#') {
                if let Some(prefix) = line.strip_prefix("-pre ") {
                    action.prefix.push(prefix.into());
                } else if let Some(suffix) = line.strip_prefix("-suf ") {
                    action.suffix.push(suffix.into())
                }
            } else if !line.trim().is_empty() {
                lines.push(line.into());
            }
        }

        (lines, action)
    }

    fn construct() -> anyhow::Result<(Vec<String>, Action, String)> {
        let Cli { file_path } =
            Cli::try_parse().context("failed to parse input, was no filename given?")?;

        io::read_to_string(File::open(&file_path).context(
            "failed to open file, does it exist and do you have the proper permissions?",
        )?)
        .context("failed to read file to memory")
        .map(|s| {
            let (lines, action) = App::parse(&s);
            (lines, action, format!("\"{}\" loaded", file_path.display()))
        })
    }
}
impl Sandbox for App {
    type Message = String;

    fn new() -> Self {
        match Self::construct() {
            Ok((lines, action, status)) => Self(Ok((lines, action)), status),
            Err(err) => Self(Err(err), "error occured".into()),
        }
    }

    fn title(&self) -> String {
        String::from("Line View")
    }

    fn update(&mut self, message: Self::Message) {
        if let Ok((_, Action { prefix, suffix })) = &self.0 {
            let iter = prefix
                .iter()
                .skip(1)
                .chain(Some(&message))
                .chain(suffix.iter());

            self.1 = prefix
                .first()
                .into_iter()
                .chain(iter.clone())
                .fold(String::new(), |mut a, b| {
                    _ = write!(&mut a, " {b}");
                    a
                })
                .trim()
                .into();

            _ = iter
                .fold(
                    &mut Command::new(prefix.first().map(String::as_str).unwrap_or("echo")),
                    |cmd, arg| cmd.arg(arg),
                )
                .spawn();
        }
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        Column::new()
            .push(match &self.0 {
                Ok((lines, _)) => lines
                    .iter()
                    .fold(Column::new(), |col, line| {
                        col.push(
                            button(text(line))
                                .style(theme::Button::Text)
                                .on_press(line.clone()),
                        )
                    })
                    .pipe(container)
                    .width(Length::Fill)
                    .pipe(scrollable)
                    .height(Length::Fill)
                    .pipe(Element::from),
                Err(ref err) => text(err)
                    .style(theme::Text::Color(Color::from_rgb8(233, 50, 0)))
                    .pipe(container)
                    .center_x()
                    .center_y()
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .padding(Padding::new(3.0))
                    .pipe(Element::from),
            })
            .push(horizontal_rule(1))
            .push(
                text(&self.1)
                    .pipe(container)
                    .width(Length::Fill)
                    .height(Length::Shrink)
                    .align_x(Horizontal::Right)
                    .padding(Padding::new(3.0))
                    .pipe(Element::from),
            )
            .into()
    }
}

fn main() -> iced::Result {
    App::run(Settings {
        window: window::Settings {
            icon: Some(
                Icon::from_file_data(include_bytes!(concat!(env!("OUT_DIR"), "/icon.png")), None)
                    .expect("image data should exist"),
            ),
            ..Default::default()
        },
        ..Default::default()
    })
}
