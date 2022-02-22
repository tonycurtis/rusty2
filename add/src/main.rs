use shmem::*;
use uname::uname;

fn main() {
    let node = uname().unwrap().nodename;

    init();

    let me = my_pe();
    let n = n_pes();

    let mut dest = SymmMem::<i32>::new(1);

    dest.set(0, 6);

    barrier_all();

    if me == 0 {
        dest.atomic_add(4, n - 1);
    }

    barrier_all();

    println!("{}: PE {:>6} dest = {:>6}", node, me, dest.get(0));

    finalize();
}
