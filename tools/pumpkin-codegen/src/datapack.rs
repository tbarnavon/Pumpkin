//! Where the generators read vanilla data from.
//!
//! The vanilla datapack is extracted from the server jar (`pumpkin-data`'s build script). On the
//! 1.21.1 branch, `assets/codegen_extra` adds the registries 1.21.1 hardcodes in Java instead of
//! shipping as data (mob variants, trades, trial spawners, …), in the later versions' format the
//! shared code reads.

use std::path::{Path, PathBuf};

/// The extracted vanilla datapack's `data` folder.
pub const VANILLA_DATA: &str = "../../assets/datapack/data";

/// The data this branch keeps for registries the vanilla datapack doesn't have.
pub const EXTRA_DATA: &str = "../../assets/codegen_extra/data";

/// `data/minecraft/<rel>`: from [`EXTRA_DATA`] when the branch keeps it there, otherwise from the
/// vanilla datapack.
#[must_use]
pub fn mc(rel: impl AsRef<Path>) -> PathBuf {
    let extra = Path::new(EXTRA_DATA).join("minecraft").join(rel.as_ref());
    if extra.exists() {
        extra
    } else {
        Path::new(VANILLA_DATA).join("minecraft").join(rel.as_ref())
    }
}
