use shmem::*;
use uname::uname;

fn main() {
    let node = uname().unwrap().nodename;

    init();

    let me = my_pe();
    let n = n_pes();

    let nextpe = (me + 1) % n;

    let mut dest = SymmMem::<i32>::new(1);

    dest.p(nextpe, nextpe);

    barrier_all();

    let val = dest.get(0);

    print!("{}: {:>6}: got {:>6}", node, me, *val);
    if *val == me {
        println!("  CORRECT");
    } else {
        println!("  WRONG, expected {}", me);
    }

    finalize();
}
