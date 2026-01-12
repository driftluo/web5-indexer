mod http_server;
mod molecule;
mod monitor;
mod pg_read;
mod pg_write;
mod rpc_client;
mod types;

pub use http_server::{did_from_addr, did_from_id};
pub use monitor::did_monitor;
pub use rpc_client::{CKB_MAINNET_RPC, CKB_TESTNET_RPC, Network, RpcClient, did_script};
pub use types::*;

use std::env;

const INIT_SQL: &str = include_str!("../db_schema/create_table.sql");

static PG_POOL: std::sync::OnceLock<sqlx::Pool<sqlx::Postgres>> = std::sync::OnceLock::new();

pub async fn create_pg_pool() {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or("postgres://postgres:password@localhost:5432/postgres".to_string());
    let pool = sqlx::Pool::<sqlx::Postgres>::connect(&database_url)
        .await
        .expect("Failed to create Postgres connection pool");
    PG_POOL.set(pool).expect("PG_POOL already set");
}

pub fn get_pg_pool() -> &'static sqlx::Pool<sqlx::Postgres> {
    PG_POOL.get().expect("PG_POOL not initialized")
}

pub async fn init_db(pool: &sqlx::Pool<sqlx::Postgres>) {
    use sqlx::Row;
    let need_init =
        sqlx::query("SELECT EXISTS(SELECT 1 FROM pg_tables WHERE tablename = 'did_documents')")
            .fetch_one(pool)
            .await
            .map(|row| !row.get::<bool, _>(0))
            .expect("Failed to check if database needs initialization");

    if need_init {
        sqlx::raw_sql(INIT_SQL)
            .execute(pool)
            .await
            .expect("Failed to execute initialization SQL");
    }
    init_global_cache(pool).await;
}

async fn init_global_cache(pool: &sqlx::Pool<sqlx::Postgres>) {
    use ckb_jsonrpc_types::BlockNumber;
    use sqlx::Row;
    let tip_number =
        sqlx::query("SELECT block_number FROM did_documents ORDER BY block_number DESC LIMIT 1")
            .fetch_one(pool)
            .await
            .ok()
            .map(|row: sqlx::postgres::PgRow| {
                let raw_block_number: Option<String> = row.try_get("block_number").ok();
                raw_block_number.map(|raw_block_number| {
                    let mut bn = [0u8; 8];
                    faster_hex::hex_decode(raw_block_number.as_bytes(), &mut bn).unwrap();
                    BlockNumber::from(u64::from_be_bytes(bn))
                })
            })
            .flatten()
            .unwrap_or_else(|| 0.into());
    let tip_number_testnet = sqlx::query(
        "SELECT block_number FROM did_documents_testnet ORDER BY block_number DESC LIMIT 1",
    )
    .fetch_one(pool)
    .await
    .ok()
    .map(|row: sqlx::postgres::PgRow| {
        let raw_block_number: Option<String> = row.try_get("block_number").ok();
        raw_block_number.map(|raw_block_number| {
            let mut bn = [0u8; 8];
            faster_hex::hex_decode(raw_block_number.as_bytes(), &mut bn).unwrap();
            BlockNumber::from(u64::from_be_bytes(bn))
        })
    })
    .flatten()
    .unwrap_or_else(|| 0.into());

    let global_cache = pg_write::global_cache();
    global_cache.store(std::sync::Arc::new(tip_number));
    let global_cache_testnet = pg_write::global_cache_testnet();
    global_cache_testnet.store(std::sync::Arc::new(tip_number_testnet));
}
