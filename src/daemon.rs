use std::process::exit;

use crate::cli::Command;
use crate::controller::{FanController, FanControllerConfig, FanState};
use crate::utils::{find_cpu, find_pwn1, write_pwn1};
use sysinfo::Components;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::oneshot;

pub async fn daemon(socket_path: String, mut shutdown_receiver: Receiver<()>) {
    /*---------setup Unix socket---------*/
    if fs::remove_file(&socket_path).await.is_err() {};
    let listener = UnixListener::bind(&socket_path).expect("Could not create unix socket");

    /*---------handle shutdown gracefully---------*/
    tokio::spawn(async move {
        match shutdown_receiver.recv().await {
            Some(()) => {
                fs::remove_file(&socket_path)
                    .await
                    .expect("Failed to remove socket file");

                exit(1);
            }
            None => {
                eprintln!(
                    "received nothing from the shutdown receiver. This should not be possible"
                )
            }
        }
    });

    let (tx, rx) = channel(1);

    tokio::spawn(async move {
        worker(rx).await;
    });

    while let Ok((stream, _)) = listener.accept().await {
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            handle_client(stream, tx_clone).await;
        });
    }
}

async fn worker(mut receiver: Receiver<(Command, oneshot::Sender<String>)>) {
    /*---------setup system interaction---------*/
    let mut components = Components::new_with_refreshed_list();
    let cpu = find_cpu(&mut components);
    let pwn1_enable = find_pwn1();

    /*---------initialize controller---------*/
    let config = FanControllerConfig::load_user_config();
    let mut controller = FanController::new(config);
    let temp = cpu.temperature().unwrap();
    controller.update(temp);

    /*---------main loop---------*/
    loop {
        tokio::select! {
            Some((cmd, resp_tx)) = receiver.recv() => {
                println!("Received: {:?}", cmd);
                match cmd {
                    Command::Status => {
                        if let Err(e) = resp_tx.send(controller.status()){
                            eprintln!("Error responding to handler; error : {e}");
                        };
                    }
                    Command::Daemon => ()
                }
            }

            _ = tokio::time::sleep_until(controller.next_read.into()) => {
                cpu.refresh();
                let temp = cpu.temperature().unwrap();
                controller.update(temp);

                // We turn the fan on full speed (0) if the temperature reach the threshold, else return it to auto ()
                let value = match controller.fan_state {
                    FanState::Enabled => 0,
                    FanState::Auto => 2,
                };

                write_pwn1(&pwn1_enable, value);
            }
        }
    }
}

async fn handle_client(mut stream: UnixStream, tx: Sender<(Command, oneshot::Sender<String>)>) {
    let mut buf: [u8; 1024] = [0u8; 1024];
    match stream.read(&mut buf).await {
        Ok(0) => (),
        Ok(n) => {
            match serde_json::from_slice(&buf[..n]) {
                Ok(cmd) => {
                    let (resp_tx, resp_rx) = oneshot::channel();
                    tx.send((cmd, resp_tx)).await.unwrap();
                    match resp_rx.await {
                        Ok(result) => {
                            if let Err(e) = stream.write_all(result.as_bytes()).await {
                                eprintln!("Error writing to client; error: {e}");
                            }
                        }
                        Err(e) => {
                            eprintln!("Error receiving from background; error: {e}");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error while reading command; error: {e}")
                }
            };
        }
        Err(e) => {
            eprintln!("Error reading from the client; error: {e}");
        }
    }
}
