/// A (stupid??) example for using fuseflag
///
/// This will run two threads on wich does "fake" loads,
/// and one which reads from stdin. The background loop
/// has to fuses, one which is burn when the user enters
/// "fast", which will change the bg-jobs behaviour. The
/// other is implicitly burnd when the user presses "j" (for exit)
/// and the main thread waits/joins the background thread.
/// This fuse is also implicitly passed to the bg-job and is
/// used to signal it to stop.fa
extern crate fuseflag;

use std::thread::{sleep};
use std::time::Duration;
use std::io;
use std::io::prelude::*;

use fuseflag::{FuseFlag, StopThreadFuse, fused_spawn};

fn background_job(not_crazy: FuseFlag) -> StopThreadFuse<()> {
    fused_spawn(move |do_continue| {
        while do_continue.check() && not_crazy.check() {
            // do some work, e.g. update sql database
            sleep(Duration::from_secs(2));
            println!("another workload done...")
        };
        while do_continue.check() {
            sleep(Duration::from_millis(600));
            println!("..so much work...");
            println!("another workload done...")
        }
    })
}

fn main_loop(speed_fuse: FuseFlag) {
    let stdin = io::stdin();
    print!("\nexit [j/n]? ");
    for line in stdin.lock().lines() {
        match line {
            Ok(text) => {
                println!("you said: {}", &*text);
                if text == "j" {
                    break;
                } else if text == "fast" {
                    println!("nooo! It might be faster?!?");
                    speed_fuse.burn();
                }
            },
            Err(err) => {
                println!("ups: {:?}", err);
                break;
            }
        }
        print!("\nexit [j/n]? ");
    }
}

fn main() {
    let fuse = FuseFlag::new();
    let bg_guard = background_job(fuse.clone());

    main_loop(fuse);

    println!("Waiting for background job to exist");
    bg_guard.stop_and_join().expect("no panic in background job");
    println!("Good by ;-)");
}