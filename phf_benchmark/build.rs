use phf_chd::MapGenerator;
use phf_shared::hash::AHasher;
use phf_shared::FIXED_SEED;
use rand::distributions::Standard;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::env::var;
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let path = Path::new(&var("OUT_DIR")?).join("codegen.rs");
    let mut file = BufWriter::new(File::create(&path)?);

    write!(
        &mut file,
        "pub const MAP_CHD_1K: Map<u64, u64, AHasher> = {};",
        MapGenerator::<u64, u64, AHasher>::from(
            SmallRng::seed_from_u64(FIXED_SEED)
                .sample_iter(Standard)
                .take(1000)
        )
    )?;

    write!(
        &mut file,
        "pub const MAP_CHD_10K: Map<u64, u64, AHasher> = {};",
        MapGenerator::<u64, u64, AHasher>::from(
            SmallRng::seed_from_u64(FIXED_SEED)
                .sample_iter(Standard)
                .take(10000)
        )
    )?;

    Ok(())
}
