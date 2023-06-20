use crate::utilities::print_util;

use std::{thread, time};
use std::sync::Mutex;

// Thread construct only needs to occur within scope of function multithreading, no mutex backpack.
