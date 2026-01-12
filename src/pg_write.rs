use crate::{Network, types::Web5DocumentData};
use ckb_jsonrpc_types::BlockNumber;
use sqlx::{PgConnection, QueryBuilder, types::Json};

use arc_swap::ArcSwap;
use std::sync::{Arc, LazyLock};

pub(crate) fn global_cache() -> &'static ArcSwap<BlockNumber> {
    static GLOBAL_CACHE: LazyLock<ArcSwap<BlockNumber>> =
        LazyLock::new(|| ArcSwap::new(Arc::new(0.into())));
    &GLOBAL_CACHE
}

pub(crate) fn global_cache_testnet() -> &'static ArcSwap<BlockNumber> {
    static GLOBAL_CACHE: LazyLock<ArcSwap<BlockNumber>> =
        LazyLock::new(|| ArcSwap::new(Arc::new(0.into())));
    &GLOBAL_CACHE
}

pub(crate) struct DidWrite {
    did: String,
    handle: String,
    signing_key: String,
    ckb_address: String,
    tx_hash: String,
    block_number: String,
    outpoint: String,
    did_document: Json<Web5DocumentData>,
    valid: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl DidWrite {
    pub fn new(
        did: String,
        handle: String,
        signing_key: String,
        ckb_address: String,
        tx_hash: String,
        block_number: String,
        outpoint: String,
        did_document: Json<Web5DocumentData>,
        created_at: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            did,
            handle,
            signing_key,
            ckb_address,
            tx_hash,
            block_number,
            outpoint,
            did_document,
            valid: true,
            created_at,
        }
    }

    pub async fn insert_batch(
        conn: &mut PgConnection,
        dids: &[DidWrite],
        net: Network,
    ) -> Result<(), sqlx::Error> {
        if dids.is_empty() {
            return Ok(());
        }
        let sql = format!(
            "INSERT INTO {} (did, handle, signing_key, ckb_address, tx_hash, block_number, outpoint, did_document, valid, created_at) ",
            net.did()
        );

        let mut query_builder: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(sql);
        query_builder.push_values(dids.iter().take(65535 / 10), |mut b, did_write| {
            b.push_bind(&did_write.did)
                .push_bind(&did_write.handle)
                .push_bind(&did_write.signing_key)
                .push_bind(&did_write.ckb_address)
                .push_bind(&did_write.tx_hash)
                .push_bind(&did_write.block_number)
                .push_bind(&did_write.outpoint)
                .push_bind(&did_write.did_document)
                .push_bind(&did_write.valid)
                .push_bind(&did_write.created_at);
        });
        query_builder.push(" ON CONFLICT (did) DO NOTHING");
        query_builder.build().execute(conn).await?;
        Ok(())
    }
}

pub(crate) struct DidDelete {
    outpoint: String,
}

impl DidDelete {
    pub fn new(outpoint: String) -> Self {
        Self { outpoint }
    }

    pub async fn delete_batch(
        conn: &mut PgConnection,
        deletes: &[DidDelete],
        net: Network,
    ) -> Result<(), sqlx::Error> {
        if deletes.is_empty() {
            return Ok(());
        }
        let sql = format!(
            "UPDATE {} SET valid = false WHERE outpoint = ANY($1)",
            net.did()
        );
        let outpoints: Vec<&str> = deletes.iter().map(|d| d.outpoint.as_str()).collect();
        sqlx::query(&sql).bind(&outpoints).execute(conn).await?;
        Ok(())
    }
}
