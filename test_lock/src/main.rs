use shmem::{SymmMemTrait};

use uname::uname;
use clap::{Arg, Command};

fn main() {
    let node = uname().unwrap().nodename;

    let matches = Command::new("Test Lock")
        .arg(Arg::new("test")
             .short('t')
             .long("test")
        )
        .arg(Arg::new("set")
             .short('s')
             .long("set")
        )
        .get_matches();

    let use_test = matches.is_present("test");

    shmem::init();

    let me = shmem::my_pe();

    let mut count = shmem::SymmMem::<i32>::new(1);

    if me == 0 {
        eprintln!("Using dynamic lock");
    }

    let mut lock = shmem::SymmMem::<i64>::new(1);
    *lock = 0;

    shmem::barrier_all();

    if use_test {
        if me == 0 { eprintln!("Using test lock"); }
        while shmem::test_lock(&lock) {};
    }
    else {
        if me == 0 { eprintln!("Using set lock"); }
        shmem::set_lock(&lock);
    }

    let val = count.g(0);

    println!("{:>8}: {:>4}: count is {}", node, me, val);

    count.p(val + 1, 0);

    shmem::clear_lock(&lock);

    shmem::finalize();
}
