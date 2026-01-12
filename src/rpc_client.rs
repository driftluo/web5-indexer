use reqwest::{Client, Url};

use std::{
    future::Future,
    io,
    str::FromStr,
    sync::{
        Arc, LazyLock,
        atomic::{AtomicU64, Ordering},
    },
};

use crate::types::{Cell, IndexerTip, Order, Pagination, SearchKey, Tx};
use ckb_jsonrpc_types::{
    BlockNumber, HeaderView, JsonBytes, Script, ScriptHashType, TransactionView, TxStatus, Uint32,
};
use ckb_sdk::NetworkType;
use ckb_types::{H256, h256};
use serde::{Deserialize, Serialize};

pub static CKB_MAINNET_RPC: LazyLock<Url> = LazyLock::new(|| {
    std::env::var("CKB_MAINNET_RPC_URL")
        .ok()
        .and_then(|url| Url::parse(&url).ok())
        .unwrap_or(Url::parse("https://mainnet.ckb.dev").unwrap())
});

pub static CKB_TESTNET_RPC: LazyLock<Url> = LazyLock::new(|| {
    std::env::var("CKB_TESTNET_RPC_URL")
        .ok()
        .and_then(|url| Url::parse(&url).ok())
        .unwrap_or(Url::parse("https://testnet.ckb.dev").unwrap())
});

macro_rules! jsonrpc {
    ($method:expr, $self:ident, $url:expr, $return:ty$(, $params:ident$(,)?)*) => {{
        let old = $self.id.fetch_add(1, Ordering::AcqRel);
        let data = format!(
            r#"{{"id": {}, "jsonrpc": "2.0", "method": "{}", "params": {}}}"#,
            old,
            $method,
            serde_json::to_value(($($params,)*)).unwrap()
        );

        let req_json: serde_json::Value = serde_json::from_str(&data).unwrap();

        let c = $self.raw.post($url).json(&req_json);
        let c = if let Some(token) = &$self.bearer_token {
            c.bearer_auth(token)
        } else {
            c
        };
        async {
            let resp = c
                .send()
                .await
                .map_err::<io::Error, _>(|e| io::Error::new(io::ErrorKind::ConnectionAborted, format!("{:?}", e)))?;
            let output = resp
                .json::<jsonrpc_core::response::Output>()
                .await
                .map_err::<io::Error, _>(|e| io::Error::new(io::ErrorKind::InvalidData, format!("{:?}", e)))?;

            match output {
                jsonrpc_core::response::Output::Success(success) => {
                    Ok(serde_json::from_value::<$return>(success.result).unwrap())
                }
                jsonrpc_core::response::Output::Failure(e) => {
                    Err(io::Error::new(io::ErrorKind::InvalidData, format!("{:?}", e)))
                }
            }
        }
    }}
}

// Default implementation of ckb Rpc client
#[derive(Clone)]
pub struct RpcClient {
    raw: Client,
    id: Arc<AtomicU64>,
    bearer_token: Option<String>,
}

impl Default for RpcClient {
    fn default() -> Self {
        Self::new()
    }
}

impl RpcClient {
    pub fn new() -> Self {
        RpcClient {
            raw: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap(),
            id: Arc::new(AtomicU64::new(0)),
            bearer_token: None,
        }
    }

    pub fn set_bearer_token(&mut self, token: Option<String>) {
        self.bearer_token = token;
    }

    pub fn get_transaction(
        &self,
        url: Url,
        hash: &H256,
    ) -> impl Future<Output = Result<Option<TransactionView>, io::Error>> {
        #[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
        struct TransactionWithStatusResponse {
            /// The transaction.
            pub transaction: Option<TransactionView>,
            /// The Transaction status.
            pub tx_status: TxStatus,
        }
        let task = jsonrpc!(
            "get_transaction",
            self,
            url,
            TransactionWithStatusResponse,
            hash
        );
        async {
            let res = task.await?;
            Ok(res.transaction)
        }
    }

    pub fn get_transactions(
        &self,
        url: Url,
        search_key: SearchKey,
        order: Order,
        limit: Uint32,
        after: Option<JsonBytes>,
    ) -> impl Future<Output = Result<Pagination<Tx>, io::Error>> {
        jsonrpc!(
            "get_transactions",
            self,
            url,
            Pagination<Tx>,
            search_key,
            order,
            limit,
            after
        )
    }

    pub fn get_cells(
        &self,
        url: Url,
        search_key: SearchKey,
        order: Order,
        limit: Uint32,
        after: Option<JsonBytes>,
    ) -> impl Future<Output = Result<Pagination<Cell>, io::Error>> {
        jsonrpc!(
            "get_cells",
            self,
            url,
            Pagination<Cell>,
            search_key,
            order,
            limit,
            after
        )
    }

    pub fn get_indexer_tip(&self, url: Url) -> impl Future<Output = Result<IndexerTip, io::Error>> {
        jsonrpc!("get_indexer_tip", self, url, IndexerTip)
    }

    pub fn get_header_by_number(
        &self,
        url: Url,
        number: BlockNumber,
    ) -> impl Future<Output = Result<HeaderView, io::Error>> {
        jsonrpc!("get_header_by_number", self, url, HeaderView, number)
    }
}

pub static MAINNET_CODE_HASH: LazyLock<H256> = LazyLock::new(|| {
    std::env::var("MAINNET_CODE_HASH")
        .ok()
        .and_then(|s| {
            let s = if s.len() < 2 {
                &s
            } else if &s[..2] == "0x" {
                &s[2..]
            } else {
                &s
            };
            H256::from_str(s).ok()
        })
        .unwrap_or(h256!(
            "0x4a06164dc34dccade5afe3e847a97b6db743e79f5477fa3295acf02849c5984a"
        ))
});

pub static TESTNET_CODE_HASH: LazyLock<H256> = LazyLock::new(|| {
    std::env::var("TESTNET_CODE_HASH")
        .ok()
        .and_then(|s| {
            let s = if s.len() < 2 {
                &s
            } else if &s[..2] == "0x" {
                &s[2..]
            } else {
                &s
            };
            H256::from_str(s).ok()
        })
        .unwrap_or(h256!(
            "0x510150477b10d6ab551a509b71265f3164e9fd4137fcb5a4322f49f03092c7c5"
        ))
});

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default)]
pub enum Network {
    #[serde(alias = "mainnet")]
    #[default]
    Mainnet,
    #[serde(alias = "testnet")]
    Testnet,
}

impl From<Network> for NetworkType {
    fn from(net: Network) -> Self {
        match net {
            Network::Mainnet => NetworkType::Mainnet,
            Network::Testnet => NetworkType::Testnet,
        }
    }
}

impl Network {
    pub fn did(&self) -> &str {
        match self {
            Network::Mainnet => "did_documents",
            Network::Testnet => "did_documents_testnet",
        }
    }
}

pub fn did_script(net: Network, args: JsonBytes) -> Script {
    Script {
        code_hash: match net {
            Network::Mainnet => MAINNET_CODE_HASH.clone(),
            Network::Testnet => TESTNET_CODE_HASH.clone(),
        },
        hash_type: ScriptHashType::Type,
        args,
    }
}
