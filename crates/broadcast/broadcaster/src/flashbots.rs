use std::sync::Arc;

use alloy_primitives::Bytes;
use alloy_provider::{ext::MevApi, ProviderBuilder};
use alloy_rpc_types_mev::EthSendBundle;
use alloy_signer_local::PrivateKeySigner;
use eyre::{eyre, Result};
use tokio::sync::broadcast::error::RecvError;
use tracing::{debug, error};
use url::Url;

use kabu_core_actors::{subscribe, Actor, ActorResult, Broadcaster, Consumer, WorkerResult};
use kabu_core_actors_macros::{Accessor, Consumer};
use kabu_core_blockchain::Blockchain;
use kabu_types_events::{MessageTxCompose, RlpState, TxComposeData, TxComposeMessageType};

#[derive(Clone)]
pub struct RelayConfig {
    pub id: u64,
    pub url: String,
    pub name: String,
    pub no_sign: Option<bool>,
}

#[derive(Clone)]
struct FlashbotsRelay {
    url: String,
    signer: Option<PrivateKeySigner>,
    name: String,
}

impl FlashbotsRelay {
    fn new(url: &str, signer: Option<PrivateKeySigner>) -> Result<Self> {
        Ok(FlashbotsRelay { url: url.to_string(), signer, name: url.to_string() })
    }

    async fn send_bundle(&self, bundle: EthSendBundle) -> Result<()> {
        let provider = ProviderBuilder::new().connect_http(Url::parse(&self.url)?);

        let result = match self.signer.clone() {
            Some(signer) => provider.send_bundle(bundle).with_auth(signer).await,
            None => provider.send_bundle(bundle).await,
        };

        match result {
            Ok(_resp) => {
                debug!("Bundle sent to: {}", self.name);
                Ok(())
            }
            Err(error) => {
                error!("{} {}", self.name, error.to_string());
                Err(eyre!("FLASHBOTS_RELAY_ERROR"))
            }
        }
    }
}

async fn broadcast_task(broadcast_request: TxComposeData, relays: Arc<Vec<FlashbotsRelay>>) -> Result<()> {
    let block_number = broadcast_request.next_block_number;

    if let Some(rlp_bundle) = broadcast_request.rlp_bundle.clone() {
        let stuffing_rlp_bundle: Vec<Bytes> = rlp_bundle.iter().map(|item| item.unwrap()).collect();
        let backrun_rlp_bundle: Vec<Bytes> =
            rlp_bundle.iter().filter(|item| matches!(item, RlpState::Backrun(_))).map(|item| item.unwrap()).collect();

        if stuffing_rlp_bundle.iter().any(|i| i.is_empty()) || backrun_rlp_bundle.iter().any(|i| i.is_empty()) {
            return Err(eyre!("RLP_BUNDLE_IS_INCORRECT"));
        }

        // Send backrun bundle
        let backrun_bundle = EthSendBundle { txs: backrun_rlp_bundle, block_number, ..Default::default() };

        // Send stuffing bundle
        let stuffing_bundle = EthSendBundle { txs: stuffing_rlp_bundle, block_number, ..Default::default() };

        // Broadcast to all relays concurrently
        for relay in relays.iter() {
            let relay_clone = relay.clone();
            let backrun_bundle_clone = backrun_bundle.clone();
            let stuffing_bundle_clone = stuffing_bundle.clone();

            tokio::spawn(async move {
                debug!("Sending bundles to {}", relay_clone.name);

                if let Err(e) = relay_clone.send_bundle(backrun_bundle_clone).await {
                    error!("Failed to send backrun bundle to {}: {}", relay_clone.name, e);
                }

                if let Err(e) = relay_clone.send_bundle(stuffing_bundle_clone).await {
                    error!("Failed to send stuffing bundle to {}: {}", relay_clone.name, e);
                }
            });
        }

        Ok(())
    } else {
        error!("rlp_bundle is None");
        Err(eyre!("RLP_BUNDLE_IS_NONE"))
    }
}

async fn flashbots_broadcaster_worker(
    relays: Arc<Vec<FlashbotsRelay>>,
    bundle_rx: Broadcaster<MessageTxCompose>,
    allow_broadcast: bool,
) -> WorkerResult {
    subscribe!(bundle_rx);

    loop {
        tokio::select! {
            msg = bundle_rx.recv() => {
                let broadcast_msg : Result<MessageTxCompose, RecvError> = msg;
                match broadcast_msg {
                    Ok(compose_request) => {
                        if let TxComposeMessageType::Broadcast(broadcast_request) = compose_request.inner {
                            if allow_broadcast {
                                tokio::task::spawn(
                                    broadcast_task(
                                        broadcast_request,
                                        relays.clone(),
                                    )
                                );
                            }
                        }
                    }
                    Err(e) => {
                        error!("flashbots_broadcaster_worker {}", e)
                    }
                }
            }
        }
    }
}

#[derive(Accessor, Consumer)]
pub struct FlashbotsBroadcastActor {
    relays: Arc<Vec<FlashbotsRelay>>,
    signer: Arc<PrivateKeySigner>,
    #[consumer]
    tx_compose_channel_rx: Option<Broadcaster<MessageTxCompose>>,
    allow_broadcast: bool,
}

impl FlashbotsBroadcastActor {
    pub fn new(signer: Option<PrivateKeySigner>, allow_broadcast: bool) -> Result<Self> {
        let signer = Arc::new(signer.unwrap_or(PrivateKeySigner::random()));
        let relays = Arc::new(Vec::new());

        Ok(FlashbotsBroadcastActor { relays, signer, tx_compose_channel_rx: None, allow_broadcast })
    }

    pub fn with_default_relays(mut self) -> Result<Self> {
        let relay_configs = vec![
            ("https://relay.flashbots.net", false),
            ("https://rpc.beaverbuild.org/", false),
            ("https://rpc.titanbuilder.xyz", false),
            ("https://rsync-builder.xyz", false),
            ("https://api.edennetwork.io/v1/bundle", false),
            ("https://eth-builder.com", true),
            ("https://api.securerpc.com/v1", true),
            ("https://BuildAI.net", true),
            ("https://rpc.payload.de", true),
            ("https://rpc.f1b.io", false),
            ("https://rpc.lokibuilder.xyz", false),
            ("https://rpc.ibuilder.xyz", false),
            ("https://rpc.jetbldr.xyz", false),
            ("https://rpc.penguinbuild.org", false),
            ("https://builder.gmbit.co/rpc", false),
        ];

        let mut relays = Vec::new();
        for (url, no_sign) in relay_configs {
            let signer = if no_sign { None } else { Some((*self.signer).clone()) };
            relays.push(FlashbotsRelay::new(url, signer)?);
        }

        self.relays = Arc::new(relays);
        Ok(self)
    }

    pub fn with_relay(mut self, url: &str) -> Result<Self> {
        let mut relays = Arc::try_unwrap(self.relays).unwrap_or_else(|arc| (*arc).clone());

        relays.push(FlashbotsRelay::new(url, Some((*self.signer).clone()))?);
        self.relays = Arc::new(relays);
        Ok(self)
    }

    pub fn with_relays(mut self, relay_configs: Vec<RelayConfig>) -> Result<Self> {
        let mut relays = Vec::new();

        for config in relay_configs {
            let signer = if config.no_sign.unwrap_or(false) { None } else { Some((*self.signer).clone()) };
            relays.push(FlashbotsRelay::new(&config.url, signer)?);
        }

        self.relays = Arc::new(relays);
        Ok(self)
    }

    pub fn on_bc(self, bc: &Blockchain) -> Self {
        Self { tx_compose_channel_rx: Some(bc.tx_compose_channel()), ..self }
    }
}

impl Actor for FlashbotsBroadcastActor {
    fn start(&self) -> ActorResult {
        let task = tokio::task::spawn(flashbots_broadcaster_worker(
            self.relays.clone(),
            self.tx_compose_channel_rx.clone().unwrap(),
            self.allow_broadcast,
        ));
        Ok(vec![task])
    }

    fn name(&self) -> &'static str {
        "FlashbotsBroadcastActor"
    }
}
