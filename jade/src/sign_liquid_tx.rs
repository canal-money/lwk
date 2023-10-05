use serde::{Deserialize, Serialize};

use crate::Network;

#[derive(Debug, Deserialize, Serialize)]
pub struct SignLiquidTxParams {
    pub network: Network,

    #[serde(with = "serde_bytes")]
    pub txn: Vec<u8>,

    pub num_inputs: u32,

    pub use_ae_signatures: bool,

    pub change: Vec<Option<Change>>,

    pub asset_info: Vec<AssetInfo>,

    pub trusted_commitments: Vec<Option<Commitment>>,

    pub additional_info: Option<AdditionalInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Commitment {
    #[serde(with = "serde_bytes")]
    pub asset_generator: Vec<u8>,

    #[serde(with = "serde_bytes")]
    pub asset_id: Vec<u8>,

    #[serde(with = "serde_bytes")]
    pub blinding_key: Vec<u8>,

    pub value: u64,

    #[serde(with = "serde_bytes")]
    pub value_commitment: Vec<u8>,

    #[serde(with = "serde_bytes")]
    pub value_blind_proof: Vec<u8>,

    #[serde(with = "serde_bytes")]
    pub asset_blind_proof: Vec<u8>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Change {
    pub variant: String,
    pub path: Vec<u32>,
    pub is_change: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AssetInfo {
    #[serde(with = "serde_bytes")]
    pub asset_id: Vec<u8>,
    pub contract: Contract,
    pub issuance_prevout: Prevout,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Contract {
    pub entity: Entity,

    pub issuer_pubkey: String,
    pub name: String,
    pub precision: u8,
    pub ticker: String,
    pub version: u8,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AdditionalInfo {
    pub tx_type: String,
    pub wallet_input_summary: Vec<Summary>,
    pub wallet_output_summary: Vec<Summary>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Summary {
    #[serde(with = "serde_bytes")]
    pub asset_id: Vec<u8>,
    pub satoshi: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Prevout {
    #[serde(with = "serde_bytes")]
    pub txid: Vec<u8>,
    pub vout: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Entity {
    pub domain: String,
}

#[cfg(test)]
mod test {
    use ureq::serde_json;

    use crate::protocol::Request;

    use super::SignLiquidTxParams;

    #[test]
    fn parse_sign_liquid_tx() {
        let json = include_str!("../test_data/sign_liquid_tx_request.json");

        let _resp: Request<SignLiquidTxParams> = serde_json::from_str(json).unwrap();
    }
}