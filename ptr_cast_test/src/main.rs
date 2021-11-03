use shmem;
use std::mem;
use uname::uname;

fn main() {
    let node = uname().unwrap().nodename;

    shmem::init();

    let me = shmem::my_pe();
    let n = shmem::n_pes();

    let ptr = shmem::malloc(1 * mem::size_of::<i32>());

    let ptr_to_int = ptr as usize;

    let mut ptr_from_int = ptr_to_int as shmem::SymmMemAddr;

    println!("{}: PE {:>6}, ptr = {:p}", node, me, ptr_from_int);

    shmem::free(ptr_from_int);

    shmem::finalize();
}
