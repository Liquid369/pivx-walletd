# pivx-walletd

Stateless PIVX Sapling (SHIELD) address-derivation service for watch-only payment
processing.

Given a Sapling **extended full viewing key** (`pxviews1...`), it derives diversified
shield payment addresses (`ps1...`) at requested diversifier indexes. It holds no
keys capable of spending, no wallet, no database, and never touches the chain —
payment *detection* stays in `pivxd`, which auto-recognizes notes sent to any
diversified address of an imported viewing key (`importsaplingviewingkey`).

Built on [librustpivx](https://github.com/Duddino/librustpivx), the PIVX-rebased
librustzcash fork used by [PIVX-Labs/pivx-shield](https://github.com/PIVX-Labs/pivx-shield),
so all PIVX network constants (HRPs `ps`/`pxviews`, coin type 119) come from the
same battle-tested stack as MyPIVXWallet. Address derivation is pure key math —
no zk parameter files required.

## API

### `POST /derive`

```json
{ "fvk": "pxviews1...", "index": 7 }
```

Response:

```json
{ "address": "ps1...", "index": 9 }
```

Not every diversifier index is valid (roughly half are), so derivation finds the
first valid index **at or after** the requested one and returns it. Callers should
persist `index + 1` as their next cursor.

Testnet keys (`pxviewtestsapling1...`) are detected automatically and produce
`ptestsapling1...` addresses.

### `GET /health`

Returns `ok`.

## Running

```sh
cargo run --release
# or
PIVX_WALLETD_BIND=0.0.0.0:8333 ./pivx-walletd
```

`PIVX_WALLETD_BIND` defaults to `127.0.0.1:8333`.

## Verifying against pivxd

From any PIVX wallet with shield support:

```sh
# 1. On the source wallet, export a viewing key for one of its shield addresses
pivx-cli exportsaplingviewingkey "ps1youraddress"

# 2. Ask pivx-walletd for index 0
curl -s -X POST localhost:8333/derive \
  -H 'Content-Type: application/json' \
  -d '{"fvk":"pxviews1...","index":0}'

# 3. Import the key on a (watch-only) node; the returned default address
#    must equal the address from step 2
pivx-cli importsaplingviewingkey "pxviews1..." no
```

## Security notes

- A full viewing key reveals all incoming (and with the FVK, outgoing) transaction
  detail for that key — treat it as confidential, even though it cannot spend.
- Run on an internal/private network only (e.g. the docker network shared with
  your payment server). Do not expose publicly.

## License

MIT
