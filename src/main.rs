slint::include_modules!();

use std::{path::PathBuf, rc::Rc};

use anyhow::Result;
use clap::Parser;
use line_viewer::LineView;
use slint::{SharedString, VecModel};

#[derive(Parser)]
struct Cli {
    #[arg(required = true)]
    file_path: Vec<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::try_parse()?;

    let views = cli
        .file_path
        .into_iter()
        .map(|path| LineView::read(&path))
        .collect::<Rc<[_]>>();

    let ui = AppWindow::new()?;

    ui.set_lines(
        Rc::new(VecModel::from(
            views[0]
                .as_ref()
                .unwrap()
                .lines()
                .map(SharedString::from)
                .collect::<Vec<_>>(),
        ))
        .into(),
    );

    ui.on_line_clicked({
        let views_handle = Rc::downgrade(&views);
        move |index: i32| {
            let views = views_handle.upgrade().unwrap();
            if let Ok(view) = &views[0] {
                let _ = view.get(index.try_into().unwrap()).unwrap().execute();
            }
        }
    });

    ui.run()?;

    Ok(())
}
