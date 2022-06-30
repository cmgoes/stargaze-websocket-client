#![allow(unused_variables)]
#![allow(unused_imports)]
use futures::StreamExt;
use tendermint::abci::Transaction;
use tendermint_rpc::event::EventData;
use tendermint_rpc::query::EventType;
use tendermint_rpc::{Client, SubscriptionClient, WebSocketClient};

use ansi_term::Colour::{Blue, Cyan, Green, Purple, Red, Yellow};

use env_logger::Env;
use log::{debug, info};

use anyhow::anyhow;
use anyhow::Error;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("{}", Yellow.bold().paint("Starting"));
    let (client, driver) = WebSocketClient::new("ws://127.0.0.1:26657/websocket")
        .await
        .unwrap();
    let driver_handle = tokio::spawn(async move { driver.run().await });

    let mut subs = client.subscribe(EventType::Tx.into()).await.unwrap();

    while let Some(res) = subs.next().await {
        let ev = res.unwrap();

        // info!("");
        // info!("{} {}", Green.bold().paint("New data ->"), ev.query);

        match ev.data {
            EventData::NewBlock {
                block,
                result_begin_block,
                result_end_block,
            } => {
                let block = block.unwrap();
                debug!(
                    "chain: {:?} proposer: {:?}",
                    &block.header.chain_id, &block.header.proposer_address
                );
            }
            EventData::Tx { tx_result } => {
                tx_result
                    .result
                    .events
                    .iter()
                    .for_each(|event| match event.type_str.as_str() {
                        "update_client" => {
                            debug!("({})", Cyan.bold().paint(event.type_str.clone()));
                        }
                        "wasm" => {
                            info!(
                                "-> {}: {:?}",
                                Red.bold().paint(event.type_str.clone()),
                                event.attributes
                            );
                        }
                        "wasm-fair_burn"
                        | "wasm-royalty-payout"
                        | "wasm-finalize-sale"
                        | "wasm-set-bid"
                        | "wasm-set-ask"
                        | "wasm-update-ask"
                        | "wasm-remove-ask" => {
                            info!(
                                "-> {}: {:?}",
                                Purple.bold().paint(event.type_str.clone()),
                                event.attributes
                            );
                        }
                        _ => {
                            if event.type_str.contains("wasm") {
                                info!(
                                    "-> {}: {:?}",
                                    Blue.bold().paint(event.type_str.clone()),
                                    event.attributes
                                );
                            } else {
                                debug!(
                                    "-> {}: {:?}",
                                    Yellow.italic().dimmed().paint(event.type_str.clone()),
                                    event.attributes
                                );
                            }
                        }
                    })
            }
            EventData::GenericJsonEvent(_) => {
                return Err(anyhow!("Weird, received GenericJsonEvent"))
            }
        }
    }

    let mut tx_subs = client.subscribe(EventType::Tx.into()).await.unwrap();
    while let Some(res) = tx_subs.next().await {
        let ev = res.unwrap();
        info!("{} {}", Yellow.bold().paint("Tx ->"), ev.query);
    }

    // Signal to the driver to terminate.
    client.close().unwrap();
    // Await the driver's termination to ensure proper connection closure.
    let _ = driver_handle.await.unwrap();

    Ok(())
}
