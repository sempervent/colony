use notify::{Watcher, RecursiveMode, Event, EventKind};
use std::path::Path;
use anyhow::Result;

pub struct ModWatcher {
    watcher: notify::RecommendedWatcher,
    mod_path: std::path::PathBuf,
}

impl ModWatcher {
    pub fn new<F>(mod_path: &Path, callback: F) -> Result<Self>
    where
        F: Fn(&Path) + Send + Sync + 'static,
    {
        let mod_path = mod_path.to_path_buf();
        let watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            match res {
                Ok(event) => {
                    if matches!(event.kind, EventKind::Modify(_)) {
                        for path in event.paths {
                            callback(&path);
                        }
                    }
                }
                Err(e) => println!("Watch error: {:?}", e),
            }
        })?;
        
        watcher.watch(mod_path.as_path(), RecursiveMode::Recursive)?;
        
        Ok(Self {
            watcher,
            mod_path,
        })
    }
}
