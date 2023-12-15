#[derive(Debug, Clone)]
pub enum Message {
    Choose(String),
    Edit(bool),
    Title(String),
    Save,
    Cancel,
    Undo,
    Redo,
    EditMessage(ListType, ListEdit),
    ToTab(usize),
}

#[derive(Debug, Clone)]
pub enum LineEdit {
    Remove,
    Update(String),
    Up,
    Down,
    Add,
}

#[derive(Debug, Clone)]
pub struct ListEdit(pub usize, pub LineEdit);

#[derive(Debug, Clone, Copy)]
pub enum ListType {
    Suffix,
    Prefix,
    Lines,
}
