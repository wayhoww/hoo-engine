mod basic_types;
mod compound_types;
mod objects;
mod tests;
mod traits;
mod types;

pub use basic_types::*;
pub use compound_types::*;
pub use objects::*;
pub use traits::*;
pub use types::*;

pub fn initialize(flags: &str) {
    use std::sync::Once;

    static INIT_FLAG: Once = Once::new();

    INIT_FLAG.call_once(|| {
        v8::V8::set_flags_from_string(flags);
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
    });
}
