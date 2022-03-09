use shmem::*;
use uname::uname;

fn main() {
    let node = uname().unwrap().nodename;

    init();

    let me = my_pe();
    let n = n_pes();

    println!("{}: PE {} of {}", node, me, n);

    finalize();
}
