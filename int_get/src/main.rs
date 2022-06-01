use shmem::*;
use uname::uname;

fn main() {
    let _node = uname().unwrap().nodename;

    init();

    let me = my_pe();
    let npes = n_pes();

    let nextpe = (me + 1) % npes;

    let mut src = SymmMem::<i32>::new(4);
    let mut dest = SymmMem::<i32>::new(4);

    for x in 0..4 {
        src.set(x as usize, x * me);
    }

    barrier_all();

    /* Get values from next pe */
    dest.get_values(&src, 4, nextpe);

    for x in 0..4 {
        println!(
            "PE {}/{}, idx {}, value = {}",
            me,
            npes,
            x,
            dest.get(x as usize)
        );
    }

    finalize();
}
