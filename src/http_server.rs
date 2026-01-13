use crate::{Network, get_pg_pool, pg_read::DidRead};

use salvo::{Request, Response, handler, macros::Extractible};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Extractible)]
#[salvo(extract(default_source(from = "query")))]
pub(crate) struct Params {
    #[serde(default)]
    pub(crate) net: Network,
    #[serde(alias = "address")]
    #[serde(alias = "did")]
    #[serde(alias = "signing_key")]
    #[serde(alias = "handle")]
    #[serde(alias = "lock_script_hash")]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) page: usize,
    pub(crate) page_size: Option<usize>,
}

#[handler]
pub async fn did_from_id(req: &mut Request, _res: &mut Response) -> Result<String, salvo::Error> {
    let params: Params = req.extract().await?;
    let pool = get_pg_pool();
    let res = DidRead::fetch_by_did(&pool, params)
        .await
        .map(|(records, next_page)| {
            serde_json::json!({
                "records": records,
                "next_page": next_page
            })
        })
        .map_err(|e| {
            log::warn!("fetch from id error: {}", e);
            salvo::Error::Io(std::io::Error::other("Failed to fetch did from id"))
        })?;

    Ok(res.to_string())
}

#[handler]
pub async fn did_from_addr(req: &mut Request, _res: &mut Response) -> Result<String, salvo::Error> {
    let params: Params = req.extract().await?;
    let pool = get_pg_pool();
    let res = DidRead::fetch_by_address(&pool, params)
        .await
        .map(|(records, next_page)| {
            serde_json::json!({
                "records": records,
                "next_page": next_page
            })
        })
        .map_err(|e| {
            log::warn!("fetch from address error: {}", e);
            salvo::Error::Io(std::io::Error::other("Failed to fetch did from address"))
        })?;

    Ok(res.to_string())
}

#[handler]
pub async fn did_from_signing_key(
    req: &mut Request,
    _res: &mut Response,
) -> Result<String, salvo::Error> {
    let params: Params = req.extract().await?;
    let pool = get_pg_pool();
    let res = DidRead::fetch_by_signing_key(&pool, params)
        .await
        .map(|(records, next_page)| {
            serde_json::json!({
                "records": records,
                "next_page": next_page
            })
        })
        .map_err(|e| {
            log::warn!("fetch from signing key error: {}", e);
            salvo::Error::Io(std::io::Error::other(
                "Failed to fetch did from signing key",
            ))
        })?;

    Ok(res.to_string())
}

#[handler]
pub async fn did_from_handle(
    req: &mut Request,
    _res: &mut Response,
) -> Result<String, salvo::Error> {
    let params: Params = req.extract().await?;
    let pool = get_pg_pool();
    let res = DidRead::fetch_by_handle(&pool, params)
        .await
        .map(|(records, next_page)| {
            serde_json::json!({
                "records": records,
                "next_page": next_page
            })
        })
        .map_err(|e| {
            log::warn!("fetch from handle error: {}", e);
            salvo::Error::Io(std::io::Error::other("Failed to fetch did from handle"))
        })?;

    Ok(res.to_string())
}

#[handler]
pub async fn did_from_lock_script_hash(
    req: &mut Request,
    _res: &mut Response,
) -> Result<String, salvo::Error> {
    let params: Params = req.extract().await?;
    let pool = get_pg_pool();
    let res = DidRead::fetch_by_lock_script_hash(&pool, params)
        .await
        .map(|(records, next_page)| {
            serde_json::json!({
                "records": records,
                "next_page": next_page
            })
        })
        .map_err(|e| {
            log::warn!("fetch from lock_script_hash error: {}", e);
            salvo::Error::Io(std::io::Error::other(
                "Failed to fetch did from lock_script_hash",
            ))
        })?;

    Ok(res.to_string())
}
