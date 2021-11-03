use shmem::*;
use std::mem::size_of;

fn main() {
    init();

    let mut counter = shmem::SymmMem::<i32>::new(1);
        
    *counter = 0;

    barrier_all();

    // counter is now 0 everywhere

    // add me+1 to PE 0
    let me = my_pe();

    int_atomic_add(&counter, me + 1, 0);

    barrier_all();

    if me == 0 {
        let n = n_pes();

        println!("Sum from 1 to {} = {}", n, *counter);
    }

    finalize();
}
