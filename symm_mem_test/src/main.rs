use shmem::*;
use uname::uname;

fn main() {
    let node = uname().unwrap().nodename;

    init();

    let me = my_pe();
    let _n = n_pes();

    let mut dest = SymmMem::<i32>::new(1);

    *dest = 5;

    println!("{}: PE {:>6} dest = {:>6}", node, me, *dest);

    drop(dest);

    shmem::finalize();
}
