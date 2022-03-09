use shmem::*;
use std::mem;
use uname::uname;

fn main() {
    let node = uname().unwrap().nodename;

    init();

    let me = my_pe();
    let _n = n_pes();

    let ptr = malloc(1 * mem::size_of::<i32>());

    let ptr_to_int = ptr as usize;

    let ptr_from_int = ptr_to_int as SymmMemAddr;

    println!("{}: PE {:>6}, ptr = {:p}", node, me, ptr_from_int);

    free(ptr_from_int);

    finalize();
}
