#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod extensions;
pub mod history;
pub mod line_view;
pub mod message;
pub mod state;

use anyhow::Context;
use clap::Parser;
use extensions::ColRowExt;
use history::History;
use iced::{
    alignment::Horizontal,
    theme,
    widget::{
        button, checkbox, container, horizontal_rule, scrollable, text, text_input, Column, Row,
    },
    window, Alignment, Color, Element, Font, Length, Padding, Sandbox, Settings,
};
use line_view::LineView;
use message::{ListType, Message};
use state::State;
use std::{fs::File, io, path::PathBuf};
use tap::Pipe;

#[derive(Parser)]
struct Cli {
    file_path: PathBuf,
}

struct App {
    state: anyhow::Result<State>,
    status: String,
}

impl App {
    fn save(state: &State) -> String {
        state.save().map_or_else(
            |err| {
                eprintln!("failed to save contents\n{err:#?}");
                format!(
                    "could not save contents to \"{}\"",
                    state.file_path.display()
                )
            },
            |()| format!("saved contents to \"{}\"", state.file_path.display()),
        )
    }

    fn run_default() -> iced::Result {
        App::run(Settings {
            default_font: Font {
                monospaced: true,
                ..Font::default()
            },
            window: window::Settings {
                icon: Some(
                    window::icon::from_file_data(
                        include_bytes!(concat!(env!("OUT_DIR"), "/icon.png")),
                        None,
                    )
                    .expect("image data should exist"),
                ),
                ..Default::default()
            },
            ..Default::default()
        })
    }
}

impl Sandbox for App {
    type Message = Message;

    fn title(&self) -> String {
        String::from("Line View")
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }

    fn new() -> Self {
        Cli::try_parse()
            .context("failed to parse input, was no filename given?")
            .and_then(|cli| {
                File::open(&cli.file_path).context(
                    "failed to open file, does it exist and do you have the proper permissions?",
                ).map(move |file| (file, cli.file_path))
            })
            .and_then(|(file, file_path)| {
                io::read_to_string(file)
                    .context("failed to read file to memory")
                    .map(move |text| (text, file_path))
            })
            .map_or_else(
                |err| Self {
                    state: Err(err),
                    status: String::from("error occured"),
                },
                |(text, file_path)| {
                    let content = LineView::parse(&text);
                    Self {
                        status: format!("\"{}\" loaded", file_path.display()),
                        state: Ok(State {
                            history: History::from_iter(Some(content.clone())),
                            content,
                            file_path,
                            edit: false,
                        }),
                    }
                },
            )
    }

    fn update(&mut self, message: Self::Message) {
        if let Ok(state) = &mut self.state {
            match message {
                Message::Choose(message) => {
                    self.status = state.content.action.spawn(message);
                }
                Message::Edit(new_state) => {
                    state.edit = new_state;
                }
                Message::Save => self.status = Self::save(state),
                Message::Cancel => state.content = state.history.soft_reset(),
                Message::Undo => state.content = state.history.undo(),
                Message::Redo => state.content = state.history.redo(),
                Message::EditMessage(list_type, msg) => {
                    let list = match list_type {
                        ListType::Suffix => &mut state.content.action.suffix,
                        ListType::Prefix => &mut state.content.action.prefix,
                        ListType::Lines => &mut state.content.lines,
                    };
                    let message::ListEdit(index, msg) = msg;
                    if index < list.len() || matches!(msg, message::LineEdit::Add) {
                        match msg {
                            message::LineEdit::Remove => {
                                list.remove(index);
                                state.update_history();
                            }
                            message::LineEdit::Update(new) => list[index] = new,
                            message::LineEdit::Up => {
                                list.swap(index, index.saturating_sub(1));
                                state.update_history();
                            }
                            message::LineEdit::Down => {
                                if (index + 1) < list.len() {
                                    list.swap(index, index + 1);
                                    state.update_history();
                                }
                            }
                            message::LineEdit::Add => {
                                list.insert(index, String::new());
                                state.update_history();
                            }
                        }
                    }
                }
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let edit_active = self
            .state
            .as_ref()
            .ok()
            .map_or(false, |content| content.edit);

        let min_button = |content| {
            button(content)
                .width(Length::Shrink)
                .height(Length::Shrink)
                .padding(2)
        };

        let line_edit_button = |list_type, index, line_edit_msg, content| {
            min_button(content).on_press(Message::EditMessage(
                list_type,
                message::ListEdit(index, line_edit_msg),
            ))
        };

        let add_button = |list_type, index| {
            line_edit_button(list_type, index, message::LineEdit::Add, "add")
                .style(theme::Button::Positive)
        };

        let line_entry = |list_type, index, elem: &str| {
            Row::new()
                .spacing(3)
                .padding(0)
                .height(Length::Shrink)
                .width(Length::Shrink)
                .align_items(Alignment::Center)
                .push(
                    line_edit_button(list_type, index, message::LineEdit::Remove, "del")
                        .style(theme::Button::Destructive),
                )
                .push(add_button(list_type, index + 1))
                .push(line_edit_button(
                    list_type,
                    index,
                    message::LineEdit::Up,
                    "up",
                ))
                .push(line_edit_button(
                    list_type,
                    index,
                    message::LineEdit::Down,
                    "down",
                ))
                .push(
                    text_input("...", elem)
                        .padding(2)
                        .on_input(move |s| {
                            Message::EditMessage(
                                list_type,
                                message::ListEdit(index, message::LineEdit::Update(s)),
                            )
                        })
                        .width(Length::Fill),
                )
        };

        Column::new()
            .push_if(edit_active, || container(text("lines")).padding(3))
            .push_if(edit_active, || horizontal_rule(1))
            .push(match &self.state {
                Ok(State {
                    content: LineView { lines, .. },
                    ..
                }) => lines
                    .iter()
                    .enumerate()
                    .fold(Column::new(), |col, (i, line)| {
                        col.push_if(!edit_active, || {
                            button(text(line))
                                .style(theme::Button::Text)
                                .on_press(Message::Choose(line.clone()))
                                .padding(0)
                        })
                        .push_if(edit_active, || line_entry(ListType::Lines, i, line))
                    })
                    .spacing(3)
                    .padding(3)
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
            .push_if(edit_active, || horizontal_rule(1))
            .push_if(edit_active, || container(text("command")).padding(3))
            .push_if(edit_active, || horizontal_rule(1))
            .push_maybe(self.state.as_ref().ok().and_then(|state| {
                state.edit.then(|| {
                    let col = Column::new()
                        .width(Length::Fill)
                        .height(Length::Shrink)
                        .spacing(3)
                        .padding(3);

                    let col = state
                        .content
                        .action
                        .prefix
                        .iter()
                        .enumerate()
                        .fold(col, |col, (i, pre)| {
                            col.push(line_entry(ListType::Prefix, i, pre))
                        });

                    let col = col.push(
                        container(text("line"))
                            .padding(2)
                            .style(theme::Container::Box)
                            .width(Length::Shrink)
                            .height(Length::Shrink),
                    );

                    let col = state
                        .content
                        .action
                        .suffix
                        .iter()
                        .enumerate()
                        .fold(col, |col, (i, suf)| {
                            col.push(line_entry(ListType::Suffix, i, suf))
                        });

                    col
                })
            }))
            .push(horizontal_rule(1))
            .push(
                Row::new()
                    .width(Length::Fill)
                    .height(Length::Shrink)
                    .padding(Padding::new(3.0))
                    .spacing(3.0)
                    .align_items(Alignment::Center)
                    .push(
                        container(
                            checkbox("edit", edit_active, Message::Edit)
                                .spacing(3)
                                .width(Length::Shrink)
                                .style(if edit_active {
                                    theme::Checkbox::Primary
                                } else {
                                    theme::Checkbox::Secondary
                                }),
                        )
                        .width(Length::Shrink)
                        .height(Length::Shrink)
                        .style(theme::Container::Box)
                        .padding(Padding::new(2.0)),
                    )
                    .push_if(edit_active, || {
                        button("save")
                            .width(Length::Shrink)
                            .height(Length::Shrink)
                            .padding(Padding::new(2.0))
                            .style(theme::Button::Positive)
                            .on_press_maybe(self.state.as_ref().ok().and_then(|state| {
                                (state.history.has_future() || state.history.has_past())
                                    .then_some(Message::Save)
                            }))
                    })
                    .push_if(edit_active, || {
                        button("cancel")
                            .width(Length::Shrink)
                            .height(Length::Shrink)
                            .padding(Padding::new(2.0))
                            .style(theme::Button::Destructive)
                            .on_press_maybe(self.state.as_ref().ok().and_then(|state| {
                                state.history.has_past().then_some(Message::Cancel)
                            }))
                    })
                    .push_if(edit_active, || {
                        button("undo")
                            .width(Length::Shrink)
                            .height(Length::Shrink)
                            .padding(Padding::new(2.0))
                            .style(theme::Button::Primary)
                            .on_press_maybe(self.state.as_ref().ok().and_then(|content| {
                                content.history.has_past().then_some(Message::Undo)
                            }))
                    })
                    .push_if(edit_active, || {
                        button("redo")
                            .width(Length::Shrink)
                            .height(Length::Shrink)
                            .padding(Padding::new(2.0))
                            .style(theme::Button::Primary)
                            .on_press_maybe(self.state.as_ref().ok().and_then(|content| {
                                content.history.has_future().then_some(Message::Redo)
                            }))
                    })
                    .push(
                        text(&self.status)
                            .pipe(container)
                            .width(Length::Fill)
                            .height(Length::Shrink)
                            .align_x(Horizontal::Right)
                            .padding(2),
                    ),
            )
            .into()
    }
}

fn main() -> iced::Result {
    App::run_default()
}
