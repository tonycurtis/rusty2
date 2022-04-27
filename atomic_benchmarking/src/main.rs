use shmem::*;
use rand::prelude::*;
use std::time::{Instant};
use std::iter::repeat;
use std::process::exit;

const OSHM_LOOP_ATOMIC: usize = 500;
const REDUCE_MIN_WRKDATA_SIZE: usize = 1;
const REDUCE_SYNC_SIZE: usize = 3;

struct PEvars {
    me: i32,
    npes: i32,
    pairs: i32,
    nxtpe: i32,
}

fn print_header_local() {
        println!("# RustySHMEM Atomic Operation Rate Test v1.0");
        let col1_spacing = repeat(' ').take(14).collect::<String>();
        let col2_spacing = repeat(' ').take(6).collect::<String>();
        println!("# Operation{}Million ops/s{}Latency (us)", col1_spacing, col2_spacing);
}

fn print_operation_rate(operation: &str, rate: f32, lat: f32)
{
    let col1_spacing = repeat(' ').take(20).collect::<String>();
    let col2_spacing = repeat(' ').take(14).collect::<String>();
    println!("{}{}{}{}{}", operation, col1_spacing, rate, col2_spacing, lat);
}

/* Generates random number over integer range */ 
fn drand48() -> i32 {
    let mut rng = rand::thread_rng();
    rng.gen::<i32>()
}

fn sum_operation_data(v: &PEvars, operation: &str, rate: f32, lat: f32,  psync1: &SymmMem::<i64>, psync2: &SymmMem::<i64>) {
    let pwrk1 = SymmMem::<f32>::new(REDUCE_MIN_WRKDATA_SIZE);
    let pwrk2 = SymmMem::<f32>::new(REDUCE_MIN_WRKDATA_SIZE);
    let mut sum_rate = SymmMem::<f32>::new(1);
    let mut rate_ptr = SymmMem::<f32>::new(1);
    let mut sum_lat = SymmMem::<f32>::new(1);
    let mut lat_ptr = SymmMem::<f32>::new(1);
    rate_ptr.set(0, rate);
    lat_ptr.set(0, lat);
    
    sum_rate.sum_to_all(&rate_ptr, 1, 0, 0, v.npes, &pwrk1, &psync1);
    sum_lat.sum_to_all(&lat_ptr, 1, 0, 0, v.npes, &pwrk2, &psync2);
    if v.me == 0 {
        print_operation_rate(operation, (sum_rate.get(0))/1e6, (sum_lat.get(0))/(v.pairs as f32));
    }
}

fn benchmark_fadd(v: &PEvars)
{
    let mut rate: f32 = 0.0;
    let mut lat: f32 = 0.0;
    let mut buffer = SymmMem::<i32>::new(OSHM_LOOP_ATOMIC);
    /* Touch memory */
    for idx in 0usize..OSHM_LOOP_ATOMIC {
        buffer.set(idx, drand48())
    }
    let mut psync1 = SymmMem::<i64>::new(REDUCE_SYNC_SIZE);
    let mut psync2 = SymmMem::<i64>::new(REDUCE_SYNC_SIZE);

    for i in 0usize..REDUCE_SYNC_SIZE {
        psync1.set(i, SYNC_VALUE);
        psync2.set(i, SYNC_VALUE);
    }
    barrier_all();

    if v.me < v.pairs {
        let value = 1;

        let now = Instant::now();
        buffer.fadd(value, v.nxtpe);

        let elapsed_time: f32 = now.elapsed().as_millis() as f32;

        rate = ((OSHM_LOOP_ATOMIC as f32) * 1e6) / (elapsed_time);
        lat = (elapsed_time) / (OSHM_LOOP_ATOMIC as f32);
    }

    sum_operation_data(&v, "shmem_int_fadd", rate, lat, &psync1, &psync2);
}

fn benchmark_finc(v: &PEvars)
{
    let mut rate: f32 = 0.0;
    let mut lat: f32 = 0.0;
    let mut buffer = SymmMem::<i32>::new(OSHM_LOOP_ATOMIC);
    /* Touch memory */
    for idx in 0usize..OSHM_LOOP_ATOMIC {
        buffer.set(idx, drand48())
    }
    let mut psync1 = SymmMem::<i64>::new(REDUCE_SYNC_SIZE);
    let mut psync2 = SymmMem::<i64>::new(REDUCE_SYNC_SIZE);

    for i in 0usize..REDUCE_SYNC_SIZE {
        psync1.set(i, SYNC_VALUE);
        psync2.set(i, SYNC_VALUE);
    }
    barrier_all();

    if v.me < v.pairs {
        let now = Instant::now();
        buffer.finc(v.nxtpe);

        let elapsed_time: f32 = now.elapsed().as_millis() as f32;

        rate = ((OSHM_LOOP_ATOMIC as f32) * 1e6) / (elapsed_time);
        lat = (elapsed_time) / (OSHM_LOOP_ATOMIC as f32);
    }

    sum_operation_data(&v, "shmem_int_finc", rate, lat, &psync1, &psync2);
}

fn benchmark_add(v: &PEvars)
{
    let mut rate: f32 = 0.0;
    let mut lat: f32 = 0.0;
    let mut buffer = SymmMem::<i32>::new(OSHM_LOOP_ATOMIC);
    /* Touch memory */
    for idx in 0usize..OSHM_LOOP_ATOMIC {
        buffer.set(idx, drand48())
    }
    let mut psync1 = SymmMem::<i64>::new(REDUCE_SYNC_SIZE);
    let mut psync2 = SymmMem::<i64>::new(REDUCE_SYNC_SIZE);

    for i in 0usize..REDUCE_SYNC_SIZE {
        psync1.set(i, SYNC_VALUE);
        psync2.set(i, SYNC_VALUE);
    }
    barrier_all();

    if v.me < v.pairs {
        let value = drand48();
        let now = Instant::now();

        buffer.add(value, v.nxtpe);

        let elapsed_time: f32 = now.elapsed().as_millis() as f32;
        rate = ((OSHM_LOOP_ATOMIC as f32) * 1e6) / (elapsed_time);
        lat = (elapsed_time) / (OSHM_LOOP_ATOMIC as f32);
    }

    sum_operation_data(&v, "shmem_int_add", rate, lat, &psync1, &psync2);
}

fn benchmark_inc(v: &PEvars)
{
    let mut rate: f32 = 0.0;
    let mut lat: f32 = 0.0;
    let mut buffer = SymmMem::<i32>::new(OSHM_LOOP_ATOMIC);
    /* Touch memory */
    for idx in 0usize..OSHM_LOOP_ATOMIC {
        buffer.set(idx, drand48())
    }
    let mut psync1 = SymmMem::<i64>::new(REDUCE_SYNC_SIZE);
    let mut psync2 = SymmMem::<i64>::new(REDUCE_SYNC_SIZE);

    for i in 0usize..REDUCE_SYNC_SIZE {
        psync1.set(i, SYNC_VALUE);
        psync2.set(i, SYNC_VALUE);
    }
    barrier_all();

    if v.me < v.pairs {
        let now = Instant::now();

        buffer.inc(v.nxtpe);

        let elapsed_time: f32 = now.elapsed().as_millis() as f32;
        rate = ((OSHM_LOOP_ATOMIC as f32) * 1e6) / (elapsed_time);
        lat = (elapsed_time) / (OSHM_LOOP_ATOMIC as f32);
    }

    sum_operation_data(&v, "shmem_int_inc", rate, lat, &psync1, &psync2);
}

fn benchmark_cswap(v: &PEvars)
{
    let mut rate: f32 = 0.0;
    let mut lat: f32 = 0.0;
    let mut buffer = SymmMem::<i32>::new(OSHM_LOOP_ATOMIC);
    /* Touch memory */
    for idx in 0usize..OSHM_LOOP_ATOMIC {
        buffer.set(idx, drand48())
    }
    let mut psync1 = SymmMem::<i64>::new(REDUCE_SYNC_SIZE);
    let mut psync2 = SymmMem::<i64>::new(REDUCE_SYNC_SIZE);

    for i in 0usize..REDUCE_SYNC_SIZE {
        psync1.set(i, SYNC_VALUE);
        psync2.set(i, SYNC_VALUE);
    }
    barrier_all();

    if v.me < v.pairs {
        let cond = v.nxtpe;
        let value = drand48();

        let now = Instant::now();
        buffer.cswap(cond, value, v.nxtpe);
        let elapsed_time: f32 = now.elapsed().as_millis() as f32;

        rate = ((OSHM_LOOP_ATOMIC as f32) * 1e6) / (elapsed_time);
        lat = (elapsed_time) / (OSHM_LOOP_ATOMIC as f32);
    }

    sum_operation_data(&v, "shmem_int_cswap", rate, lat, &psync1, &psync2);
}

fn benchmark_swap(v: &PEvars)
{
    let mut rate: f32 = 0.0;
    let mut lat: f32 = 0.0;
    let mut buffer = SymmMem::<i32>::new(OSHM_LOOP_ATOMIC);
    /* Touch memory */
    for idx in 0usize..OSHM_LOOP_ATOMIC {
        buffer.set(idx, drand48())
    }
    let mut psync1 = SymmMem::<i64>::new(REDUCE_SYNC_SIZE);
    let mut psync2 = SymmMem::<i64>::new(REDUCE_SYNC_SIZE);

    for i in 0usize..REDUCE_SYNC_SIZE {
        psync1.set(i, SYNC_VALUE);
        psync2.set(i, SYNC_VALUE);
    }
    barrier_all();

    if v.me < v.pairs {
        let value = drand48();
        let now = Instant::now();

        buffer.swap(value, v.nxtpe);

        let elapsed_time: f32 = now.elapsed().as_millis() as f32;
        rate = ((OSHM_LOOP_ATOMIC as f32) * 1e6) / (elapsed_time);
        lat = (elapsed_time) / (OSHM_LOOP_ATOMIC as f32);
    }

    sum_operation_data(&v, "shmem_int_swap", rate, lat, &psync1, &psync2);
}

fn benchmark_set(v: &PEvars)
{
    let mut rate: f32 = 0.0;
    let mut lat: f32 = 0.0;
    let mut buffer = SymmMem::<i32>::new(OSHM_LOOP_ATOMIC);
    /* Touch memory */
    for idx in 0usize..OSHM_LOOP_ATOMIC {
        buffer.set(idx, drand48())
    }
    let mut psync1 = SymmMem::<i64>::new(REDUCE_SYNC_SIZE);
    let mut psync2 = SymmMem::<i64>::new(REDUCE_SYNC_SIZE);

    for i in 0usize..REDUCE_SYNC_SIZE {
        psync1.set(i, SYNC_VALUE);
        psync2.set(i, SYNC_VALUE);
    }
    barrier_all();

    if v.me < v.pairs {
        let value = 1;
        let now = Instant::now();

        buffer.set(value, v.nxtpe);

        let elapsed_time: f32 = now.elapsed().as_millis() as f32;
        rate = ((OSHM_LOOP_ATOMIC as f32) * 1e6) / (elapsed_time);
        lat = (elapsed_time) / (OSHM_LOOP_ATOMIC as f32);
    }

    sum_operation_data(&v, "shmem_int_set", rate, lat, &psync1, &psync2);
}

fn benchmark_fetch(v: &PEvars)
{
    let mut rate: f32 = 0.0;
    let mut lat: f32 = 0.0;
    let mut buffer = SymmMem::<i32>::new(OSHM_LOOP_ATOMIC);
    /* Touch memory */
    for idx in 0usize..OSHM_LOOP_ATOMIC {
        buffer.set(idx, drand48())
    }
    let mut psync1 = SymmMem::<i64>::new(REDUCE_SYNC_SIZE);
    let mut psync2 = SymmMem::<i64>::new(REDUCE_SYNC_SIZE);

    for i in 0usize..REDUCE_SYNC_SIZE {
        psync1.set(i, SYNC_VALUE);
        psync2.set(i, SYNC_VALUE);
    }
    barrier_all();

    if v.me < v.pairs {
        let now = Instant::now();
        buffer.fetch(v.nxtpe);
        let elapsed_time: f32 = now.elapsed().as_millis() as f32;
        rate = ((OSHM_LOOP_ATOMIC as f32) * 1e6) / (elapsed_time);
        lat = (elapsed_time) / (OSHM_LOOP_ATOMIC as f32);
    }

    sum_operation_data(&v, "shmem_int_fetch", rate, lat, &psync1, &psync2);
}


fn benchmark(v: &PEvars)
{
    let mut msg_buffer_src = SymmMem::<i32>::new(OSHM_LOOP_ATOMIC);
    let msg_buffer_dest = SymmMem::<i32>::new(OSHM_LOOP_ATOMIC);

    /* warmup with puts */
    if v.me < v.pairs {
        for i in 0..OSHM_LOOP_ATOMIC {
            msg_buffer_src.putmem(&msg_buffer_dest, i as u64, v.npes);
        }
    }

    /* Performance with atomics */
    benchmark_fadd(&v);
    benchmark_finc(&v);
    benchmark_add(&v);
    benchmark_inc(&v);
    benchmark_cswap(&v);
    benchmark_swap(&v);
	benchmark_set(&v);
	benchmark_fetch(&v);
}

/* RustySHMEM Atomics Test [Heap Mode Only]*/
fn main() {
    init();

    let me = my_pe();
    let npes = n_pes();

    let mut v = PEvars {
        me: me,
        npes: npes,
        pairs: npes / 2,
        nxtpe: -1
    };

    v.nxtpe = if v.me < v.pairs { v.me + v.pairs } else { v.me - v.pairs };

    if v.npes != 2 {
        if v.me == 0 {
            eprintln!("This test requires exactly two processes\n");
        }
        exit(1);
    }

    barrier_all();

    if v.me == 0 {
        print_header_local();
    }

    /* Time Put Message Rate */
    benchmark(&v);

    /* Finalize */
    finalize();
}