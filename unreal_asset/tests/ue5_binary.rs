use std::io::Cursor;

use unreal_asset::{engine_version::EngineVersion, Asset, Error};

mod shared;

macro_rules! assets_folder {
    () => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/assets/ue5/")
    };
}

const TEST_ASSETS: [&[u8]; 1] = [include_bytes!(concat!(
    assets_folder!(),
    "BP_Pickup_Rifle.uasset"
))];

#[test]
fn ue5_4() -> Result<(), Error> {
    for asset_data in TEST_ASSETS {
        let parsed = Asset::new(
            Cursor::new(asset_data),
            None,
            EngineVersion::VER_UE5_4,
            None,
        )?;
        shared::verify_all_exports_parsed(&parsed);
    }

    Ok(())
}
