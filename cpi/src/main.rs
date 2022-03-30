use shmem::*;
use uname::uname;
use std::time::{Instant};
use std::cmp; 
use std::env;

fn f(a: f64) -> f64 {
    4.0 / (1.0 + a * a)
}

fn main() {
    const PI25DT: f64 = 3.141592653589793238462643;
    let node = uname().unwrap().nodename;   

    init();
    let me = my_pe();
    let npes = n_pes();

    let args: Vec<String> = env::args().collect();
    let mut num_rectangles = 10000; // default # of rectangles
    if me == 0 {
        println!("PE {} on host {}\n", me, node);
        if args.len() > 1 {
            num_rectangles = args[1].parse::<i32>().ok().expect("Expected integer value as argument for number of rectangles");
        }
    }

    let now = Instant::now(); 

    /* reduction of 1 value: size the workspace accordingly */
    let reduce_min_wrkdata_size = REDUCE_MIN_WRKDATA_SIZE;
    let p_wrk_reduce_size: usize = cmp::max(1 / 2 + 1, reduce_min_wrkdata_size);

    let p_wrk_reduce = SymmMem::<f64>::new(p_wrk_reduce_size);

    let mut p_sync_bcast = SymmMem::<i64>::new(BCAST_SYNC_SIZE); /* for broadcast */
    let mut p_sync_reduce = SymmMem::<i64>::new(REDUCE_SYNC_SIZE); /* for reduction */

    /* initialize sync arrays */ //might want to create rust initialize array function
    for i in 0..BCAST_SYNC_SIZE {
        p_sync_bcast.set(i, SYNC_VALUE);
    }
    for i in 0..REDUCE_SYNC_SIZE {
        p_sync_reduce.set(i, SYNC_VALUE);
    }

    barrier_all();

    /* -=- set up done -=- */

    /* send "n" out to everyone */
    let mut n = SymmMem::<i32>::new(1);
    n.set(0, num_rectangles);
    broadcast32(&n, &n, 1, 0, 0, 0, npes, &p_sync_bcast);

    /* do partial computation */
    let h: f64 = 1.0 / num_rectangles as f64;
    let mut sum: f64 = 0.0;

    /* A slightly better approach starts from large i and works back */
    for i in (me+1..=num_rectangles).step_by(npes as usize) {
        let index = i as f64;
        let x = h * (index - 0.5);
        sum += f(x);
    }

    let mut mypi = SymmMem::<f64>::new(1);
    mypi.set(0, h * sum);
    let mut pi = SymmMem::<f64>::new(1);

    /* wait for everyone to finish */
    barrier_all();

    /* add up partial pi computations into PI */
    mypi.sum_to_all(&pi, 1, 0, 0, npes, &p_wrk_reduce, &p_sync_reduce);


    if me == 0 {
        let elapsed_time = now.elapsed().as_millis();
        let approx_pi = pi.get(0);
        println!("pi is approximately {}, Error is {}\n", approx_pi, (approx_pi - PI25DT).abs());
        println!("run time = {} ms \n", elapsed_time);
    }


    finalize();
}
