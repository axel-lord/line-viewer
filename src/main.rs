#![allow(missing_debug_implementations)]

slint::include_modules!();

use std::{
    path::PathBuf,
    rc::Rc,
    sync::{atomic::AtomicUsize, Arc, Mutex, RwLock, Weak},
};

use anyhow::Result;
use clap::Parser;
use line_viewer::LineView;
use notify::{Event, EventKind, Watcher};
use slint::{ModelRc, SharedString, VecModel};
use tap::Pipe;

#[derive(Parser)]
struct Cli {
    file_path: PathBuf,
}

fn vec_to_model_rc<T>(v: Vec<T>) -> ModelRc<T>
where
    T: Clone + 'static,
{
    VecModel::from(v).pipe(Rc::new).pipe(ModelRc::from)
}

fn lines(view: &LineView) -> ModelRc<SharedString> {
    view.lines()
        .map(SharedString::from)
        .collect::<Vec<_>>()
        .pipe(vec_to_model_rc)
}

fn create_watcher(
    lock_handle: Weak<Mutex<bool>>,
    view: &LineView,
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

    for source in view.all_sources() {
        drop(watcher.watch(source, notify::RecursiveMode::NonRecursive));
    }

    Ok(watcher)
}

fn main() -> Result<()> {
    let cli = Cli::try_parse()?;
    let view = LineView::read(&cli.file_path)?;

    let ui = AppWindow::new()?;
    let lock = Arc::new(Mutex::new(false));
    let watcher = Arc::new(Mutex::new(Some(create_watcher(
        Arc::downgrade(&lock),
        &view,
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
                    let mut view = view.write().unwrap();
                    if let Err(err) = view.reload() {
                        println!("{err}");
                        return;
                    };
                }

                let view = view.read().unwrap();
                ui.set_lines(lines(&view));
                ui.set_view_title(SharedString::from(view.title()));

                let watcher = watcher_handle.upgrade().unwrap();
                let mut watcher = watcher.lock().unwrap();

                if let Ok(new_watcher) = create_watcher(lock_handle.clone(), &view) {
                    println!("updated watcher");
                    *watcher = Some(new_watcher);
                }
            }
        },
    );

    ui.run()?;

    Ok(())
}
