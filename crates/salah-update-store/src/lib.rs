//! Experimental app-private storage for authenticated civil-time rule packs.
//!
//! F2-TZ9-P2b.1 is a Unix filesystem adapter, separate from the portable
//! verifier and calculation engine. It archives immutable signed packages,
//! atomically replaces a small state record, and reverts an unconfirmed trial
//! on reopen while retaining the accepted sequence high-water mark.
//!
//! The root must be an app-private directory on a local filesystem whose
//! rename, advisory locks, and file/directory sync semantics are supported.
//! This is not protection against a privileged actor replacing that directory
//! or another process with the app's credentials modifying files outside this
//! API. Hardware/power-loss behavior and mobile packaging need separate review.
//!
//! Stored selection does not automatically replace a live runtime. A caller
//! can load and move the verified selection into `salah-engine`'s explicit
//! snapshot API. The application must exercise it before confirming a trial;
//! this crate neither calculates schedules nor performs that confirmation.

/// Whether this adapter has an implementation for the compilation target.
pub const STORAGE_PLATFORM_SUPPORTED: bool = cfg!(unix);

#[cfg(unix)]
mod repository;
#[cfg(unix)]
pub use repository::{
    RepositoryRecovery, RepositoryStatus, RulePackRepository, StoreError, StoredPackStatus,
};
