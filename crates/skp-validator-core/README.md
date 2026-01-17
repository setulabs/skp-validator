# skp-validator-core

Core engine and traits for `skp-validator` - the most advanced, flexible and modular validation library for Rust.

## Purpose

This crate provides the foundational traits (`Validate`, `Rule`) and types (`ValidationErrors`, `ValidationError`, `FieldPath`) that power the `skp-validator` ecosystem.

## Integration

If you are building a custom validation rule or an integration for a new framework, you should depend on this crate. For general usage, please see the [skp-validator](https://crates.io/crates/skp-validator) crate.
