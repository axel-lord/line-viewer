use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[clap(author, version)]
struct Cli {
    /// File to open and view
    file_path: PathBuf,
}

fn main() -> line_view::Result {
    let Cli { file_path } = Cli::parse();
    let view = line_view::LineView::read(&file_path)?;

    for line in &view {
        if line.is_title() {
            print!("-- ")
        }
        if line.is_warning() {
            print!("[warning] ")
        }
        println!("{}", line.text());
    }

    Ok(())
}
