use shmem;

fn main() {
    shmem::init();

    let mut counter = shmem::SymmMem::<i32>::new(1);

    *counter = 0;

    let me = shmem::my_pe();

    shmem::barrier_all();

    shmem::int_atomic_add(&counter, me + 1, 0);

    shmem::barrier_all();

    if me == 0 {
        let n = shmem::n_pes();
        println!("Sum from 1 to {} = {}", n, *counter);
    }

    shmem::finalize();
}
