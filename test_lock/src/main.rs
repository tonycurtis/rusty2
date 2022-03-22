use shmem::*;
use uname::uname;
extern crate getopts;
use getopts::Options;
use std::env;

fn main() {
    let node = uname().unwrap().nodename;
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optflag("t", "test", "Use test_lock instead of set_lock");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(e) => { panic!("{}", e.to_string()) }
    };

    init();

    let me = my_pe();

    let mut count = SymmMem::<i32>::new(1);

    if me == 0 {
        eprintln!("Using dynamic lock");
    }

    let mut lock = SymmMem::<i64>::new(1);
    *lock = 0;

    barrier_all();

    if matches.opt_present("t") {
        if me == 0 { eprintln!("Using test lock"); }
        while test_lock(&lock) {};
    }
    else {
        if me == 0 { eprintln!("Using set lock"); }
        set_lock(&lock);
    }

    let val = count.g(0);

    println!("{:>8}: {:>4}: count is {}", node, me, val);

    count.p(val + 1, 0);

    clear_lock(&lock);

    finalize();
}
