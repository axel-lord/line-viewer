#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod extensions;
pub mod fwidget;
pub mod history;
pub mod line_view;
pub mod message;
pub mod state;

use anyhow::Context;
use clap::Parser;
use extensions::{ColRowExt, TapIf};
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
    #[arg(required = true)]
    file_path: Vec<PathBuf>,
}

struct App {
    state: Vec<anyhow::Result<State>>,
    status: Vec<String>,
    current: usize,
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
            .map_or_else(
                |err| Self {
                    state: vec![Err(err)],
                    status: vec![String::from("error occured")],
                    current: 0,
                },
                |cli| {
                    let (state, status): (Vec<_>, _) = cli.file_path.into_iter().map(|file_path| {
                        File::open(&file_path)
                            .context("failed to open file, does it exist and do you have the proper permissions?")
                            .and_then(|file| io::read_to_string(file).context("failed to read file to memory"))
                            .map(|content| LineView::parse(&content)).map_or_else(|err| {
                                let status = format!("error reading \"{}\", {err}", file_path.display());
                                (Err(err), status)
                            }, |content| {
                                let status = format!("\"{}\" loaded", file_path.display());
                                (Ok(State { history: History::from_iter(Some(content.clone())), content, file_path: file_path.clone(), edit: false }), status)
                            })
                    }).unzip();
                    Self {
                        current: state.len().saturating_sub(1),
                        state,
                        status,
                    }
                },
            )
    }

    fn update(&mut self, message: Self::Message) {
        let (Some(Ok(state)), Some(status)) = (
            self.state.get_mut(self.current),
            self.status.get_mut(self.current),
        ) else {
            return;
        };

        match message {
            Message::ToTab(n) => self.current = n,
            Message::Choose(message) => {
                *status = state.content.action.spawn(message);
            }
            Message::Edit(new_state) => {
                state.edit = new_state;
            }
            Message::Save => *status = Self::save(state),
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
            Message::Title(title) => state.content.title = (!title.is_empty()).then_some(title),
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let Some(status) = self.status.get(self.current) else {
            panic!("status not correctly initialized, this is probably a programming error");
        };
        let Some(state) = self.state.get(self.current) else {
            panic!("state not correctly initialized, this is probably a programming error");
        };
        let edit_active = state.as_ref().map_or(false, |content| content.edit);
        let show_tabs = self.status.len() > 1
            || state
                .as_ref()
                .map_or(false, |state| state.content.title.is_some());

        Column::new()
            .push_if(show_tabs, || {
                self.state
                    .iter()
                    .enumerate()
                    .fold(Row::new(), |row, (i, state)| {
                        let is_current = self.current == i;
                        let text_content = state.as_ref().map_or_else(
                            |_err| String::from("err"),
                            |state| {
                                state.content.title.as_ref().map_or_else(
                                    || state.file_path.display().to_string(),
                                    String::clone,
                                )
                            },
                        );
                        row.push_if_else(
                            is_current,
                            || {
                                text(&text_content).pipe(container).padding(3).style(
                                    |t: &iced::Theme| -> container::Appearance {
                                        let palette = t.extended_palette();
                                        container::Appearance {
                                            background: Some(palette.primary.weak.color.into()),
                                            text_color: Some(palette.primary.weak.text),
                                            ..<iced::Theme as container::StyleSheet>::appearance(
                                                t,
                                                &theme::Container::Box,
                                            )
                                        }
                                    },
                                )
                            },
                            || {
                                button(text(&text_content))
                                    .padding(2)
                                    .style(theme::Button::Secondary)
                                    .on_press(Message::ToTab(i))
                            },
                        )
                    })
                    .align_items(Alignment::End)
                    .padding(3)
                    .spacing(3)
            })
            .push_if(show_tabs, || horizontal_rule(1))
            .push_if(edit_active, || text("title").pipe(container).padding(3))
            .push_if(show_tabs, || horizontal_rule(1))
            .push_if(edit_active, || {
                text_input(
                    "...",
                    &state
                        .as_ref()
                        .ok()
                        .and_then(|s| s.content.title.clone())
                        .unwrap_or_default(),
                )
                .tap_if(state.is_ok(), |ti| ti.on_input(Message::Title))
                .padding(2)
                .pipe(container)
                .width(Length::Fill)
                .padding(3)
            })
            .push_if(edit_active, || horizontal_rule(1))
            .push_if(edit_active, || {
                Row::new().push(text("lines")).padding(3).spacing(3)
            })
            .push_if(edit_active, || horizontal_rule(1))
            .push(match state {
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
                        .push_if(edit_active, || {
                            fwidget::line_entry(ListType::Lines, i, line)
                        })
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
            .push_if(edit_active, || {
                Row::new().padding(3).spacing(3).push(text("command"))
            })
            .push_if(edit_active, || horizontal_rule(1))
            .push_maybe(state.as_ref().ok().and_then(|state| {
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
                            col.push(fwidget::line_entry(ListType::Prefix, i, pre))
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
                            col.push(fwidget::line_entry(ListType::Suffix, i, suf))
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
                        Row::new()
                            .padding(0)
                            .spacing(3)
                            .align_items(Alignment::Center)
                            .height(Length::Shrink)
                            .width(Length::Shrink)
                            .push({
                                fwidget::min_button("save")
                                    .style(theme::Button::Positive)
                                    .on_press_maybe(state.as_ref().ok().and_then(|state| {
                                        (state.history.has_future() || state.history.has_past())
                                            .then_some(Message::Save)
                                    }))
                            })
                            .push({
                                fwidget::min_button("cancel")
                                    .style(theme::Button::Destructive)
                                    .on_press_maybe(state.as_ref().ok().and_then(|state| {
                                        state.history.has_past().then_some(Message::Cancel)
                                    }))
                            })
                            .push({
                                fwidget::min_button("undo")
                                    .style(theme::Button::Primary)
                                    .on_press_maybe(state.as_ref().ok().and_then(|content| {
                                        content.history.has_past().then_some(Message::Undo)
                                    }))
                            })
                            .push({
                                fwidget::min_button("redo")
                                    .style(theme::Button::Primary)
                                    .on_press_maybe(state.as_ref().ok().and_then(|content| {
                                        content.history.has_future().then_some(Message::Redo)
                                    }))
                            })
                            .push({
                                fwidget::line_edit_button(
                                    ListType::Lines,
                                    0,
                                    message::LineEdit::Add,
                                    "add line",
                                )
                                .style(theme::Button::Positive)
                            })
                            .push({
                                fwidget::line_edit_button(
                                    ListType::Prefix,
                                    0,
                                    message::LineEdit::Add,
                                    "add prefix",
                                )
                                .style(theme::Button::Positive)
                            })
                            .push({
                                fwidget::line_edit_button(
                                    ListType::Suffix,
                                    0,
                                    message::LineEdit::Add,
                                    "add suffix",
                                )
                                .style(theme::Button::Positive)
                            })
                    })
                    .push(
                        text(status)
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
