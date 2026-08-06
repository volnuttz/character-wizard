//! SRD 5.2.1 level-1 rule catalog.

#[path = "catalog.rs"]
mod srd_data_catalog;
#[path = "types.rs"]
mod srd_data_types;

pub use srd_data_catalog::*;
pub use srd_data_types::*;
