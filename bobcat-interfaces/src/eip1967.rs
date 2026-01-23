use bobcat_maths::U;

use bobcat_cd::const_keccak256;

/// A topic0 that the implementation was upgraded.
pub const TOPIC_UPGRADED: U = const_keccak256(b"Upgraded(address)");

/// A topic0 that the admin was changed.
pub const TOPIC_ADMIN_CHANGED: U = const_keccak256(b"adminChanged(address,address)");
