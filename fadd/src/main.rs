use shmem::*;
use uname::uname;

fn main() {
    let node = uname().unwrap().nodename;

    init();

    let me = my_pe();

    let mut dest = SymmMem::<i32>::new(1);
    *dest = 22;
    barrier_all();

    let mut old = -1;
    if me == 1 {
        old = dest.atomic_fetch_add(44, 0);
    }

    barrier_all();

    println!(
        "{}: PE {:>6} old = {:>6} dest = {:>6}",
        node, me, old, *dest
    );

    finalize();
}
