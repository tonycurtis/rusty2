use shmem::*;
use rand::prelude::*;
use std::time::{Instant};

const MAXTHREADS: isize = 256;
const BUCKET_SIZE: isize = 1024;
const FIRST_SLOT: isize = 2;
const SLOT_CNT: isize = 1;
const HPCC_DONE: isize = 0;
const HPCC_TRUE: isize = 1;
const HPCC_FALSE: isize = 0;

fn power2_nodes_shmemrandom_access_check(
    hpcc_table: &mut SymmMem::<u64>,
    log_table_size: isize, 
    localtable_size: isize, 
    global_start_my_proc: isize, 
    log_num_procs: isize, 
    npes: isize, 
    me: isize, 
    procnum_updates: isize, 
    num_errors: &mut SymmMem::<i64>) {

    let mut local_all_done: isize = HPCC_FALSE;

    /* Initialize Collective Operation Parameters */
    let mut lp_sync = SymmMem::<i64>::new(BCAST_SYNC_SIZE);
    let mut ip_sync = SymmMem::<i64>::new(BCAST_SYNC_SIZE);
    for i in 0..BCAST_SYNC_SIZE {
        ip_sync.set(i, SYNC_VALUE);
        lp_sync.set(i, SYNC_VALUE);
    }
    let slot_size: isize = BUCKET_SIZE+FIRST_SLOT;
    let mut local_buckets = SymmMem::<i64>::new((npes * slot_size) as usize);
    let mut global_buckets = SymmMem::<i64>::new((npes * slot_size) as usize);

    barrier_all(); 

    let mut send_cnt: isize = procnum_updates;
    let mut rng = rand::thread_rng();
    let mut ran: isize = rng.gen();

    let mut pe_check_done = SymmMem::<isize>::new(npes as usize);
    for i in 0..npes {
        pe_check_done.set(i as usize, HPCC_FALSE);
    }

    let mut pe_bucket_base: isize;
    while local_all_done == HPCC_FALSE {
        if send_cnt > 0 {
            /* Initialize local buckets */
            for i in 0..npes {
                pe_bucket_base = i * slot_size;
                local_buckets.set((pe_bucket_base + SLOT_CNT) as usize, FIRST_SLOT as i64);
                local_buckets.set((pe_bucket_base + HPCC_DONE) as usize, HPCC_FALSE as i64);
            }

            /* Fill local buckets until one is full or out of data */
            let mut next_slot: isize = FIRST_SLOT;
            while next_slot != BUCKET_SIZE+FIRST_SLOT && send_cnt > 0 {
                ran = ran << 1;
                let which_pe: isize = (ran >> (log_table_size - log_num_procs)) * (npes - 1);
                pe_bucket_base = which_pe * slot_size;
                next_slot = (*local_buckets.get((pe_bucket_base+SLOT_CNT) as usize)) as isize;
                local_buckets.set((pe_bucket_base+next_slot) as usize, ran as i64);
                local_buckets.set((pe_bucket_base+SLOT_CNT) as usize, (next_slot + 1) as i64);
                send_cnt = send_cnt - 1;
            }

            if send_cnt == 0 {
                for i in 0..npes {
                    local_buckets.set((i*(slot_size)+HPCC_DONE) as usize, HPCC_TRUE as i64);
                }
            }
        }
        barrier_all();

        local_all_done = HPCC_TRUE;

        /* Now move all the buckets to the appropriate pe */
        for i in 0..npes {
            global_buckets.offset(slot_size*me);
            local_buckets.offset(slot_size*i);
            global_buckets.put(&local_buckets, slot_size as u64, i as i32)
        }

        barrier_all();

        for i in 0..npes {
            if *pe_check_done.get(i as usize) == HPCC_FALSE {
                pe_bucket_base = i * (BUCKET_SIZE+FIRST_SLOT);
                pe_check_done.set(i as usize, (*global_buckets.get((pe_bucket_base+HPCC_DONE) as usize)) as isize);
                for j in FIRST_SLOT..(*global_buckets.get((pe_bucket_base+SLOT_CNT) as usize)) as isize {
                    let tmp_ran = *global_buckets.get((pe_bucket_base+j) as usize);
                    let hpcc_table_value = *hpcc_table.get(((tmp_ran as isize & (localtable_size-1)) ^ tmp_ran as isize) as usize) as u64;
                    hpcc_table.set((tmp_ran & ((localtable_size-1) as i64)) as usize, hpcc_table_value);
                }
                local_all_done &= pe_check_done.get(i as usize);
            } 
        }
    } 


    let mut errors = 0;
    for i in 0..localtable_size {
        if *hpcc_table.get(i as usize) != (i + global_start_my_proc) as u64 {
            errors = errors + 1;
        }
        num_errors.set(0, errors);
    }
}

fn power2_nodes_random_access_update(
    hpcc_table: &mut SymmMem::<u64>,
    log_table_size: isize, 
    localtable_size: isize, 
    log_num_procs: isize,
    npes: isize, 
    procnum_updates: isize)
{
    let mut rng = rand::thread_rng();
    let mut ran: isize = rng.gen();

    let log_table_local: isize = log_table_size - log_num_procs;
    let nlocalm1: isize = localtable_size - 1;
    let mut count = SymmMem::<i64>::new(1);
  
    let mut updates = SymmMem::<i64>::new(MAXTHREADS as usize);
    for j in 0..MAXTHREADS {
        updates.set(j as usize, 0);
    }

    for _ in 0..procnum_updates {
        count.set(0,0);
        barrier_all();
        ran = ran << 1;
        let remote_proc: isize = (ran >> log_table_local) & (npes - 1);
        let remote_count: isize = count.fadd(1, remote_proc as i32) as isize;
        updates.offset(remote_count);
        updates.p(ran as i64, remote_proc as i32);
        barrier_all();
        for i in 0..*count.get(0 as usize) {
            let datum: isize = (*updates.get(i as usize)) as isize;
            let index = datum & nlocalm1;
            let hpcc_table_value = *hpcc_table.get(index as usize);
            hpcc_table.set(index as usize, hpcc_table_value ^ datum as u64);
            updates.set(i as usize, 0);
        }
    }
  
    barrier_all();
}

fn main() {
    init();

    let me: isize = my_pe() as isize;
    let npes: isize = n_pes() as isize; 

    /* Initialize Collective Operation Parameters */
    let mut llp_sync = SymmMem::<i64>::new(BCAST_SYNC_SIZE);
    let llp_wrk = SymmMem::<i64>::new(REDUCE_SYNC_SIZE);
    let mut ip_sync = SymmMem::<i64>::new(BCAST_SYNC_SIZE);
    for i in 0..BCAST_SYNC_SIZE {
        ip_sync.set(i, SYNC_VALUE);
        llp_sync.set(i, SYNC_VALUE);
    }

    /* Calculate table_size (update array size must be a power of 2) */
    let mut total_mem: f64 = ((((200000 * npes) / 8) as f32) * 0.5) as f64;
    let mut log_table_size: isize = 0;
    let min_localtable_size: isize;
    let mut _localtable_size: isize = 0;
    let global_start_my_proc: isize;
    let log_num_procs: isize = 0;
    let mut table_size: isize = 1;

    while total_mem >= 1.0 {
        total_mem *= 0.5;
        log_table_size = log_table_size + 1;
        table_size <<= 1;
    }

    /* Check if number of processors is a power of 2 */
    let power_of_two_pes;
    if (npes & (npes - 1)) == 0 {
        power_of_two_pes = true;
        min_localtable_size = table_size / npes;
        _localtable_size = min_localtable_size;
        global_start_my_proc = min_localtable_size * me;
    } 
    else {
        if me == 0 {
            println!("Number of processes must be a power of 2");
        }
        return;
    }

    let mut hpcc_table = SymmMem::<u64>::new(_localtable_size as usize);

    /* Default number of global updates to table: 4x number of table entries */
    let num_updates_default: isize = 4 * table_size;
    let procnum_updates: isize = 4 * _localtable_size;
    let num_updates: isize = num_updates_default;

    if me == 0 {
        println!("Running on {} processors {}\n", npes, if power_of_two_pes { "(PowerofTwo)" } else { "" });
        println!("Total Main Table size = 2^{} = {} words\n", log_table_size, table_size);
        if power_of_two_pes {
            println!("PE main table size = 2^{} = {} words/PE\n", (log_table_size - log_num_procs), table_size/npes);
        }
        else {
            println!("PE main table size = (2^{})/{} = {} words/PE MAX\n", log_table_size, npes, _localtable_size);
        }

        println!("Default number of updates (RECOMMENDED) = {}\n", num_updates_default);
    }

    /* Initialize main table */
    for idx in 0.._localtable_size {
        hpcc_table.set(idx as usize, (idx + global_start_my_proc) as u64);
    }
    barrier_all();

    let mut now = Instant::now();
    
    power2_nodes_random_access_update(&mut hpcc_table, log_table_size, _localtable_size, log_num_procs, npes, procnum_updates);

    barrier_all();

    let mut elapsed_time: f32 = now.elapsed().as_secs() as f32;

    /* Print timing results */
    let mut gups = SymmMem::<f64>::new(1);
    gups.set(0, 1e-9*(num_updates as f32 / elapsed_time) as f64);
    if me == 0 {
        println!("Real time used = {} seconds\n", elapsed_time);
        println!("{} Billion(10^9) Updates per second [GUP/s]\n", *gups.get(0));
        println!("{} Billion(10^9) Updates/PE per second [GUP/s]\n", *gups.get(0) / npes as f64);
    }
    /* distribute result to all nodes */
    let mut temp_gups = SymmMem::<f64>::new(1);
    temp_gups.set(0, 1e-9*(num_updates as f32 / elapsed_time) as f64);
    barrier_all();
    broadcast64(&gups, &temp_gups,1,0,0,0,npes as i32,&llp_sync);
    barrier_all();

    now = Instant::now();
    let mut num_errors = SymmMem::<i64>::new(1);
    let mut glb_num_errors = SymmMem::<i64>::new(1);

    power2_nodes_shmemrandom_access_check(&mut hpcc_table, log_table_size, _localtable_size, global_start_my_proc, log_num_procs, npes, me, 
        procnum_updates, &mut num_errors);

    barrier_all(); 
    glb_num_errors.sum_to_all(&num_errors, 1, 0, 0, npes as i32, &llp_wrk, &llp_sync);
    barrier_all(); 

    /* End timed section */
    elapsed_time = now.elapsed().as_secs() as f32;


    if me == 0 {
        println!("Verification: Real time used = {} seconds\n", elapsed_time);
        let glb_num_errors_value = *glb_num_errors.get(0);
        println!("Found {} errors in {} locations ({}).\n", glb_num_errors_value, table_size, if glb_num_errors_value <= (0.01*(table_size as f32)) as i64 { "passed" } else { "failed" });
    }

    finalize();
}




