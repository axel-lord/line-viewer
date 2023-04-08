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
use std::{fs::File, io, path::PathBuf};
use tap::Pipe;

#[derive(Parser)]
struct Cli {
    file_path: PathBuf,
}

struct App(anyhow::Result<Vec<&'static str>>, String);
impl App {
    fn construct() -> anyhow::Result<(Vec<&'static str>, String)> {
        let Cli { file_path } =
            Cli::try_parse().context("failed to parse input, was no filename given?")?;

        io::read_to_string(File::open(&file_path).context(
            "failed to open file, does it exist and do you have the proper permissions?",
        )?)
        .context("failed to read file to memory")
        .map(|s| {
            (
                s.into_boxed_str().pipe(Box::leak).lines().collect(),
                format!("\"{}\" loaded", file_path.display()),
            )
        })
    }
}
impl Sandbox for App {
    type Message = &'static str;

    fn new() -> Self {
        match Self::construct() {
            Ok((lines, status)) => Self(Ok(lines), status),
            Err(err) => Self(Err(err), "error occured".into()),
        }
    }

    fn title(&self) -> String {
        String::from("Line View")
    }

    fn update(&mut self, message: Self::Message) {
        if let Err(e) = cli_clipboard::set_contents(message.into()) {
            println!("failed to open \"{message}\", {e}");
        } else {
            self.1 = format!("\"{message}\" copied to clipboard");
        }
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        Column::new()
            .push(match self.0 {
                Ok(ref lines) => lines
                    .iter()
                    .fold(Column::new(), |col, line| {
                        col.push(button(*line).style(theme::Button::Text).on_press(*line))
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
