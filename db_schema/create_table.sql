create table did_documents (
    did text not null primary key,
    handle text not null,
    signing_key text not null,
    ckb_address text not null,
    tx_hash text not null,
    block_number text not null,
    outpoint text not null,
    did_document jsonb not null,
    valid boolean default true,
    created_at TIMESTAMPTZ not null
);

create index idx_did_documents_did on did_documents(did);
create index idx_did_documents_ckb_address on did_documents(ckb_address);
create index idx_did_documents_outpoint on did_documents(outpoint);
create index idx_did_documents_valid on did_documents(valid);


create table did_documents_testnet (
    did text not null primary key,
    handle text not null,
    signing_key text not null,
    ckb_address text not null,
    tx_hash text not null,
    block_number text not null,
    outpoint text not null,
    did_document jsonb not null,
    valid boolean default true,
    created_at TIMESTAMPTZ not null
);

create index idx_did_documents_testnet_did on did_documents_testnet(did);
create index idx_did_documents_testnet_ckb_address on did_documents_testnet(ckb_address);
create index idx_did_documents_testnet_outpoint on did_documents_testnet(outpoint);
create index idx_did_documents_testnet_valid on did_documents_testnet(valid);
