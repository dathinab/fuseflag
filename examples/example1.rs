extern crate fuseflag;

use std::time::Duration;
use std::thread::{sleep, spawn, JoinHandle};

use fuseflag::FuseFlag;

fn background_job(end_flag: FuseFlag, id: u64) -> JoinHandle<()> {
    spawn(move || {
        while end_flag.check() {
            println!("\tbg-job({}): continues workin", id);
            //Background work Work
            sleep(Duration::from_millis(430*id));
        }
        println!("\tbg-job({}) stops running", id);
    })
}

fn main() {
    let end_fuse = FuseFlag::new();

    let bg_guard = vec![
        background_job(end_fuse.clone(), 1),
        background_job(end_fuse.clone(), 2)
    ];


    println!("running fake ui loop...");
    //naturally a very nice ui called sleep ;)
    sleep(Duration::from_secs(3));
    println!("main loop ended: waiting for background jobs to stop");

    //this will trigger .check() to return false (for ever from now on)
    end_fuse.burn();

    //join the threads
    for guard in bg_guard {
        guard.join().unwrap();
    }
}