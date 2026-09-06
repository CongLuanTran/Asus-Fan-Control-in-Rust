use ::tracing_subscriber::prelude::*;
use clap::Parser;
use fanctl::cli::{Cli, Command};
use fanctl::client::status;
use fanctl::daemon::daemon;

use tokio::{signal, sync::mpsc::channel};

#[tokio::main]
async fn main() {
    let socket_path = String::from("/run/fanctl.socket");
    let (shutdown_sender, shutdown_receiver) = channel(1);

    match tracing_journald::layer() {
        Ok(layer) => {
            tracing_subscriber::registry().with(layer).init();
        }
        // journald is typically available on Linux systems, but nowhere else. Portable software
        // should handle its absence gracefully.
        Err(e) => {
            eprintln!("couldn't connect to journald: {}", e);
            std::process::exit(1);
        }
    }

    tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(()) => {
                shutdown_sender.send(()).await.unwrap();
            }
            Err(e) => {
                eprintln!("{}", e)
            }
        }
    });

    let args = Cli::parse();

    match args.cmd {
        Command::Daemon => daemon(socket_path, shutdown_receiver).await,
        Command::Status => status(socket_path).await,
    }
}
