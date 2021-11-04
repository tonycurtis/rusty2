use shmem;
use std::mem;
use uname::uname;

fn main() {
    let node = uname().unwrap().nodename;

    shmem::init();

    let me = shmem::my_pe();

    let mut dest = shmem::SymmMem::<i32>::new(1);
    *dest = 22;
    shmem::barrier_all();

    let mut old = -1;
    if me == 1 {
        old = shmem::int_atomic_fetch_add(&dest, 44, 0);
    }

    shmem::barrier_all();

    println!(
        "{}: PE {:>6} old = {:>6} dest = {:>6}",
        node, me, old, *dest
    );

    shmem::finalize();
}
