use shmem::*;
use uname::uname;
use std::time::{Instant};
use std::cmp; //cmp::max
extern crate getopts;
use getopts::Options;
use std::mem;

pub const N: i64 = 10000;

fn f(a: f64) -> f64 {
    4.0 / (1.0 + a * a)
}

fn main() {
    const PI25DT = 3.141592653589793238462643;
    let node = uname().unwrap().nodename;   

    init();
    let me = my_pe();
    let npes = n_pes();

    let args: Vec<String> = env::args().collect();
    let numRectangles = 10000; // default # of rectangles
    if me == 0 {
        println!("PE {} on host {}\n", me, node);
        if args.len() > 1 {
            numRectangles = args.parse::<i32>().ok().expect("Expected integer value as argument for number of rectangles");
        }
        let now = Instant::now();
    }

    /* reduction of 1 value: size the workspace accordingly */
    const pWrkRSize = cmp::max(1 / 2 + 1, REDUCE_MIN_WRKDATA_SIZE);

    let pWrkR = SymmMem::<f64>::new(pWrkRSize);

    let mut pSyncB = SymmMem::<i32>::new(BCAST_SYNC_SIZE); /* for broadcast */
    let mut pSyncR = SymmMem::<i32>::new(REDUCE_SYNC_SIZE); /* for reduction */

    /* initialize sync arrays */ //might want to create rust initialize array function
    for i in 0..BCAST_SYNC_SIZE {
        pSyncB.set(i, SYNC_VALUE);
    }
    for i in 0..REDUCE_SYNC_SIZE {
        pSyncR.set(i, SYNC_VALUE);
    }

    barrier_all();

    /* -=- set up done -=- */

    /* send "n" out to everyone */
    broadcast32(&numRectangles, &numRectangles, 1, 0, 0, 0, npes, pSyncB);

    /* do partial computation */
    let h: f64 = 1.0 / N as f64;
    let mut sum: f64 = 0.0;

    /* A slightly better approach starts from large i and works back */
    for i in (me+1..=numRectangles).step_by(npes) {
        const x: f64 = h * (i: f64 - 0.5);
        sum += f(x);
    }

    let mypi: f64 = h * sum;

    /* wait for everyone to finish */
    barrier_all();

    /* add up partial pi computations into PI */
    mypi.sum_to_all(&pi, 1, 0, 0, npes, &pWrkR, &pSyncR);


    if me == 0 {
        let elapsed_time = now.elapsed().as_millis();
        println!("pi is approximately {}, Error is {}\n", pi, (pi - PI25DT).abs());
        println!("run time = {} ms \n", elapsed_time);
    }


    finalize();
}
