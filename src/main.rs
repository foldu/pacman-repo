#![feature(async_await)]
mod handlers;

use std::sync::Arc;

use serde::Deserialize;
use snafu::{ResultExt, Snafu};
use warp::http::StatusCode;
use warp::Filter;

#[derive(Snafu, Debug)]
pub enum Error {
    #[snafu(display("Can't create repo dir {}: {}", REPO_DIR, source))]
    CreateRepo { source: std::io::Error },
}

const REPO_DIR: &str = "/repo";

fn run() -> Result<(), Error> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let config = Arc::new(Config {
        password: "hunter2".into(),
    });

    let config = warp::any().map(move || config.clone());

    let upload = warp::post2()
        .and(warp::path::end())
        .and(config.clone())
        .and(warp::multipart::form().max_length(5 * (1 << 20)))
        .and_then(handlers::upload);

    std::fs::create_dir_all(REPO_DIR).context(CreateRepo)?;

    let repo = warp::get2()
        .and(warp::path("repo"))
        .and(warp::path::end())
        .and(warp::fs::dir(REPO_DIR));

    let routes = upload.or(repo).recover(handlers::handle_error);
    warp::serve(routes.with(warp::log("pacman-repo"))).run(([0; 4], 8080));

    Ok(())
}

#[derive(Deserialize, Debug)]
pub struct Config {
    password: String,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
