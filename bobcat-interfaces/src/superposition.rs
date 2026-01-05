use crate::selectors;

selectors! {
    SEL_FEATURES = b"features()"
}

pub fn make_fn_features() -> [u8; 4] {
    SEL_FEATURES
}
