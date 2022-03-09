use shmem::*;

fn main() {
    init();

    let mut counter = SymmMem::<i32>::new(1);

    *counter = 0;

    let me = my_pe();

    barrier_all();

    counter.atomic_add(me + 1, 0);

    barrier_all();

    if me == 0 {
        let n = n_pes();
        println!("Sum from 1 to {} = {}", n, counter.get(0));
    }

    finalize();
}
