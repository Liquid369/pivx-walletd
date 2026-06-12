# pivx-walletd

Stateless PIVX Sapling (SHIELD) address derivation service, intended for
watch-only payment processing.

Given a Sapling extended full viewing key (`pxviews1...`) it derives diversified
shield payment addresses (`ps1...`) at requested diversifier indexes. It holds
no spending keys, no wallet and no database, and it never talks to the chain.
Payment detection stays in pivxd, which recognizes notes sent to any diversified
address of an imported viewing key (`importsaplingviewingkey`).

The crypto comes from [librustpivx](https://github.com/Duddino/librustpivx),
the PIVX port of librustzcash used by
[PIVX-Labs/pivx-shield](https://github.com/PIVX-Labs/pivx-shield), so the
network constants (`ps`, `pxviews`, coin type 119) are the same ones
MyPIVXWallet runs on. Address derivation is plain key math and needs no zk
parameter files.

## API

`POST /derive`

```json
{ "fvk": "pxviews1...", "index": 7 }
```

```json
{ "address": "ps1...", "index": 9 }
```

Roughly half of all diversifier indexes are invalid, so derivation finds the
first valid index at or after the requested one and returns it. Callers should
persist `index + 1` as their next cursor.

Testnet keys (`pxviewtestsapling1...`) are detected automatically and produce
`ptestsapling1...` addresses.

`GET /health` returns `ok`.

## Running

```sh
cargo run --release
# or
PIVX_WALLETD_BIND=0.0.0.0:8333 ./pivx-walletd
```

`PIVX_WALLETD_BIND` defaults to `127.0.0.1:8333`.

## Verifying against pivxd

```sh
# export a viewing key from a wallet with shield support
pivx-cli exportsaplingviewingkey "ps1youraddress"

# derive index 0
curl -s -X POST localhost:8333/derive \
  -H 'Content-Type: application/json' \
  -d '{"fvk":"pxviews1...","index":0}'

# import the key on another node; the default address it reports
# must match the derived address above
pivx-cli importsaplingviewingkey "pxviews1..." no
```

## Security

A full viewing key reveals the complete incoming transaction history of that
key. It cannot spend, but treat it as confidential and run this service on an
internal network only.

## License

MIT
