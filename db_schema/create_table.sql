create table did_documents (
    did text not null,
    handle text not null,
    signing_key text not null,
    ckb_address text not null,
    tx_hash text not null,
    block_number text not null,
    outpoint text not null primary key,
    did_document jsonb not null,
    cell_data text not null,
    lock_script_hash text not null,
    valid boolean default true,
    created_at TIMESTAMPTZ not null,
    consumed_tx text,
    consumed_at TIMESTAMPTZ
);

create index idx_did_documents_did on did_documents(did);
create index idx_did_documents_ckb_address on did_documents(ckb_address);
create index idx_did_documents_outpoint on did_documents(outpoint);
create index idx_did_documents_valid on did_documents(valid);
create index idx_did_documents_signing_key on did_documents(signing_key);
create index idx_did_documents_created_at on did_documents(created_at);
create index idx_did_documents_lock_script_hash on did_documents(lock_script_hash);


create table did_documents_testnet (
    did text not null,
    handle text not null,
    signing_key text not null,
    ckb_address text not null,
    tx_hash text not null,
    block_number text not null,
    outpoint text not null primary key,
    did_document jsonb not null,
    cell_data text not null,
    lock_script_hash text not null,
    valid boolean default true,
    created_at TIMESTAMPTZ not null,
    consumed_tx text,
    consumed_at TIMESTAMPTZ
);

create index idx_did_documents_testnet_did on did_documents_testnet(did);
create index idx_did_documents_testnet_ckb_address on did_documents_testnet(ckb_address);
create index idx_did_documents_testnet_outpoint on did_documents_testnet(outpoint);
create index idx_did_documents_testnet_valid on did_documents_testnet(valid);
create index idx_did_documents_testnet_signing_key on did_documents_testnet(signing_key);
create index idx_did_documents_testnet_created_at on did_documents_testnet(created_at);
create index idx_did_documents_testnet_lock_script_hash on did_documents_testnet(lock_script_hash);
