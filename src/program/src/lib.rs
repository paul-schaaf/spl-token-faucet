pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

solana_program::declare_id!("8f5EGAKk9pabn9c9apZNLh7qRHoZpsF21WWjDxDMGYT4");

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;
