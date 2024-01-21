slint::include_modules!();

use std::{path::PathBuf, rc::Rc};

use anyhow::Result;
use clap::Parser;
use line_viewer::LineView;
use slint::{ModelRc, SharedString, VecModel};
use tap::Pipe;

#[derive(Parser)]
struct Cli {
    file_path: PathBuf,
}

fn vec_to_model_rc<T>(v: Vec<T>) -> ModelRc<T> where T: Clone + 'static {
    VecModel::from(v).pipe(Rc::new).pipe(ModelRc::from)
}

fn main() -> Result<()> {
    let cli = Cli::try_parse()?;
    let view = LineView::read(&cli.file_path)?.pipe(Rc::new);
    let lines = view
        .lines()
        .map(SharedString::from)
        .collect::<Vec<_>>()
        .pipe(vec_to_model_rc);

    let ui = AppWindow::new()?;

    ui.set_lines(lines);
    ui.set_view_title(SharedString::from(view.title()));

    ui.on_line_clicked({
        let view_handle = Rc::downgrade(&view);
        move |index: i32| {
            let index: usize = index.try_into().unwrap();
            let view = view_handle.upgrade().unwrap();
            let _ = view.get(index).unwrap().execute();
        }
    });

    ui.run()?;

    Ok(())
}
