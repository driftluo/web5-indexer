use crate::{http_server::Params, types::Web5DocumentData};
use sqlx::{FromRow, Pool, Postgres, types::Json};

const PAGE_SIZE: usize = 500;

#[derive(FromRow)]
pub(crate) struct DidRead {
    did: String,
    handle: String,
    signing_key: String,
    ckb_address: String,
    tx_hash: String,
    block_number: String,
    outpoint: String,
    did_document: Json<Web5DocumentData>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl DidRead {
    pub async fn fetch_by_did(
        conn: &Pool<Postgres>,
        params: Params,
    ) -> sqlx::Result<(Vec<serde_json::Value>, usize)> {
        let page_size = std::cmp::min(params.page_size.unwrap_or(PAGE_SIZE), PAGE_SIZE);
        let offset = params.page.saturating_mul(page_size);
        let sql = format!(
            r#"SELECT did, handle, signing_key, ckb_address, tx_hash, block_number, outpoint, did_document, created_at
            FROM {} 
            WHERE did = $1 AND valid = true ORDER BY created_at DESC LIMIT {page_size} OFFSET {offset}"#,
            params.net.did()
        );

        let record: Vec<serde_json::Value> = sqlx::query_as::<_, DidRead>(&sql)
            .bind(params.name)
            .fetch_all(conn)
            .await?
            .iter()
            .map(|r| {
                serde_json::json!({
                    "did": r.did,
                    "handle": r.handle,
                    "signing_key": r.signing_key,
                    "ckb_address": r.ckb_address,
                    "tx_hash": format!("0x{}", r.tx_hash),
                    "block_number": format!("0x{}", r.block_number),
                    "outpoint": format!("0x{}", r.outpoint),
                    "did_document": r.did_document,
                    "created_at": r.created_at.to_rfc3339(),

                })
            })
            .collect();
        Ok((record, params.page.saturating_add(1)))
    }

    pub async fn fetch_by_address(
        conn: &Pool<Postgres>,
        params: Params,
    ) -> sqlx::Result<(Vec<serde_json::Value>, usize)> {
        let page_size = std::cmp::min(params.page_size.unwrap_or(PAGE_SIZE), PAGE_SIZE);
        let offset = params.page.saturating_mul(page_size);
        let sql = format!(
            r#"SELECT did, handle, signing_key, ckb_address, tx_hash, block_number, outpoint, did_document, created_at
            FROM {} 
            WHERE ckb_address = $1 AND valid = true ORDER BY created_at DESC LIMIT {page_size} OFFSET {offset}"#,
            params.net.did()
        );
        let did = faster_hex::hex_string(&params.name.as_bytes());
        let record: Vec<serde_json::Value> = sqlx::query_as::<_, DidRead>(&sql)
            .bind(did)
            .fetch_all(conn)
            .await?
            .iter()
            .map(|r| {
                serde_json::json!({
                    "did": r.did,
                    "handle": r.handle,
                    "signing_key": r.signing_key,
                    "ckb_address": r.ckb_address,
                    "tx_hash": format!("0x{}", r.tx_hash),
                    "block_number": format!("0x{}", r.block_number),
                    "outpoint": format!("0x{}", r.outpoint),
                    "did_document": r.did_document,
                    "created_at": r.created_at.to_rfc3339(),

                })
            })
            .collect();
        Ok((record, params.page.saturating_add(1)))
    }
}
