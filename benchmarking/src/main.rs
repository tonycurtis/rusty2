use shmem::*;
use uname::uname;
use std::time::{Instant};

/* OpenSHMEM Put Benchmarking Example
 * * This tests speed of looping put 100x     
 */
fn main() {
    let _node = uname().unwrap().nodename;

    init();

    let me = my_pe();
    let npes = n_pes();
    let nextpe = (me + 1) % npes;
    let num_elems = 10000;
    let num_iters = 100000;

    let mut src = SymmMem::<i32>::new(num_elems);
    let mut dest = SymmMem::<i32>::new(num_elems);

    for x in 0..num_elems {
        src.set(x, (x as i32) * (me + 1));
    }

    barrier_all();

    let now = Instant::now();

    for _x in 0..num_iters {
        src.put(&dest, num_elems as u64, nextpe);
    }

    barrier_all();

    let elapsed_time = now.elapsed();

    if me == 0
    {
        for x in 0..4 {
            println!(
                "PE {}/{}, idx {}, value = {}",
                me,
                npes,
                x,
                dest.get(x as usize)
            );
        }

        println!("Elapsed Time: {} ms", elapsed_time.as_millis());
    }

    finalize();
}
