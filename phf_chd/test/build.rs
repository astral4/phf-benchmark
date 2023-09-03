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

const ENTRY_ONE: [(u8, u8); 1] = [(123, 45)];

const ENTRY_MULTIPLE: [(&'static str, u32); 4] =
    [("foo", 1234), ("bar", 5678), ("baz", 42424242), ("qux", 0)];

fn main() -> Result<(), Box<dyn Error>> {
    let path = Path::new(&var("OUT_DIR")?).join("codegen.rs");
    let mut file = BufWriter::new(File::create(&path)?);

    write!(
        &mut file,
        "const ENTRY_ONE: Map<u8, u8, AHasher> = {};",
        MapGenerator::<_, _, AHasher>::from(ENTRY_ONE.into_iter())
    )?;

    write!(
        &mut file,
        "const ENTRY_MULTIPLE: Map<&'static str, u32, AHasher> = {};",
        MapGenerator::<_, _, AHasher>::from(ENTRY_MULTIPLE.into_iter())
    )?;

    write!(
        &mut file,
        "const ENTRY_MANY: Map<u32, u32, AHasher> = {};",
        MapGenerator::<u32, u32, AHasher>::from(
            SmallRng::seed_from_u64(FIXED_SEED)
                .sample_iter(Standard)
                .take(10000)
        )
    )?;

    Ok(())
}
