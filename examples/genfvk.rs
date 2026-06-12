//! Dev helper: prints a deterministic test viewing key (DO NOT use with real funds).
//! Usage: cargo run --example genfvk [seed-byte]

use sapling::zip32::ExtendedSpendingKey;
use zcash_keys::encoding::encode_extended_full_viewing_key;

fn main() {
    let seed_byte: u8 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(7);
    let extsk = ExtendedSpendingKey::master(&[seed_byte; 32]);
    #[allow(deprecated)]
    let efvk = extsk.to_extended_full_viewing_key();
    println!("{}", encode_extended_full_viewing_key("pxviews", &efvk));
}
