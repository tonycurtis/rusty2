use shmem::*;
use uname::uname;
use std::env;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

const OSHM_LOOP_ATOMIC = 500;
const CHAR_MAX = 127;
const INT_MAX = 2147483647;
const REDUCE_MIN_WRKDATA_SIZE = 1;
const REDUCE_SYNC_SIZE = 3;

struct pe_vars {
    me: i32,
    npes: i32,
    pairs: i32,
    nxtpe: i32,
}

fn print_header_local(my_pe: i32) {
    if my_pe == 0 {
        println!("# RustySHMEM Atomic Operation Rate Test v1.0");
        let col1_spacing = repeat(' ').take(14).collect::<String>();
        let col1_spacing = repeat(' ').take(6).collect::<String>();
        println!("# Operation{}Million ops/s{}Latency (us)", col1_spacing, col2_spacing);
    }
}

fn print_operation_rate(myid: i32, operation: SymmMem::<u8>, rate: f32, lat: f32)
{
    if myid == 0 {
        let col1_spacing = repeat(' ').take(20).collect::<String>();
        let col2_spacing = repeat(' ').take(14).collect::<String>();
        println!("{}{}{}{}{}", operation, col1_spacing, rate, col2_spacing, lat);
    }
}

pub fn drand48() -> f32 {
    let rand_float : f32 = rand::thread_rng().gen();
    rand_float
}

fn benchmark_fadd(v: pe_vars, iterations: u64, psync1: SymmMem::<i64>, psync2: SymmMem::<i64>)
{
    let (rate, sum_rate, lat, sum_lat): (f32, f32, f32, f32) = (0, 0, 0, 0); 
    let buffer = SymmMem::<i32>::new(OSHM_LOOP_ATOMIC);
    /* Touch memory */
    for idx in 0..OSHM_LOOP_ATOMIC {
        buffer.set(idx, CHAR_MAX * drand48())
    }
    barrier_all();

    if v.me < v.pairs {
        let value = 1;
        let old_value = -1;

        let begin: f32 = Instant::now();
        for i in 0..iterations {
            old_value = (buffer.get(i)).fadd(value, v.nxtpe);
        }

        let elapsed_time = now.elapsed().as_millis();

        rate = ((iterations as f32) * 1e6) / (elapsed_time);
        lat = (elapsed_time) / (iterations as f32);
    }

    let pwrk1 = SymmMem::<f32>::new(REDUCE_MIN_WRKDATA_SIZE);
    let pwrk2 = SymmMem::<f32>::new(REDUCE_MIN_WRKDATA_SIZE);
    sum_rate.sum_to_all(rate, 1, 0, 0, v.npes, pwrk1, psync1);
    sum_lat.sum_to_all(lat, 1, 0, 0, v.npes, pwrk2, psync2);
    print_operation_rate(v.me, "shmem_int_fadd", sum_rate/1e6, sum_lat/v.pairs);

    return 0;
}

fn benchmark_finc(v: pe_vars, iterations: i64, psync1: SymmMem::<i64>, psync2: SymmMem::<i64>)
{
    let (rate, sum_rate, lat, sum_lat): (f32, f32, f32, f32) = (0, 0, 0, 0); 
    let buffer = SymmMem::<i32>::new(OSHM_LOOP_ATOMIC);
    /* Touch memory */
    for idx in 0..OSHM_LOOP_ATOMIC {
        buffer.set(idx, CHAR_MAX * drand48())
    }
    barrier_all();

    if v.me < v.pairs {
        let value = 1;
        let old_value = -1;

        let begin: f32 = Instant::now();
        for i in 0..iterations {
            old_value = (buffer.get(i)).finc(v.nxtpe);
        }

        let elapsed_time = now.elapsed().as_millis();

        rate = ((iterations as f32) * 1e6) / (elapsed_time);
        lat = (elapsed_time) / (iterations as f32);
    }

    let pwrk1 = SymmMem::<f32>::new(REDUCE_MIN_WRKDATA_SIZE);
    let pwrk2 = SymmMem::<f32>::new(REDUCE_MIN_WRKDATA_SIZE);
    sum_rate.sum_to_all(rate, 1, 0, 0, v.npes, pwrk1, psync1);
    sum_lat.sum_to_all(lat, 1, 0, 0, v.npes, pwrk2, psync2);
    print_operation_rate(v.me, "shmem_int_finc", sum_rate/1e6, sum_lat/v.pairs);

    return 0;
}

fn benchmark_add(v: pe_vars, iterations: u64, psync1: SymmMem::<i64>, psync2: SymmMem::<i64>)
{
    let (rate, sum_rate, lat, sum_lat): (f32, f32, f32, f32) = (0, 0, 0, 0); 
    let buffer = SymmMem::<i32>::new(OSHM_LOOP_ATOMIC);
    /* Touch memory */
    for idx in 0..OSHM_LOOP_ATOMIC {
        buffer.set(idx, CHAR_MAX * drand48())
    }
    barrier_all();

    if v.me < v.pairs {
        let value = INT_MAX * drand48();

        let begin: f32 = Instant::now();
        for i in 0..iterations {
            (buffer.get(i)).add(value, v.nxtpe);
        }

        let elapsed_time = now.elapsed().as_millis();

        rate = ((iterations as f32) * 1e6) / (elapsed_time);
        lat = (elapsed_time) / (iterations as f32);
    }

    let pwrk1 = SymmMem::<f32>::new(REDUCE_MIN_WRKDATA_SIZE);
    let pwrk2 = SymmMem::<f32>::new(REDUCE_MIN_WRKDATA_SIZE);
    sum_rate.sum_to_all(rate, 1, 0, 0, v.npes, pwrk1, psync1);
    sum_lat.sum_to_all(lat, 1, 0, 0, v.npes, pwrk2, psync2);
    print_operation_rate(v.me, "shmem_int_add", sum_rate/1e6, sum_lat/v.pairs);

    return 0;
}

fn benchmark_add_longlong(v: pe_vars, iterations: u32, psync1: SymmMem::<i64>, psync2: SymmMem::<i64>)
{
    let (rate, sum_rate, lat, sum_lat): (f32, f32, f32, f32) = (0, 0, 0, 0); 
    let buffer = SymmMem::<i32>::new(OSHM_LOOP_ATOMIC);
    /* Touch memory */
    for idx in 0..OSHM_LOOP_ATOMIC {
        buffer.set(idx, CHAR_MAX * drand48())
    }
    barrier_all();

    if v.me < v.pairs {
        let value: i64 = INT_MAX * drand48();

        let begin: f32 = Instant::now();
        for i in 0..iterations {
            (buffer.get(i)).longlong_add(value, v.nxtpe);
        }

        let elapsed_time = now.elapsed().as_millis();

        rate = ((iterations as f32) * 1e6) / (elapsed_time);
        lat = (elapsed_time) / (iterations as f32);
    }

    let pwrk1 = SymmMem::<f32>::new(REDUCE_MIN_WRKDATA_SIZE);
    let pwrk2 = SymmMem::<f32>::new(REDUCE_MIN_WRKDATA_SIZE);
    sum_rate.sum_to_all(rate, 1, 0, 0, v.npes, pwrk1, psync1);
    sum_lat.sum_to_all(lat, 1, 0, 0, v.npes, pwrk2, psync2);
    print_operation_rate(v.me, "shmem_int_add", sum_rate/1e6, sum_lat/v.pairs);

    return 0;
}

fn benchmark_inc(v: pe_vars, iterations: u64, psync1: SymmMem::<i64>, psync2: SymmMem::<i64>)
{
    let (rate, sum_rate, lat, sum_lat): (f32, f32, f32, f32) = (0, 0, 0, 0); 
    let buffer = SymmMem::<i32>::new(OSHM_LOOP_ATOMIC);
    /* Touch memory */
    for idx in 0..OSHM_LOOP_ATOMIC {
        buffer.set(idx, CHAR_MAX * drand48())
    }
    barrier_all();

    if v.me < v.pairs {
        let begin: f32 = Instant::now();
        for i in 0..iterations {
            (buffer.get(i)).inc(v.nxtpe);
        }

        let elapsed_time = now.elapsed().as_millis();

        rate = ((iterations as f32) * 1e6) / (elapsed_time);
        lat = (elapsed_time) / (iterations as f32);
    }

    let pwrk1 = SymmMem::<f32>::new(REDUCE_MIN_WRKDATA_SIZE);
    let pwrk2 = SymmMem::<f32>::new(REDUCE_MIN_WRKDATA_SIZE);
    sum_rate.sum_to_all(rate, 1, 0, 0, v.npes, pwrk1, psync1);
    sum_lat.sum_to_all(lat, 1, 0, 0, v.npes, pwrk2, psync2);
    print_operation_rate(v.me, "shmem_inc", sum_rate/1e6, sum_lat/v.pairs);

    return 0;
}

fn benchmark_cswap(v: pe_vars, iterations: u64, psync1: SymmMem::<i64>, psync2: SymmMem::<i64>)
{
    let (rate, sum_rate, lat, sum_lat): (f32, f32, f32, f32) = (0, 0, 0, 0); 
    let buffer = SymmMem::<i32>::new(OSHM_LOOP_ATOMIC);
    /* Touch memory */
    for idx in 0..OSHM_LOOP_ATOMIC {
        buffer.set(idx, v.me)
    }
    barrier_all();

    if v.me < v.pairs {
        let cond = v.nxtpe;
        let value = INT_MAX * drand48();
        let old_value = -1;

        begin = Instant::now();
        for (i = 0; i < iterations; i++) {
            old_value =  (buffer.get(i)).cswap(cond, value, v.nxtpe);
        }
        let elapsed_time = now.elapsed().as_millis();

        rate = ((iterations as f32) * 1e6) / (elapsed_time);
        lat = (elapsed_time) / (iterations as f32);
    }

    let pwrk1 = SymmMem::<f32>::new(REDUCE_MIN_WRKDATA_SIZE);
    let pwrk2 = SymmMem::<f32>::new(REDUCE_MIN_WRKDATA_SIZE);
    sum_rate.sum_to_all(rate, 1, 0, 0, v.npes, pwrk1, psync1);
    sum_lat.sum_to_all(lat, 1, 0, 0, v.npes, pwrk2, psync2);
    print_operation_rate(v.me, "shmem_int_cswap", sum_rate/1e6, sum_lat/v.pairs);

    return 0;
}

fn benchmark_swap(v: pe_vars, iterations: u64, psync1: SymmMem::<i64>, psync2: SymmMem::<i64>)
{
    let (rate, sum_rate, lat, sum_lat): (f32, f32, f32, f32) = (0, 0, 0, 0); 
    let buffer = SymmMem::<i32>::new(OSHM_LOOP_ATOMIC);
    /* Touch memory */
    for idx in 0..OSHM_LOOP_ATOMIC {
        buffer.set(idx, CHAR_MAX * drand48());
    }
    barrier_all();

    if v.me < v.pairs {
        let value = INT_MAX * drand48();
        let old_value = -1;

        begin = Instant::now();
        for (i = 0; i < iterations; i++) {
            old_value =  (buffer.get(i)).swap(value, v.nxtpe);
        }
        let elapsed_time = now.elapsed().as_millis();

        rate = ((iterations as f32) * 1e6) / (elapsed_time);
        lat = (elapsed_time) / (iterations as f32);
    }

    let pwrk1 = SymmMem::<f32>::new(REDUCE_MIN_WRKDATA_SIZE);
    let pwrk2 = SymmMem::<f32>::new(REDUCE_MIN_WRKDATA_SIZE);
    sum_rate.sum_to_all(rate, 1, 0, 0, v.npes, pwrk1, psync1);
    sum_lat.sum_to_all(lat, 1, 0, 0, v.npes, pwrk2, psync2);
    print_operation_rate(v.me, "shmem_int_swap", sum_rate/1e6, sum_lat/v.pairs);

    return 0;
}

fn benchmark_set(v: pe_vars, iterations: u64)
{
    let (rate, sum_rate, lat, sum_lat): (f32, f32, f32, f32) = (0, 0, 0, 0); 
    let buffer = SymmMem::<i32>::new(OSHM_LOOP_ATOMIC);
    /* Touch memory */
    for idx in 0..OSHM_LOOP_ATOMIC {
        buffer.set(idx, CHAR_MAX * drand48());
    }
    barrier_all();

    if v.me < v.pairs {
        let value = 1;

        begin = Instant::now();
        for (i = 0; i < iterations; i++) {
            old_value =  (buffer.get(i)).set(value, v.nxtpe);
        }
        let elapsed_time = now.elapsed().as_millis();

        rate = ((iterations as f32) * 1e6) / (elapsed_time);
        lat = (elapsed_time) / (iterations as f32);
    }

    let pwrk1 = SymmMem::<f32>::new(REDUCE_MIN_WRKDATA_SIZE);
    let pwrk2 = SymmMem::<f32>::new(REDUCE_MIN_WRKDATA_SIZE);
    sum_rate.sum_to_all(rate, 1, 0, 0, v.npes, pwrk1, psync1);
    sum_lat.sum_to_all(lat, 1, 0, 0, v.npes, pwrk2, psync2);
    print_operation_rate(v.me, "shmem_int_set", sum_rate/1e6, sum_lat/v.pairs);

    return 0;
}

fn benchmark(v: pe_vars, psync1: SymmMem::<i64>, psync2: SymmMem::<i64>)
{
    let mut rng = ChaCha8Rng::seed_from_u64(v.me);
    let int_msg_buffer = SymmMem::<i32>::new(OSHM_LOOP_ATOMIC);

    /* warmup with puts */
    if v.me < v.pairs {
        for i in 0..OSHM_LOOP_ATOMIC {
            int_msg_buffer.putmem(int_msg_buffer, i, v.npes);
        }
    }

    /* Performance with atomics */
    benchmark_fadd(v, OSHM_LOOP_ATOMIC, psync1, psync2);
    benchmark_finc(v,  OSHM_LOOP_ATOMIC, psync1, psync2);
    benchmark_add(v, OSHM_LOOP_ATOMIC, psync1, psync2);
    benchmark_inc(v, OSHM_LOOP_ATOMIC, psync1, psync2);
    benchmark_cswap(v, OSHM_LOOP_ATOMIC, psync1, psync2);
    benchmark_swap(v, OSHM_LOOP_ATOMIC, psync1, psync2);
	benchmark_set(v, OSHM_LOOP_ATOMIC, psync1, psync2);
	benchmark_fetch(v, msg_buffer, OSHM_LOOP_ATOMIC);
    
    benchmark_fadd_longlong(v, msg_buffer, OSHM_LOOP_ATOMIC);
    benchmark_finc_longlong(v, msg_buffer, OSHM_LOOP_ATOMIC);
    benchmark_add_longlong(v, msg_buffer, OSHM_LOOP_ATOMIC);
    benchmark_inc_longlong(v, msg_buffer, OSHM_LOOP_ATOMIC);
    benchmark_cswap_longlong(v, msg_buffer, OSHM_LOOP_ATOMIC);
    benchmark_swap_longlong(v, msg_buffer, OSHM_LOOP_ATOMIC);
	benchmark_set_longlong(v, msg_buffer, OSHM_LOOP_ATOMIC);
	benchmark_fetch_longlong(v, msg_buffer, OSHM_LOOP_ATOMIC);
}

/* RustySHMEM Atomics Test [Heap Mode Only]*/
fn main() {
    const MEMORY_SELECTION: i32 = 1;
    const _SHMEM_SYNC_VALUE: i64 = -1;
    const _SHMEM_REDUCE_SYNC_SIZE: u32 = 3;

    let avg_time = 0.0;
    let max_time = 0.0;
    let min_time = 0.0;
    let double_latency = 0.0;


    init();

    let pe = pe_vars {
        me = my_pe(),
        npes = n_pes(),
    };

    pe.pairs = pe.npes / 2;
    pe.nxtpe = pe.me < pe.pairs ? pe.me + pe.pairs : pe.me - pe.pairs;

    if pe.npes != 2 {
        if myid == 0 {
            eprintln!("This test requires exactly two processes\n");
        }
        exit(1);
    }

    let mut psync1 = SymmMem::<i64>::new(REDUCE_SYNC_SIZE); 
    let mut psync2 = SymmMem::<i64>::new(REDUCE_SYNC_SIZE); 

    for i in 0..REDUCE_SYNC_SIZE {
        psync1.set(i, SYNC_VALUE);
        psync2.set(i, SYNC_VALUE);
    }

    barrier_all();

    print_header_local(v.me);

    /* Time Put Message Rate */
    benchmark(v, psync1, psync2);

    /* Finalize */
    finalize();
}

double
benchmark_fadd_longlong (struct pe_vars v, union data_types *buffer,
                unsigned long iterations)
{
    double begin, end; 
    int i;
    static double rate = 0, sum_rate = 0, lat = 0, sum_lat = 0;

    /*
     * Touch memory
     */
    memset(buffer, CHAR_MAX * drand48(), sizeof(union data_types
                [OSHM_LOOP_ATOMIC]));

    shmem_barrier_all();

    if (v.me < v.pairs) {
        long long value = 1;
        long long old_value;

        begin = TIME();
        for (i = 0; i < iterations; i++) {
            old_value = shmem_longlong_fadd(&(buffer[i].longlong_type), value, v.nxtpe);
        }
        end = TIME();

        rate = ((double)iterations * 1e6) / (end - begin);
        lat = (end - begin) / (double)iterations;
    }

    shmem_double_sum_to_all(&sum_rate, &rate, 1, 0, 0, v.npes, pwrk1, psync1);
    shmem_double_sum_to_all(&sum_lat, &lat, 1, 0, 0, v.npes, pwrk2, psync2);    
    print_operation_rate(v.me, "shmem_longlong_fadd", sum_rate/1e6, sum_lat/v.pairs);
    return 0;
}

double
benchmark_finc_longlong (struct pe_vars v, union data_types *buffer,
                unsigned long iterations)
{
    double begin, end; 
    int i;
    static double rate = 0, sum_rate = 0, lat = 0, sum_lat = 0;

    /*
     * Touch memory
     */
    memset(buffer, CHAR_MAX * drand48(), sizeof(union data_types
                [OSHM_LOOP_ATOMIC]));

    shmem_barrier_all();

    if (v.me < v.pairs) {
        long long old_value;

        begin = TIME();
        for (i = 0; i < iterations; i++) {
            old_value = shmem_longlong_finc(&(buffer[i].longlong_type), v.nxtpe);
        }
        end = TIME();

        rate = ((double)iterations * 1e6) / (end - begin);
        lat = (end - begin) / (double)iterations;
    }

    shmem_double_sum_to_all(&sum_rate, &rate, 1, 0, 0, v.npes, pwrk1, psync1);
    shmem_double_sum_to_all(&sum_lat, &lat, 1, 0, 0, v.npes, pwrk2, psync2);
    print_operation_rate(v.me, "shmem_longlong_finc", sum_rate/1e6, sum_lat/v.pairs);

    return 0;
}

double
benchmark_inc_longlong (struct pe_vars v, union data_types *buffer,
                        unsigned long iterations)
{
    double begin, end; 
    int i;
    static double rate = 0, sum_rate = 0, lat = 0, sum_lat = 0;

    /*
     * Touch memory
     */
    memset(buffer, CHAR_MAX * drand48(), sizeof(union data_types
                [OSHM_LOOP_ATOMIC]));

    shmem_barrier_all();

    if (v.me < v.pairs) {
        begin = TIME();
        for (i = 0; i < iterations; i++) {
            shmem_longlong_inc(&(buffer[i].longlong_type), v.nxtpe);
        }
        end = TIME();

        rate = ((double)iterations * 1e6) / (end - begin);
        lat = (end - begin) / (double)iterations;
    }

    shmem_double_sum_to_all(&sum_rate, &rate, 1, 0, 0, v.npes, pwrk1, psync1);
    shmem_double_sum_to_all(&sum_lat, &lat, 1, 0, 0, v.npes, pwrk2, psync2);
    print_operation_rate(v.me, "shmem_longlong_inc", sum_rate/1e6, sum_lat/v.pairs);

    return 0;
}

double
benchmark_swap_longlong (struct pe_vars v, union data_types *buffer,
                         unsigned long iterations)
{
    double begin, end; 
    int i;
    static double rate = 0, sum_rate = 0, lat = 0, sum_lat = 0;

    /*
     * Touch memory
     */
    memset(buffer, CHAR_MAX * drand48(), sizeof(union data_types
                [OSHM_LOOP_ATOMIC]));

    shmem_barrier_all();

    if (v.me < v.pairs) {
        long long value = INT_MAX * drand48();
        long long old_value;

        begin = TIME();
        for (i = 0; i < iterations; i++) {
            old_value = shmem_longlong_swap(&(buffer[i].longlong_type), value, v.nxtpe);
        }
        end = TIME();

        rate = ((double)iterations * 1e6) / (end - begin);
        lat = (end - begin) / (double)iterations;        
    }

    shmem_double_sum_to_all(&sum_rate, &rate, 1, 0, 0, v.npes, pwrk1, psync1);
    shmem_double_sum_to_all(&sum_lat, &lat, 1, 0, 0, v.npes, pwrk2, psync2);
    print_operation_rate(v.me, "shmem_longlong_swap", sum_rate/1e6, sum_lat/v.pairs);

    return 0;
}

double
benchmark_cswap_longlong (struct pe_vars v, union data_types *buffer,
                          unsigned long iterations)
{
    double begin, end; 
    int i;
    static double rate = 0, sum_rate = 0, lat = 0, sum_lat = 0;

    /*
     * Touch memory
     */
    for (i=0; i<OSHM_LOOP_ATOMIC; i++) {
        buffer[i].int_type = v.me;
    }

    shmem_barrier_all();


    if (v.me < v.pairs) {
        long long cond = v.nxtpe;
        long long value = INT_MAX * drand48();
        long long old_value;

        begin = TIME();
        for (i = 0; i < iterations; i++) {
            old_value = shmem_longlong_cswap(&(buffer[i].longlong_type), cond, value, v.nxtpe);
        }
        end = TIME();

        rate = ((double)iterations * 1e6) / (end - begin);
        lat = (end - begin) / (double)iterations;        
    }

    shmem_double_sum_to_all(&sum_rate, &rate, 1, 0, 0, v.npes, pwrk1, psync1);
    shmem_double_sum_to_all(&sum_lat, &lat, 1, 0, 0, v.npes, pwrk2, psync2);
    print_operation_rate(v.me, "shmem_longlong_cswap", sum_rate/1e6, sum_lat/v.pairs);

    return 0;
}


double
benchmark_fetch (struct pe_vars v, union data_types *buffer,
               unsigned long iterations)
{
    double begin, end; 
    int i;
    static double rate = 0, sum_rate = 0, lat = 0, sum_lat = 0;

    /*
     * Touch memory
     */
    memset(buffer, CHAR_MAX * drand48(), sizeof(union data_types
                [OSHM_LOOP_ATOMIC]));
	

    shmem_barrier_all();

    if (v.me < v.pairs) {
        begin = TIME();
        for (i = 0; i < iterations; i++) {
            shmem_int_fetch(&buffer[i].int_type, v.nxtpe);
        }
        end = TIME();

        rate = ((double)iterations * 1e6) / (end - begin);
        lat = (end - begin) / (double)iterations;
    }

    shmem_double_sum_to_all(&sum_rate, &rate, 1, 0, 0, v.npes, pwrk1, psync1);
    shmem_double_sum_to_all(&sum_lat, &lat, 1, 0, 0, v.npes, pwrk2, psync2);
    print_operation_rate(v.me, "shmem_int_fetch", sum_rate/1e6, sum_lat/v.pairs);

    return 0;
}

double
benchmark_fetch_longlong (struct pe_vars v, union data_types *buffer,
                        unsigned long iterations)
{
    double begin, end; 
    int i;
    static double rate = 0, sum_rate = 0, lat = 0, sum_lat = 0;

    /*
     * Touch memory
     */
    memset(buffer, CHAR_MAX * drand48(), sizeof(union data_types
                [OSHM_LOOP_ATOMIC]));

    shmem_barrier_all();

    if (v.me < v.pairs) {
        begin = TIME();
        for (i = 0; i < iterations; i++) {
            int res = shmem_longlong_fetch(&(buffer[i].longlong_type), v.nxtpe);
        }
        end = TIME();

        rate = ((double)iterations * 1e6) / (end - begin);
        lat = (end - begin) / (double)iterations;
    }

 
    shmem_double_sum_to_all(&sum_rate, &rate, 1, 0, 0, v.npes, pwrk1, psync1);
    shmem_double_sum_to_all(&sum_lat, &lat, 1, 0, 0, v.npes, pwrk2, psync2);
    print_operation_rate(v.me, "shmem_longlong_fetch", sum_rate/1e6, sum_lat/v.pairs);

    return 0;
}

double
benchmark_set_longlong (struct pe_vars v, union data_types *buffer,
                unsigned long iterations)
{
    double begin, end; 
    int i;
    static double rate = 0, sum_rate = 0, lat = 0, sum_lat = 0;

    /*
     * Touch memory
     */
    memset(buffer, CHAR_MAX * drand48(), sizeof(union data_types
                [OSHM_LOOP_ATOMIC]));

    shmem_barrier_all();

    if (v.me < v.pairs) {
        long long value = 1;

        begin = TIME();
        for (i = 0; i < iterations; i++) {
            shmem_longlong_set(&(buffer[i].longlong_type), value, v.nxtpe);
        }
        end = TIME();

        rate = ((double)iterations * 1e6) / (end - begin);
        lat = (end - begin) / (double)iterations;
    }

    shmem_double_sum_to_all(&sum_rate, &rate, 1, 0, 0, v.npes, pwrk1, psync1);
    shmem_double_sum_to_all(&sum_lat, &lat, 1, 0, 0, v.npes, pwrk2, psync2);    
    print_operation_rate(v.me, "shmem_longlong_set", sum_rate/1e6, sum_lat/v.pairs);
    return 0;
}


void
benchmark (struct pe_vars v, union data_types *msg_buffer)
{

    srand(v.me);

    /*
     * Warmup with puts
     */
    if (v.me < v.pairs) {
        unsigned long i;

        for (i = 0; i < OSHM_LOOP_ATOMIC; i++) {
            shmem_putmem(&msg_buffer[i].int_type, &msg_buffer[i].int_type,
                    sizeof(int), v.nxtpe);
        }
    }
   
    /*
     * Performance with atomics
     */ 
    benchmark_fadd(v, msg_buffer, OSHM_LOOP_ATOMIC);
    benchmark_finc(v, msg_buffer, OSHM_LOOP_ATOMIC);
    benchmark_add(v, msg_buffer, OSHM_LOOP_ATOMIC);
    benchmark_inc(v, msg_buffer, OSHM_LOOP_ATOMIC);
    benchmark_cswap(v, msg_buffer, OSHM_LOOP_ATOMIC);
    benchmark_swap(v, msg_buffer, OSHM_LOOP_ATOMIC);
	benchmark_set(v, msg_buffer, OSHM_LOOP_ATOMIC);
	benchmark_fetch(v, msg_buffer, OSHM_LOOP_ATOMIC);
    
    benchmark_fadd_longlong(v, msg_buffer, OSHM_LOOP_ATOMIC);
    benchmark_finc_longlong(v, msg_buffer, OSHM_LOOP_ATOMIC);
    benchmark_add_longlong(v, msg_buffer, OSHM_LOOP_ATOMIC);
    benchmark_inc_longlong(v, msg_buffer, OSHM_LOOP_ATOMIC);
    benchmark_cswap_longlong(v, msg_buffer, OSHM_LOOP_ATOMIC);
    benchmark_swap_longlong(v, msg_buffer, OSHM_LOOP_ATOMIC);
	benchmark_set_longlong(v, msg_buffer, OSHM_LOOP_ATOMIC);
	benchmark_fetch_longlong(v, msg_buffer, OSHM_LOOP_ATOMIC);
}
