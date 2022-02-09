use shmem;
use uname::uname;

fn main() {
    let node = uname().unwrap().nodename;

    shmem::init();

    let me = shmem::my_pe();
    let n = shmem::n_pes();

    let nextpe = (me + 1) % n;

    let mut dest = shmem::SymmMem::<i32>::new(1);

    shmem::int_p(&dest, nextpe, nextpe);

    shmem::barrier_all();

    let val = dest.get(0);

    print!("{}: {:>6}: got {:>6}", node, me, *val);
    if *val == me {
        println!("  CORRECT");
    } else {
        println!("  WRONG, expected {}", me);
    }

    shmem::finalize();
}
