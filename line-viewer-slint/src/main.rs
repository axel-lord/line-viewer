#![allow(missing_debug_implementations)]

slint::include_modules!();

use std::{
    cell::RefCell,
    collections::HashSet,
    path::{Path, PathBuf},
    rc::Rc,
    sync::{atomic::AtomicUsize, Arc, Mutex, RwLock, Weak},
};

use anyhow::{anyhow, Result};
use clap::{ArgGroup, Parser};
use line_view::{
    provide::{self, PathReadProvider},
    LineView,
};
use notify::{Event, EventKind, Watcher};
use slint::{ModelRc, SharedString, VecModel};
use tap::Pipe;

#[derive(Parser)]
#[clap(author, version)]
#[clap(group(
        ArgGroup::new("input")
        .args(&["mime_type", "application", "file_path"])
        ))]
struct Cli {
    #[clap(short, long)]
    /// Output xdg mimetype xml
    mime_type: bool,

    #[clap(short, long)]
    /// Output xdg .desktop file
    application: bool,

    /// File to open and view
    file_path: Option<PathBuf>,
}

fn vec_to_model_rc<T>(v: Vec<T>) -> ModelRc<T>
where
    T: Clone + 'static,
{
    VecModel::from(v).pipe(Rc::new).pipe(ModelRc::from)
}

fn lines(view: &LineView) -> ModelRc<Line> {
    view.iter()
        .map(|line| Line {
            text: SharedString::from(line.text()),
            has_command: line.has_command(),
            is_title: line.is_title(),
            is_warning: line.is_warning(),
        })
        .collect::<Vec<_>>()
        .pipe(vec_to_model_rc)
}

#[derive(Debug, Default)]
struct PathReadProviderWrap(PathReadProvider, Rc<RefCell<HashSet<PathBuf>>>);
impl provide::Read for PathReadProviderWrap {
    type BufRead = <provide::PathReadProvider as provide::Read>::BufRead;

    fn provide(&self, from: &str) -> line_view::Result<Self::BufRead> {
        let Self(provider, path_set) = self;
        let reader = provider.provide(from)?;
        path_set.borrow_mut().insert(PathBuf::from(from));
        Ok(reader)
    }
}

impl From<&Rc<RefCell<HashSet<PathBuf>>>> for PathReadProviderWrap {
    fn from(value: &Rc<RefCell<HashSet<PathBuf>>>) -> Self {
        Self(PathReadProvider, value.clone())
    }
}

fn create_watcher(
    lock_handle: Weak<Mutex<bool>>,
    imported: &HashSet<PathBuf>,
) -> Result<notify::RecommendedWatcher> {
    static WATCHER_COUNT: AtomicUsize = AtomicUsize::new(0);
    let id = WATCHER_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let mut watcher = notify::recommended_watcher({
        move |res: notify::Result<Event>| {
            let Ok(EventKind::Modify(_)) = res.map(|event: Event| event.kind) else {
                return;
            };

            let Some(lock) = lock_handle.upgrade() else {
                return;
            };

            if let Ok(mut lock) = lock.lock() {
                println!("found event as [{id}]");
                *lock = true;
            };
        }
    })?;

    for source in imported {
        drop(watcher.watch(source, notify::RecursiveMode::NonRecursive));
    }

    Ok(watcher)
}

fn run(file_path: &Path) -> Result<()> {
    let provider = PathReadProviderWrap::default();
    let path_set = provider.1.clone();
    let view = LineView::read(file_path.to_string_lossy(), provider)?;

    let ui = AppWindow::new()?;
    let lock = Arc::new(Mutex::new(false));
    let watcher = Arc::new(Mutex::new(Some(create_watcher(
        Arc::downgrade(&lock),
        &path_set.borrow(),
    )?)));

    let view = Arc::new(RwLock::new(view));

    ui.set_lines(lines(&view.read().unwrap()));
    ui.set_view_title(SharedString::from(view.read().unwrap().title()));

    ui.on_line_clicked({
        let view_handle = Arc::downgrade(&view);
        move |index: i32| {
            let index: usize = index.try_into().unwrap();
            let view = view_handle.upgrade().unwrap();
            if let Ok(view) = view.read() {
                drop(view.get(index).unwrap().execute());
            };
        }
    });

    let timer = slint::Timer::default();
    timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(300),
        {
            let ui_handle = ui.as_weak();
            let view_handle = Arc::downgrade(&view);
            let lock_handle = Arc::downgrade(&lock);
            let watcher_handle = Arc::downgrade(&watcher);
            let file_path = file_path.to_path_buf();
            let path_set = path_set.clone();
            move || {
                // Update lock
                {
                    let lock = lock_handle.upgrade().unwrap();
                    let mut lock = lock.lock().unwrap();
                    if *lock {
                        *watcher_handle.upgrade().unwrap().lock().unwrap() = None;
                    } else {
                        return;
                    }
                    *lock = false;
                }

                let ui = ui_handle.upgrade().unwrap();
                let view = view_handle.upgrade().unwrap();

                // Reload view
                {
                    match LineView::read(file_path.to_string_lossy(), PathReadProviderWrap::from(&path_set)) {
                        Ok(v) => *view.write().unwrap() = v,
                        Err(err) => {
                            println!("{err}");
                            return;
                        }
                    }
                }

                let view = view.read().unwrap();
                ui.set_lines(lines(&view));
                ui.set_view_title(SharedString::from(view.title()));

                let watcher = watcher_handle.upgrade().unwrap();
                let mut watcher = watcher.lock().unwrap();

                if let Ok(new_watcher) = create_watcher(lock_handle.clone(), &path_set.borrow()) {
                    println!("updated watcher");
                    *watcher = Some(new_watcher);
                }
            }
        },
    );

    ui.run().map_err(anyhow::Error::from)
}

fn file_dialog() -> Result<PathBuf> {
    use nfde::{DialogResult, FilterableDialogBuilder, Nfd, NfdPathBuf, SingleFileDialogBuilder};

    fn show_dialog() -> Result<DialogResult<NfdPathBuf>, nfde::Error> {
        Ok(Nfd::new()?
            .open_file()
            .add_filter("line-view File", "txtlv")?
            .add_filter("Text File", "txt")?
            .show())
    }

    fn convert_result(res: Result<DialogResult<NfdPathBuf>, nfde::Error>) -> Result<PathBuf> {
        let res = res.map_err(|err| anyhow!(err))?;
        match res {
            DialogResult::Ok(path) => Ok(path.to_path_buf()),
            DialogResult::Cancel => Err(anyhow!("no file chosen")),
            DialogResult::Err(err) => Err(anyhow!(err)),
        }
    }

    convert_result(show_dialog())
}
fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.mime_type {
        print!(include_str!("../assets/application-x-lineview.xml"));
        return Ok(());
    }

    if cli.application {
        print!(include_str!("../assets/line-view.desktop"));
        return Ok(());
    }

    let file_path = cli.file_path.map_or_else(file_dialog, Ok)?;

    run(&file_path)
}
