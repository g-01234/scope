use crate::utils;
use ethers::{
    prelude::{LocalWallet, Provider, SignerMiddleware},
    providers::{Http, Middleware, ProviderExt},
    signers::Signer,
    types::{H160, U256},
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ClientProviderWrapper {
    pub endpoint: String,
    pub chain_id: U256,
    pub selected_address: H160,
    pub client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
}

impl ClientProviderWrapper {
    pub async fn new(endpoint: String) -> Result<Self, Box<dyn std::error::Error>> {
        // Connect to the network
        let provider = Provider::<Http>::connect(&endpoint).await;

        // Get chain id
        let chain_id = provider.get_chainid().await?;

        // Instantiate the wallet
        let wallet: LocalWallet = utils::key(0).with_chain_id(chain_id.as_u64());

        // Instantiate the client with the wallet
        let client = Arc::new(SignerMiddleware::new(provider, wallet));

        Ok(Self {
            endpoint,
            chain_id,
            selected_address: client.address(),
            client,
        })
    }

    pub fn set_address(&mut self, new_address: String) {
        let address_stripped = new_address.strip_prefix("0x").unwrap_or(&new_address);
        let address_bytes_vec = hex::decode(address_stripped).expect("Decoding failed");

        // Ensure the decoded byte vector has exactly 20 bytes
        assert_eq!(address_bytes_vec.len(), 20);

        let mut address_bytes_array = [0u8; 20];
        address_bytes_array.copy_from_slice(&address_bytes_vec);

        self.selected_address = H160::from(address_bytes_array);
    }

    pub fn set_endpoint(&mut self, new_endpoint: String) {
        self.endpoint = new_endpoint;
    }
}
