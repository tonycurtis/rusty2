use shmem;
use uname::uname;

fn main() {
    let node = uname().unwrap().nodename;

    shmem::init();

    let me = shmem::my_pe();
    let n = shmem::n_pes();

    println!("{}: PE {} of {}", node, me, n);

    shmem::finalize();
}
