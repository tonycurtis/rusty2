use shmem::*;
use uname::uname;

pub const N: i64 = 10000;

fn f(a: f64) -> f64 {
    4.0 / (1.0 + a * a)
}

fn main() {
    let node = uname().unwrap().nodename;

    init();
    let me = my_pe();
    let npes = n_pes();

    let mut pi = SymmMem::<f64>::new(1);

    let h: f64 = 1.0 / N as f64;
    let mut sum: f64 = 0.0;

    for i in (me + 1..N as i32).step_by(npes as usize) {
        let x = h * ((i as f64) - 0.5);

        sum += f(x);
    }

    let mypi = h * sum;

    barrier_all();

    // shmem::double_sum_to_all(pi, mypi, 1, 0, 0, npes);

    pi.set(0, mypi * npes as f64); // fudge

    barrier_all();

    println!("PE {}/{} on \"{}\" pi = {}", me, npes, node, pi.get(0));

    finalize();
}
