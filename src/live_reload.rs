use crate::SaaruInstance;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::time;
use std::{path::PathBuf, sync::mpsc};

pub fn live_reload(instance: &mut SaaruInstance) {
    // Live Reload
    // Run a watcher thread and a new renderer thread
    // They'll talk via a channel, sending over the name of the file that got changed
    // No live reload for anything that's a template (i.e. `.jinja`), you'll need a full reload for that
    // Watcher thread finds out which file

    // TODO Refactor : implement full re-render for individual file
    // re-render all the files for tags and collections
    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();

    watcher
        .watch(
            instance.arguments.source_dir.as_ref(),
            RecursiveMode::Recursive,
        )
        .unwrap();

    for res in rx {
        match res {
            Ok(event) => {
                log::info!("[LIVERELOAD] Received Event {:?}", event);
                match event.kind {
                    // Watch for files getting modified
                    notify::EventKind::Modify(metadata) => {
                        log::info!(
                            "[LIVERELOAD] File Modified: {:?} -> {:?}",
                            event.paths,
                            metadata
                        );

                        // TODO Trigger a re-render for this specific file
                        log::info!("[LIVERELOAD] Re-processing file {:?}", &event.paths[0]);
                        instance.preprocess_file_data(&event.paths[0]);

                        let start = time::Instant::now();

                        let current_frontmatter = instance
                            .frontmatter_map
                            .get(&event.paths[0].display().to_string())
                            .unwrap()
                            .clone();
                        log::info!(
                            "[LIVERELOAD] Triggering HTML Conversion for file {:?}",
                            &event.paths[0]
                        );
                        let html_content =
                            instance.render_file_from_frontmatter(current_frontmatter.clone());
                        log::info!(
                            "[LIVERELOAD] Writing to Destination for file {:?}",
                            &event.paths[0]
                        );
                        instance.write_html_to_file(
                            PathBuf::from(&current_frontmatter.write_path),
                            html_content,
                        );
                        let end = time::Instant::now();
                        log::info!("File {:?} re-rendered in {:?}", event.paths[0], end - start);
                    }

                    // Watch for files getting created
                    notify::EventKind::Create(metadata) => {
                        log::info!(
                            "[LIVERELOAD] File Created: {:?} -> {:?}",
                            event.paths,
                            metadata
                        );

                        // TODO Trigger a re-render for this specific file
                        log::info!("[LIVERELOAD] Processing file {:?}", &event.paths[0]);
                        instance.preprocess_file_data(&event.paths[0]);

                        let start = time::Instant::now();

                        let current_frontmatter = instance
                            .frontmatter_map
                            .get(&event.paths[0].display().to_string())
                            .unwrap()
                            .clone();
                        log::info!(
                            "[LIVERELOAD] Triggering HTML Conversion for file {:?}",
                            &event.paths[0]
                        );
                        let html_content =
                            instance.render_file_from_frontmatter(current_frontmatter.clone());
                        log::info!(
                            "[LIVERELOAD] Writing to Destination for file {:?}",
                            &event.paths[0]
                        );
                        instance.write_html_to_file(
                            PathBuf::from(&current_frontmatter.write_path),
                            html_content,
                        );
                        let end = time::Instant::now();
                        log::info!("File {:?} re-rendered in {:?}", event.paths[0], end - start);
                    }

                    // watch for files getting deleted
                    notify::EventKind::Remove(metadata) => {
                        log::info!(
                            "[LIVERELOAD] File Removed: {:?} -> {:?}",
                            event.paths,
                            metadata
                        );
                        log::error!("Live Remove not implemented yet!")
                    }
                    _ => {}
                }
            }
            Err(e) => log::error!("[LIVERELOAD] {:?}", e),
        }
    }
}
