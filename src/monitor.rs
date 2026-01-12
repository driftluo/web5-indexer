use crate::{
    CKB_MAINNET_RPC, CKB_TESTNET_RPC, CellType, IndexerScriptSearchMode, Network, Order, RpcClient,
    ScriptType, SearchKey, SearchKeyFilter, Tx, calculate_address, calculate_web5_did,
    check_did_doc, did_script, parse_didoc_cell,
};

use chrono::DateTime;
use ckb_types::{packed, prelude::Entity};

use std::sync::Arc;

pub async fn did_monitor(rpc: &RpcClient) {
    let (tip_testnet, tip) = loop {
        let testnet_tip = rpc.get_indexer_tip(CKB_TESTNET_RPC.clone()).await;
        let mainnet_tip = rpc.get_indexer_tip(CKB_MAINNET_RPC.clone()).await;
        if let (Ok(tip_testnet), Ok(tip)) = (testnet_tip, mainnet_tip) {
            break (tip_testnet, tip);
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    };

    for net in [Network::Mainnet, Network::Testnet] {
        let last_number = match net {
            Network::Mainnet => crate::pg_write::global_cache().load(),
            Network::Testnet => crate::pg_write::global_cache_testnet().load(),
        };
        log::info!(
            "Starting DID monitor for {:?}, from {}, to {}",
            net,
            last_number.value(),
            match net {
                Network::Mainnet => tip.block_number.value(),
                Network::Testnet => tip_testnet.block_number.value(),
            }
        );
        let url = match net {
            Network::Mainnet => CKB_MAINNET_RPC.clone(),
            Network::Testnet => CKB_TESTNET_RPC.clone(),
        };

        let search_key = SearchKey {
            script: did_script(net, ckb_jsonrpc_types::JsonBytes::default()),
            script_type: ScriptType::Type,
            filter: Some(SearchKeyFilter::block_range(
                *last_number.as_ref(),
                match net {
                    Network::Mainnet => tip.block_number,
                    Network::Testnet => tip_testnet.block_number,
                },
            )),
            with_data: None,
            script_search_mode: Some(IndexerScriptSearchMode::Prefix),
            group_by_transaction: Some(true),
        };

        let mut raw_tx_with_cell = Vec::new();
        let mut after_cursor = None;
        loop {
            match rpc
                .get_transactions(
                    url.clone(),
                    search_key.clone(),
                    Order::Asc,
                    500.into(),
                    after_cursor.clone(),
                )
                .await
            {
                Ok(result) => {
                    let has_more = result.objects.len() == 500;
                    raw_tx_with_cell.extend(result.objects);
                    if !has_more {
                        break;
                    }
                    after_cursor = Some(result.last_cursor);
                }
                Err(e) => {
                    log::error!("Error fetching transactions: {:?}", e);
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            }
        }
        let mut dids = Vec::new();
        let mut did_deletes = Vec::new();
        for t in raw_tx_with_cell {
            if let Tx::Grouped(tx) = t {
                let tx_all = loop {
                    let tx = rpc.get_transaction(url.clone(), &tx.tx_hash).await;
                    if let Ok(tx) = tx {
                        break tx.unwrap();
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                };
                let header = loop {
                    let header = rpc.get_header_by_number(url.clone(), tx.block_number).await;
                    if let Ok(header) = header {
                        break header;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                };
                for (typ, index) in tx.cells {
                    match typ {
                        CellType::Output => {
                            let out_point =
                                packed::OutPoint::new(tx.tx_hash.clone().into(), index.value());

                            let lock = tx_all
                                .inner
                                .outputs
                                .get(index.value() as usize)
                                .unwrap()
                                .lock
                                .clone();
                            let ckb_addr = calculate_address(&lock.into(), net.into());
                            let cell_data = tx_all
                                .inner
                                .outputs_data
                                .get(index.value() as usize)
                                .unwrap();
                            let didoc = match parse_didoc_cell(cell_data.as_bytes()) {
                                Some(didoc) => didoc,
                                None => {
                                    log::warn!(
                                        "Failed to parse DIDoc cell data at tx: {}, index: {}",
                                        tx.tx_hash,
                                        index.value()
                                    );
                                    continue;
                                }
                            };

                            let (handle, signing_key) = match check_did_doc(&didoc) {
                                Some(handle) => handle,
                                None => {
                                    log::warn!(
                                        "DIDoc check failed at tx: {}, index: {}",
                                        tx.tx_hash,
                                        index.value()
                                    );
                                    continue;
                                }
                            };
                            let type_script = tx_all
                                .inner
                                .outputs
                                .get(index.value() as usize)
                                .unwrap()
                                .type_
                                .as_ref()
                                .unwrap();
                            let web5_did = calculate_web5_did(&type_script.args.as_bytes()[..20]);

                            // insert to db
                            dids.push(crate::pg_write::DidWrite::new(
                                web5_did,
                                handle,
                                signing_key,
                                ckb_addr.to_string(),
                                faster_hex::hex_string(tx.tx_hash.as_bytes()),
                                faster_hex::hex_string(&tx.block_number.value().to_be_bytes()),
                                faster_hex::hex_string(&out_point.as_bytes()),
                                sqlx::types::Json(didoc),
                                DateTime::from_timestamp_millis(
                                    header.inner.timestamp.value() as i64
                                )
                                .unwrap(),
                            ));
                        }
                        CellType::Input => {
                            let out_point: packed::OutPoint = tx_all
                                .inner
                                .inputs
                                .get(index.value() as usize)
                                .unwrap()
                                .previous_output
                                .clone()
                                .into();
                            // change state to invalid
                            did_deletes.push(crate::pg_write::DidDelete::new(
                                faster_hex::hex_string(&out_point.as_bytes()),
                            ));
                        }
                    }
                }
            }
        }
        if !dids.is_empty() {
            log::info!("{:?} Inserting {} DID entries", net, dids.len());
            let mut conn = crate::get_pg_pool().begin().await.unwrap();
            crate::pg_write::DidWrite::insert_batch(&mut conn, &dids, net)
                .await
                .expect("Failed to insert did writes");
            conn.commit().await.unwrap();
        }
        if !did_deletes.is_empty() {
            log::info!("{:?} Deleting {} DID entries", net, did_deletes.len());
            let mut conn = crate::get_pg_pool().begin().await.unwrap();
            crate::pg_write::DidDelete::delete_batch(&mut conn, &did_deletes, net)
                .await
                .expect("Failed to delete did deletes");
            conn.commit().await.unwrap();
        }
        match net {
            Network::Mainnet => {
                crate::pg_write::global_cache().store(Arc::new(tip.block_number));
            }
            Network::Testnet => {
                crate::pg_write::global_cache_testnet().store(Arc::new(tip_testnet.block_number));
            }
        }
        log::info!(
            "Finished processing DID cells for {:?} up to block number {:?}",
            net,
            match net {
                Network::Mainnet => tip.block_number.value(),
                Network::Testnet => tip_testnet.block_number.value(),
            }
        );
    }
}
