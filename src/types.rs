use ckb_jsonrpc_types::{BlockNumber, CellOutput, JsonBytes, OutPoint, Script, Uint32, Uint64};
use ckb_sdk::{Address, AddressPayload, NetworkType};
use ckb_types::{H256, packed};
use molecule::prelude::Entity;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::molecule::did_cell::{Bytes, DidWeb5Data, DidWeb5DataUnion};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Order {
    Desc,
    Asc,
}

#[derive(Serialize, Deserialize)]
pub struct Pagination<T> {
    pub objects: Vec<T>,
    pub last_cursor: JsonBytes,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum CellType {
    Input,
    Output,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TxWithCell {
    pub tx_hash: H256,
    pub block_number: BlockNumber,
    pub tx_index: Uint32,
    pub io_index: Uint32,
    pub io_type: CellType,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TxWithCells {
    pub tx_hash: H256,
    pub block_number: BlockNumber,
    pub tx_index: Uint32,
    pub cells: Vec<(CellType, Uint32)>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum Tx {
    Ungrouped(TxWithCell),
    Grouped(TxWithCells),
}

impl Tx {
    pub fn tx_hash(&self) -> H256 {
        match self {
            Tx::Ungrouped(tx) => tx.tx_hash.clone(),
            Tx::Grouped(tx) => tx.tx_hash.clone(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum IndexerScriptSearchMode {
    /// Mode `prefix` search script with prefix
    #[default]
    Prefix,
    /// Mode `exact` search script with exact match
    Exact,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchKey {
    pub script: Script,
    pub script_type: ScriptType,
    pub script_search_mode: Option<IndexerScriptSearchMode>,
    pub filter: Option<SearchKeyFilter>,
    pub with_data: Option<bool>,
    pub group_by_transaction: Option<bool>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct SearchKeyFilter {
    pub script: Option<Script>,
    pub script_len_range: Option<[Uint64; 2]>,
    pub output_data_len_range: Option<[Uint64; 2]>,
    pub output_capacity_range: Option<[Uint64; 2]>,
    pub block_range: Option<[BlockNumber; 2]>,
}

impl SearchKeyFilter {
    pub fn block_range(start: BlockNumber, end: BlockNumber) -> Self {
        Self {
            script: None,
            script_len_range: None,
            output_data_len_range: None,
            output_capacity_range: None,
            block_range: Some([start, end]),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScriptType {
    Lock,
    Type,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Cell {
    pub output: CellOutput,
    pub output_data: Option<JsonBytes>,
    pub out_point: OutPoint,
    pub block_number: BlockNumber,
    pub tx_index: Uint32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IndexerTip {
    pub block_hash: H256,
    pub block_number: BlockNumber,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Service {
    #[serde(rename = "type")]
    pub r#type: String,
    pub endpoint: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Web5DocumentData {
    #[serde(rename = "verificationMethods")]
    pub verification_methods: BTreeMap<String, String>,
    #[serde(rename = "alsoKnownAs")]
    pub also_known_as: Vec<String>,
    pub services: BTreeMap<String, Service>,
}

pub fn calculate_address(lock_script: &packed::Script, network: NetworkType) -> Address {
    let payload = AddressPayload::from(lock_script.clone());
    Address::new(network, payload, true)
}

pub fn parse_didoc_cell(cell_data: &[u8]) -> Option<Web5DocumentData> {
    let did_data = DidWeb5Data::from_slice(cell_data).ok()?;
    let DidWeb5DataUnion::DidWeb5DataV1(did_data_v1) = did_data.to_enum();
    let did_doc: Bytes = did_data_v1.document();
    serde_ipld_dagcbor::from_slice(&did_doc.raw_data()).ok()
}

pub fn check_did_doc(doc: &Web5DocumentData) -> Option<(String, String)> {
    if doc.also_known_as.len() == 0 || !doc.also_known_as[0].starts_with("at://") {
        return None;
    }
    if doc.services.len() == 0 {
        return None;
    }
    let handle = doc.also_known_as[0][5..].to_string();
    if let Some(key) = doc.verification_methods.get("atproto") {
        if !check_signing_key_str(key) {
            None
        } else {
            Some((handle, key.clone()))
        }
    } else {
        None
    }
}

pub fn check_signing_key_str(did: &str) -> bool {
    did.starts_with("did:key")
}

pub fn calculate_web5_did(args: &[u8]) -> String {
    data_encoding::BASE32.encode(args).to_lowercase()
}
