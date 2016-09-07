//! fuseflag provides a non resettable fuse/flag shared between threads
//!
//! # Example
//! ```rust
//! # use std::time::Duration;
//! # use std::thread::{sleep, spawn, yield_now, JoinHandle};
//! # use fuseflag::FuseFlag;
//!
//! // some background job
//! fn background_job() -> (FuseFlag, JoinHandle<()>) {
//!     let (fuse, th_fuse) = FuseFlag::new_pair();
//!     let guard = spawn(move || {
//!         while th_fuse.check() {
//!             // do some infinite background job
//!             sleep(Duration::from_millis(10));
//!         }
//!     });
//!     (fuse, guard)
//! }
//!
//! fn main_loop() {
//!     // some use input, gui loop etc.
//!     sleep(Duration::from_millis(100));
//! }
//!
//! fn main() {
//!     let (stop_bg_fuse, bg_guard) = background_job();
//!     main_loop();
//!     // we are about to exit the program, so stop the bg-job
//!     stop_bg_fuse.burn();
//!     bg_guard.join();
//! }
//! ```
//!

//reexport actual api
pub use self::fused_spawn::{StopThreadFuse, fused_spawn};
pub use self::fuse::FuseFlag;

mod fuse;
mod fused_spawn;

