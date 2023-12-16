use iced::{
    theme,
    widget::{button, text_input, Button, Row},
    Alignment, Element, Length,
};

use crate::message::{self, Message};

pub fn min_button<'a, M>(content: impl Into<Element<'a, M>>) -> Button<'a, M> {
    button(content)
        .width(Length::Shrink)
        .height(Length::Shrink)
        .padding(2)
}

pub fn line_edit_button<'a>(
    list_type: message::ListType,
    index: usize,
    line_edit_msg: message::LineEdit,
    content: impl Into<Element<'a, message::Message>>,
) -> Button<'a, message::Message> {
    min_button(content).on_press(Message::EditMessage(
        list_type,
        message::ListEdit(index, line_edit_msg),
    ))
}

pub fn add_button<'a>(list_type: message::ListType, index: usize) -> Button<'a, message::Message> {
    line_edit_button(list_type, index, message::LineEdit::Add, "add").style(theme::Button::Positive)
}

pub fn line_entry<'a>(
    list_type: message::ListType,
    index: usize,
    elem: &str,
) -> Row<'a, message::Message> {
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
}
