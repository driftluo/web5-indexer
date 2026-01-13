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
    cell_data: String,
    lock_script_hash: String,
    did_document: Json<Web5DocumentData>,
    created_at: chrono::DateTime<chrono::Utc>,
    consumed_tx: Option<String>,
    consumed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl DidRead {
    pub async fn fetch_by_did(
        conn: &Pool<Postgres>,
        params: Params,
    ) -> sqlx::Result<(Vec<serde_json::Value>, usize)> {
        let page_size = std::cmp::min(params.page_size.unwrap_or(PAGE_SIZE), PAGE_SIZE);
        let offset = params.page.saturating_mul(page_size);
        let sql = format!(
            r#"SELECT did, handle, signing_key, ckb_address, tx_hash, block_number, outpoint, did_document, cell_data, consumed_tx, created_at, consumed_at, lock_script_hash
            FROM {} 
            WHERE did = $1 ORDER BY created_at DESC LIMIT {page_size} OFFSET {offset}"#,
            params.net.did()
        );

        let record: Vec<serde_json::Value> = Self::fetch_doc(conn, &sql, &params.name).await?;
        Ok((record, params.page.saturating_add(1)))
    }

    pub async fn fetch_by_address(
        conn: &Pool<Postgres>,
        params: Params,
    ) -> sqlx::Result<(Vec<serde_json::Value>, usize)> {
        let page_size = std::cmp::min(params.page_size.unwrap_or(PAGE_SIZE), PAGE_SIZE);
        let offset = params.page.saturating_mul(page_size);
        let sql = format!(
            r#"SELECT did, handle, signing_key, ckb_address, tx_hash, block_number, outpoint, did_document, cell_data, consumed_tx, created_at, consumed_at, lock_script_hash
            FROM {} 
            WHERE ckb_address = $1 ORDER BY created_at DESC LIMIT {page_size} OFFSET {offset}"#,
            params.net.did()
        );

        let record: Vec<serde_json::Value> = Self::fetch_doc(conn, &sql, &params.name).await?;
        Ok((record, params.page.saturating_add(1)))
    }

    pub async fn fetch_by_signing_key(
        conn: &Pool<Postgres>,
        params: Params,
    ) -> sqlx::Result<(Vec<serde_json::Value>, usize)> {
        let page_size = std::cmp::min(params.page_size.unwrap_or(PAGE_SIZE), PAGE_SIZE);
        let offset = params.page.saturating_mul(page_size);
        let sql = format!(
            r#"SELECT did, handle, signing_key, ckb_address, tx_hash, block_number, outpoint, did_document, cell_data, consumed_tx, created_at, consumed_at, lock_script_hash
            FROM {} 
            WHERE signing_key = $1 ORDER BY created_at DESC LIMIT {page_size} OFFSET {offset}"#,
            params.net.did()
        );

        let record: Vec<serde_json::Value> = Self::fetch_doc(conn, &sql, &params.name).await?;
        Ok((record, params.page.saturating_add(1)))
    }

    pub async fn fetch_by_handle(
        conn: &Pool<Postgres>,
        params: Params,
    ) -> sqlx::Result<(Vec<serde_json::Value>, usize)> {
        let page_size = std::cmp::min(params.page_size.unwrap_or(PAGE_SIZE), PAGE_SIZE);
        let offset = params.page.saturating_mul(page_size);
        let sql = format!(
            r#"SELECT did, handle, signing_key, ckb_address, tx_hash, block_number, outpoint, did_document, cell_data, consumed_tx, created_at, consumed_at, lock_script_hash
            FROM {} 
            WHERE handle = $1 ORDER BY created_at DESC LIMIT {page_size} OFFSET {offset}"#,
            params.net.did()
        );

        let record: Vec<serde_json::Value> = Self::fetch_doc(conn, &sql, &params.name).await?;
        Ok((record, params.page.saturating_add(1)))
    }

    pub async fn fetch_by_lock_script_hash(
        conn: &Pool<Postgres>,
        params: Params,
    ) -> sqlx::Result<(Vec<serde_json::Value>, usize)> {
        let page_size = std::cmp::min(params.page_size.unwrap_or(PAGE_SIZE), PAGE_SIZE);
        let offset = params.page.saturating_mul(page_size);
        let sql = format!(
            r#"SELECT did, handle, signing_key, ckb_address, tx_hash, block_number, outpoint, did_document, cell_data, consumed_tx, created_at, consumed_at, lock_script_hash
            FROM {} 
            WHERE lock_script_hash = $1 ORDER BY created_at DESC LIMIT {page_size} OFFSET {offset}"#,
            params.net.did()
        );
        let name = if params.name.starts_with("0x") {
            params.name[2..].to_string()
        } else {
            params.name
        };
        let record: Vec<serde_json::Value> = Self::fetch_doc(conn, &sql, &name).await?;
        Ok((record, params.page.saturating_add(1)))
    }

    async fn fetch_doc(
        conn: &Pool<Postgres>,
        sql: &str,
        key: &str,
    ) -> sqlx::Result<Vec<serde_json::Value>> {
        Ok(sqlx::query_as::<_, DidRead>(&sql)
            .bind(key)
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
                    "cell_data": format!("0x{}", r.cell_data),
                    "lock_script_hash": format!("0x{}", r.lock_script_hash),
                    "did_document": r.did_document,
                    "created_at": r.created_at.to_rfc3339(),
                    "consumed_tx": r.consumed_tx.as_ref().map(|tx| format!("0x{}", tx)),
                    "consumed_at": r.consumed_at.map(|dt| dt.to_rfc3339()),
                })
            })
            .collect())
    }
}
