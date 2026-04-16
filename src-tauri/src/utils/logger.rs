// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// LOGGER
// Global diagnostic logging initialization and environment configuration.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

/// Initializes the global logger with a default filter level of 'info'.
pub fn init() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
}
