use std::thread::{spawn, JoinHandle};
use std::thread::{Result as ThreadResult };
use std::convert::Into;

use super::fuse::FuseFlag;


/// A `StopThreadFuse` combines a fuse and a `thread::JoinHandle`.
///
/// When calling `join` on this struct the contained fuse is burned
/// befor calling the `JoinHandle`'s join. The burining of the fuse
/// _might_ signal the thread that this thread seams to be waiting for
/// it, so that it can terminate softly.
pub struct StopThreadFuse<T> {
    fuse: FuseFlag,
    join_handle: JoinHandle<T>
}


impl<T> StopThreadFuse<T> {

    /// create a new StopThreadFuse from a fuse and a JoinHandle
    pub fn new(fuse: FuseFlag, handle: JoinHandle<T>) -> StopThreadFuse<T> {
        StopThreadFuse {
            fuse: fuse,
            join_handle: handle
        }
    }

    /// burns the fuse which is used to signal the thread to stop
    pub fn request_stop(&self) {
        self.fuse.burn()
    }

    /// burns the fuse and then waits for the thread to terminate
    /// the returned result is the JoinHandle's result.
    pub fn stop_and_join(self) -> ThreadResult<T> {
        self.request_stop();
        self.join_handle.join()
    }
}

impl<T> Into<(FuseFlag, JoinHandle<T>)> for StopThreadFuse<T> {

    /// usable to destruct the StopThreadFuse into it's part
    fn into(self) -> (FuseFlag, JoinHandle<T>) {
        (self.fuse, self.join_handle)
    }
}


/// spawns a closure with `thread::spawn` but passes
/// a fuse into this closure, which is also returned
/// with the `JoinHandle` in form of a `StopThreadFuse`.
///
/// The thread is expected (but not forced) to check the
/// fuse from time to time and stop/terminated itself when
/// the fuse is burnt.
///
/// # Example
///
/// ```rust
/// # use fuseflag::fused_spawn;
/// # use std::thread::sleep;
/// # use std::time::Duration;
/// //some background job
/// let guard = fused_spawn(move |req_end_fuse| {
///     while req_end_fuse.check() {
///         //do work
///         sleep(Duration::from_millis(10));
///     }
/// });
///
/// //some work here
/// sleep(Duration::from_millis(100));
/// //ok, now (softly) stop the background job
/// guard.stop_and_join().expect("the background thread not to fail");
/// ```
///
pub fn fused_spawn<F, R>(func: F) -> StopThreadFuse<R>
    where F: Send + 'static + FnOnce(FuseFlag) -> R,
          R: Send + 'static
{
    let (ret_fuse, th_fuse) = FuseFlag::new_pair();
    let guard = spawn(move || {
        func(th_fuse)
    });

    StopThreadFuse::new(ret_fuse, guard)
}


#[cfg(test)]
mod tests {
    use std::thread::yield_now;
    use std::time::Instant;
    use super::fused_spawn;

    #[test]
    fn test_fused_spawn() {
        let guard = fused_spawn(move |req_end_fuse| {
            let then = Instant::now();
            while req_end_fuse.check() {
                yield_now();
                if then.elapsed().as_secs() > 1 {
                    return false;
                }
            }
            true
        });

        let had_no_timeout = guard.stop_and_join().expect("the waiting thread not to fail");
        assert!(had_no_timeout);

    }
}
