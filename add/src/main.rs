use shmem;
use std::mem;
use uname::uname;

fn main() {
    let node = uname().unwrap().nodename;

    shmem::init();

    let me = shmem::my_pe();
    let n = shmem::n_pes();

    let mut dest = shmem::SymmMem::<i32>::new(1);

    dest.set(0, 6);

    shmem::barrier_all();
    
    if me == 0 {
	    shmem::int_atomic_add(&dest, 4, n - 1);
    }

    shmem::barrier_all();

	println!("{}: PE {:>6} dest = {:>6}", node, me, dest.get(0));

    shmem::finalize();
}