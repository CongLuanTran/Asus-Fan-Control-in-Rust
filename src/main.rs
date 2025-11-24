use clap::Parser;
use fanctl::cli::{Cli, Command};
use fanctl::client::status;
use fanctl::daemon::daemon;

use tokio::{signal, sync::mpsc::channel};

#[tokio::main]
async fn main() {
    let socket_path = String::from("/run/fanctl/fanctl.socket");
    let (shutdown_sender, shutdown_receiver) = channel(1);

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
