use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use zcash_keys::encoding::{decode_extended_full_viewing_key, encode_payment_address};
use zip32::DiversifierIndex;

// PIVX bech32 HRPs (PIVX Core src/chainparams.cpp)
const HRP_FVK_MAIN: &str = "pxviews";
const HRP_ADDR_MAIN: &str = "ps";
const HRP_FVK_TEST: &str = "pxviewtestsapling";
const HRP_ADDR_TEST: &str = "ptestsapling";

#[derive(Deserialize)]
struct DeriveRequest {
    /// Sapling extended full viewing key (pxviews1... / pxviewtestsapling1...)
    fvk: String,
    /// Diversifier index to start the search from.
    index: u64,
}

#[derive(Serialize)]
struct DeriveResponse {
    /// Diversified shield payment address (ps1... / ptestsapling1...)
    address: String,
    /// Diversifier index actually used (first valid index >= requested).
    /// Callers should persist `index + 1` as the next cursor.
    index: u64,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

fn err(status: StatusCode, msg: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (status, Json(ErrorResponse { error: msg.into() }))
}

fn index_to_u64(j: DiversifierIndex) -> Result<u64, &'static str> {
    let bytes = j.as_bytes(); // 11 bytes, little-endian
    if bytes[8..].iter().any(|b| *b != 0) {
        return Err("diversifier index exceeds u64");
    }
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&bytes[..8]);
    Ok(u64::from_le_bytes(buf))
}

fn derive_address(fvk: &str, index: u64) -> Result<(String, u64), String> {
    let (hrp_fvk, hrp_addr) = if fvk.starts_with(HRP_FVK_TEST) {
        (HRP_FVK_TEST, HRP_ADDR_TEST)
    } else {
        (HRP_FVK_MAIN, HRP_ADDR_MAIN)
    };

    let efvk = decode_extended_full_viewing_key(hrp_fvk, fvk)
        .map_err(|e| format!("invalid viewing key: {e}"))?;
    let dfvk = efvk.to_diversifiable_full_viewing_key();

    let (j, addr) = dfvk
        .find_address(DiversifierIndex::from(index))
        .ok_or_else(|| "no valid diversifier at or after requested index".to_string())?;

    let used = index_to_u64(j).map_err(|e| e.to_string())?;
    Ok((encode_payment_address(hrp_addr, &addr), used))
}

async fn derive(
    Json(req): Json<DeriveRequest>,
) -> Result<Json<DeriveResponse>, (StatusCode, Json<ErrorResponse>)> {
    let (address, index) =
        derive_address(&req.fvk, req.index).map_err(|e| err(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(DeriveResponse { address, index }))
}

async fn health() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let bind = std::env::var("PIVX_WALLETD_BIND").unwrap_or_else(|_| "127.0.0.1:8333".into());
    let app = Router::new()
        .route("/derive", post(derive))
        .route("/health", get(health));

    tracing::info!("pivx-walletd listening on {bind}");
    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .expect("failed to bind");
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
        })
        .await
        .expect("server error");
}

#[cfg(test)]
mod tests {
    use super::*;
    use sapling::zip32::ExtendedSpendingKey;
    use zcash_keys::encoding::encode_extended_full_viewing_key;

    /// Deriving from the FVK must agree with the spending key's own
    /// default address (the first valid diversifier).
    #[test]
    fn derive_index_zero_matches_default_address() {
        let extsk = ExtendedSpendingKey::master(&[7u8; 32]);
        let dfvk = extsk.to_diversifiable_full_viewing_key();
        let (default_j, default_addr) = dfvk.default_address();

        #[allow(deprecated)]
        let efvk = extsk.to_extended_full_viewing_key();
        let fvk_str = encode_extended_full_viewing_key(HRP_FVK_MAIN, &efvk);

        let (addr, used) = derive_address(&fvk_str, 0).expect("derivation failed");
        assert_eq!(addr, encode_payment_address(HRP_ADDR_MAIN, &default_addr));
        assert_eq!(used, index_to_u64(default_j).unwrap());
        assert!(addr.starts_with(HRP_ADDR_MAIN));
    }

    /// Distinct diversifier indexes must yield distinct addresses,
    /// and re-deriving the same index must be deterministic.
    #[test]
    fn derivation_is_deterministic_and_unique() {
        let extsk = ExtendedSpendingKey::master(&[42u8; 32]);
        #[allow(deprecated)]
        let efvk = extsk.to_extended_full_viewing_key();
        let fvk_str = encode_extended_full_viewing_key(HRP_FVK_MAIN, &efvk);

        let (a0, j0) = derive_address(&fvk_str, 0).unwrap();
        let (a0_again, j0_again) = derive_address(&fvk_str, 0).unwrap();
        assert_eq!((&a0, j0), (&a0_again, j0_again));

        let (a1, j1) = derive_address(&fvk_str, j0 + 1).unwrap();
        assert_ne!(a0, a1);
        assert!(j1 > j0);
    }

    #[test]
    fn rejects_garbage_keys() {
        assert!(derive_address("pxviews1notakey", 0).is_err());
        assert!(derive_address("zxviews1wrongchain", 0).is_err());
    }
}
