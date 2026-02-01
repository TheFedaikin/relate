//! Define-both syntax (defining two structs inline) is not supported.
//!
//! This syntax was removed for simplicity. Define structs separately and
//! use the standard relation syntax instead:
//!
//! ```ignore
//! #[derive(Debug, Clone)]
//! struct Source { id: i32 }
//!
//! #[derive(Debug, Clone)]
//! struct Target { id: i32 }
//!
//! relate_structs! {
//!     Source ~ Target { id; }
//! }
//! ```

use relate::relate_structs;

relate_structs! {
    #[derive(Debug, Clone)]
    struct Source
    ~
    #[derive(Debug, Clone)]
    struct Target
    {
        id: i32,
    }
}

fn main() {}
