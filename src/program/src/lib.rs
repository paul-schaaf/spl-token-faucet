pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

solana_program::declare_id!("4bXpkKSV8swHSnwqtzuboGPaPDeEgAn4Vt8GfarV5rZt");

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;
