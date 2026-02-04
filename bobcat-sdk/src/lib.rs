#![no_std]

pub mod prelude {
    pub use super::{
        call::*, cd::*, create::*, entry::*, events::*, features::*, host, maths::*, proxy::*,
        storage::*,
    };

    pub use super::interfaces;
    pub use super::precompiles;

    #[cfg(any(feature = "panic", feature = "panic-revert"))]
    pub use super::panic::*;

    #[cfg(feature = "console")]
    pub use super::console::*;
}

pub use bobcat_call as call;
pub use bobcat_cd as cd;
pub use bobcat_create as create;
pub use bobcat_entry as entry;
pub use bobcat_events as events;
pub use bobcat_features as features;
pub use bobcat_host as host;
pub use bobcat_interfaces as interfaces;
pub use bobcat_maths as maths;
pub use bobcat_precompiles as precompiles;
pub use bobcat_proxy as proxy;
pub use bobcat_storage as storage;

#[cfg(any(feature = "panic", feature = "panic-revert"))]
pub use bobcat_panic as panic;

#[cfg(feature = "console")]
pub use bobcat_console as console;
