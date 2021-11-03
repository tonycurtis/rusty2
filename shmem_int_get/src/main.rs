use shmem;
use std::mem;
use uname::uname;

fn main() {
    let node = uname().unwrap().nodename;

    shmem::init();

    let me = shmem::my_pe();
    let npes = shmem::n_pes();

    let nextpe = (me + 1) % npes;

    let mut src = shmem::SymmMem::<i32>::new(4);
    let mut dest = shmem::SymmMem::<i32>::new(4);
    
    for x in 0..4 {
        src.set(x as usize, x * me);
    }

    shmem::barrier_all();

    /* Get values from next pe */
    shmem::int_get(&dest, &src, 4, nextpe);

    for x in 0..4 {
        println!("PE {}/{}, idx {}, value = {}", me, npes, x, dest.get(x as usize));
    }

    shmem::finalize();
}
