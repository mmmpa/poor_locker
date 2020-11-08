use serde::{Deserialize, Serialize};
use std::sync::Arc;
use wrapped_string_type_macro::prelude::*;

define_arc_wrapped_string_type!(LockKey);
