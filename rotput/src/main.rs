use shmem;
use std::mem;
use uname::uname;

fn main() {
    let node = uname().unwrap().nodename;

    shmem::init();

    // let mut r: shmem::RDMA;

    let me = shmem::my_pe();
    let n = shmem::n_pes();

    let nextpe = (me + 1) % n;

    let mut dest = shmem::SymmMem::<i32>::new(1);

    shmem::int_p(&dest, nextpe, nextpe);

    // let xx = shmem::RdmaOp { dest: dest, src: nextpe, pe: nextpe };
    // shmem::p(xx);

    shmem::barrier_all();

    print!("{}: {:>6}: got {:>6}", node, me, *dest);
    if *dest == me {
        println!("  CORRECT");
    } else {
        println!("  WRONG, expected {}", me);
    }

    shmem::finalize();
}