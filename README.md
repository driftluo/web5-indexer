## Web5-indexer

http api method list:

```
/did_from_id？page=0&page_size=10&did=...
/did_from_address?page=0&page_size=10&address=...
```

All apis that include paging functions have a page_size parameter. The default is 500, and the maximum is 500. It can be adjusted by passing parameters.
All APIs have a parameter called net, which can be testnet or mainnet. The default is mainnet.
