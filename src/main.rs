use crate::arguments::SaaruArguments;
use crate::saaru::SaaruInstance;
use clap::Parser;
use log::LevelFilter;
use std::path::PathBuf;
use std::time;

mod arguments;
mod frontmatter;
mod saaru;
mod utils;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// This is the base path
    #[arg(short, long)]
    base_path: PathBuf,

    #[arg(long)]
    /// Having this on spawns both the web server and the live reloader
    live_reload: bool,

    #[arg(long)]
    /// Having this on only turns on the live re-render, assuming you have your own web server
    live_rerender: bool,

    #[arg(short, long)]
    serve: bool,
}

fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .env()
        .init()
        .unwrap();
    log::debug!("Initialized Logger");

    let commandline_arguments = Arguments::parse();
    log::debug!("Command Line Arguments -> {:?}", &commandline_arguments);

    // Check for options
    if commandline_arguments.live_reload && commandline_arguments.live_rerender {
        log::error!("Cannot have both Live Reload (--live-reload) AND Live Re-Render (--live-rerender) on at once!");
        panic!();
    }

    // Expect to see a `.saaru.json` file here in the base path
    let args = SaaruArguments::new(
        commandline_arguments.base_path,
        commandline_arguments.live_reload,
        commandline_arguments.live_rerender,
    );
    let mut instance = SaaruInstance::new(args);

    let start = time::Instant::now();
    instance.set_template_environment();
    instance.render_pipeline();
    let end = time::Instant::now();
    log::info!("Total Time Taken -> {:?}", end - start);

    //  Implementing Browser-Side Live-Reload
    //
    //  Turns out doing this while still making each thing (live re-render on file change +
    //  web serving) while still keeping functionality to have both work together is going to be
    //  non-trivial. Here's the current plan - Connect all these independent components
    //  (FSwatcher, server, re-render listener) run on independent threads or a tokio runtime
    //  with an individual listener of sorts that executes the right action based on the event
    //  that's currently sent over the shared channel.
    //  I'm writing this because I think i'll forget, so anyway, here's an ASCII Diagram of what I
    //  look at implementing soon -
    //
    //                          Crossbeam MPMC Channel
    //                                   |
    //                                   |
    //                                   |
    //     +-----------------+           |   [4-R]    +----------------+
    //     |                 |   [1]     |----------> |                |
    //     |  FS Watcher     |---------->|            | Web Server     |
    //     |                 |           |            |                |
    //     +-----------------+           |            +----------------+
    //                                   |
    //                                   |[1-R]
    //      +-----------------+          |-----\      +----------------+
    //      |                 |   [2]    |      ----->|                |
    //      | Saaru re-render |<----------------------|  Saaru         |
    //      | watcher         |---------------------> |  Orchestrator  |
    //      |                 |   [3]    |            |                |
    //      +-----------------+          |            +----------------+
    //                                   |               |
    //                                   |   [4]         |
    //                                   |<--------------+
    //
    //  The events, as you see them, are
    //
    //  [1]   ->  The filesystem watcher (in this case, `notify`) recieves a change event. The handler
    //            for the change event wraps it in a `SaaruEvent::FileChanged` and sends it off into the channel
    //  [1-R] ->  The Saaru Orchestrator receives the SaaruEvent sent by the FS Watcher
    //  [2]   ->  The Saaru Re-Render function calls into the current `SaaruInstance` to trigger the
    //            re-render of the individual file that's been changed. This file is then
    //            read into memory, converted to markdown, and written to a new .html file.
    //  [3]   ->  Once the Re-render completes, Re-render watcher fires a `SaaruEvent::FileRewritten`
    //            event. The Saaru Orchestrator consumes this event. Maybe we can cache the file
    //            content? Some other improvement? HACK
    //  [4]   ->  The Orchestrator fires a `SaaruEvent::reload` into the channel. This is to indicate
    //            that the web server should reload.
    //  [4-R] ->  The Web server recieves the `SaaruEvent::reload` and reloads on the browser side.

    if commandline_arguments.live_reload || commandline_arguments.live_rerender {
        instance.orchestrator();
    }
}
