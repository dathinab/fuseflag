use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;


/// Represents a (not resettable) fuse/flag which can be shared between different threads.
///
///
/// # Note
/// given that `FuseFlag` is only meant for a lose coupling between threads
/// (e.g. to give a thread a signal to stop doing it's background job "sometime"
/// in the future) it is not necessary suited as a tool for synchronisation and
/// uses the relaxed memory ordering.
#[derive(Clone, Debug)]
pub struct FuseFlag(Arc<AtomicBool>);

impl FuseFlag {
    /// creates a new FuseFlag
    pub fn new() -> FuseFlag {
        FuseFlag(Arc::new(AtomicBool::new(true)))
    }

    /// creates a pair of a Check and a Burn handler for a FuseFlag
    pub fn new_pair() -> (FuseFlag, FuseFlag) {
        let flag = FuseFlag::new();
        (flag.clone(), flag)
    }


    /// returns true as long as the fuse was not burnt
    pub fn check(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }


    /// burns the fuse, from then check will return false
    pub fn burn(&self) {
        self.0.store(false, Ordering::Relaxed)
    }

}

impl Default for FuseFlag {
    fn default() -> FuseFlag { FuseFlag::new() }
}



#[cfg(test)]
mod tests {

    use std::time::Instant;
    use std::thread::{spawn, yield_now, JoinHandle};
    use super::FuseFlag;

    fn not(bl: bool) -> bool { !bl }

    //NOTE: st_ is for single threaded test
    #[test]
    fn has_default() {
        assert!(FuseFlag::default().check());
    }

    #[test]
    fn single_thread_check_burn_check() {
        let fuse = FuseFlag::new();
        assert!(fuse.check());
        fuse.burn();
        assert!(not(fuse.check()));
    }

    #[test]
    fn single_thread_burn_burn() {
        let fuse = FuseFlag::new();
        fuse.burn();
        fuse.burn();
        assert!(not(fuse.check()))
    }

    #[test]
    fn multi_thread_check() {
        let fuse = FuseFlag::new();
        let th_fuse = fuse.clone();

        let ok = spawn(move || {
            th_fuse.check()
        }).join().unwrap();

        assert!(ok);
        assert!(fuse.check());
    }


    fn wait_for_fuse_burn(fuse_check: FuseFlag) -> JoinHandle<bool> {
        assert!(fuse_check.check());
        spawn(move || {
            let then = Instant::now();
            while fuse_check.check() {
                yield_now();
                if then.elapsed().as_secs() > 1 {
                    return false;
                }
            }
            true
        })
    }

    #[test]
    fn multi_thread_burn() {
        let fuse = FuseFlag::new();

        let guard = wait_for_fuse_burn(fuse.clone());

        fuse.burn();

        assert!(not(fuse.check()));
        let had_no_timeout = guard.join().expect("no panic in waiting thread");
        assert!(had_no_timeout);
    }

    #[test]
    fn multi_thread_pair() {
        let (fuse_check, fuse) = FuseFlag::new_pair();

        let guard = wait_for_fuse_burn(fuse_check);

        fuse.burn();

        let had_no_timeout = guard.join().expect("no panic in waiting thread");
        assert!(had_no_timeout);
    }

}

