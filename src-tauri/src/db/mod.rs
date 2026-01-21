//! Database operations and connection management.
//!
//! This module handles SQLite database initialization, connection pooling,
//! schema creation, and CRUD operations for conversation data.

pub mod sqlite;

pub use sqlite::{Database, DbError, DbResult, init_db};
