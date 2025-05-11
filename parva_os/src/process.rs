// Importing types from the `alloc` crate:
// - `BTreeMap`: an ordered key-value map.
// - `String`, `ToString`: heap-allocated strings and conversion trait.
use alloc::{collections::BTreeMap, string::{String, ToString}};

// Importing atomic types from the `core` crate:
// - `AtomicUsize`: a thread-safe integer used for unique IDs.
// - `Ordering`: memory ordering used when performing atomic operations.
use core::sync::atomic::{AtomicUsize, Ordering};

// Importing the `lazy_static!` macro which allows us to define
// lazily-initialized static variables in a safe and convenient way.
use lazy_static::lazy_static;

// Importing `Mutex` from the `spin` crate.
// This is a simple, fast mutual exclusion primitive used to ensure thread-safe access
// to shared data without depending on an operating system.
use spin::Mutex;

// Define two static variables using lazy_static!
// These are global, shared across threads, and initialized only once at runtime.
lazy_static! {
    // A global atomic counter for generating unique process IDs.
    pub static ref PIDS: AtomicUsize = AtomicUsize::new(0);

    // A global Process instance protected by a mutex to ensure thread-safe access.
    // It is initialized with the directory "/" and a unique ID.
    pub static ref PROCESS: Mutex<Process> = Mutex::new(Process::new("/"));
}

// The `Process` struct represents a simplified process model.
pub struct Process {
    id: usize,                         // Unique process ID.
    env: BTreeMap<String, String>,     // Map of environment variables (key-value pairs).
    dir: String                        // Current working directory of the process.
}

impl Process {
    // Constructor method for creating a new `Process`.
    pub fn new(dir: &str) -> Self {
        // Atomically fetch and increment the global process ID counter.
        let id = PIDS.fetch_add(1, Ordering::SeqCst);

        // Initialize an empty environment variable map.
        let env = BTreeMap::new();

        // Convert the input directory to a `String`.
        let dir = dir.to_string();

        // Return a new Process instance with these initialized values.
        Self { id, env, dir }
    }
}

// Retrieve the ID of the current process.
pub fn id() -> usize {
    // Lock the PROCESS mutex to safely access the `id` field.
    PROCESS.lock().id
}

// Retrieve the value of a specific environment variable, if it exists.
pub fn env(key: &str) -> Option<String> {
    // Lock the PROCESS mutex and attempt to get the value associated with `key`.
    match PROCESS.lock().env.get(key.into()) {
        Some(val) => Some(val.clone()), // If found, return a clone of the value.
        None => None,                   // Otherwise, return None.
    }
}

// Retrieve a copy of all environment variables in the current process.
pub fn envs() -> BTreeMap<String, String> {
    // Lock the PROCESS mutex and clone the entire environment map.
    PROCESS.lock().env.clone()
}

// Retrieve the current working directory of the process.
pub fn dir() -> String {
    // Lock the PROCESS mutex and return a clone of the directory string.
    PROCESS.lock().dir.clone()
}

// Set or update an environment variable with a new key-value pair.
pub fn set_env(key: &str, val: &str) {
    // Lock the PROCESS mutex and insert the key-value pair into the environment map.
    PROCESS.lock().env.insert(key.into(), val.into());
}

// Set the current working directory of the process.
pub fn set_dir(dir: &str) {
    // Lock the PROCESS mutex and update the `dir` field.
    PROCESS.lock().dir = dir.into();
}
