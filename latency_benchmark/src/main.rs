use shmem::*;
use std::env;
use std::process::exit;
use std::time::{Instant};

/* RustySHMEM Get Latency Test [Heap Mode Only]*/
fn main() {
    const MYBUFSIZE: usize = 1 << 22;
    let skip: i32 = 1000;
    let repeat: i32 = 10000;
    let skip_large: i32 = 10;
    let repeat_large: i32 = 100;
    const large_message_size: i32 = 8192;
    const MESSAGE_ALIGNMENT: i32 = 64;
    const MAX_MSG_SIZE_PT2PT: usize = 1<<20;

    let args: Vec<String> = env::args().collect();
    let memory_query = &args[1];
    let elapsed_time;

    init();

    let myid = my_pe();
    let numprocs = n_pes();

    if numprocs != 2 {
        if myid == 0 {
            eprintln!("This test requires exactly two processes\n");
        }
        exit(1);
    }

    /**************Memory Allocation*********************/

    let mut s_buf_original: [char; MYBUFSIZE] = ['\0'; MYBUFSIZE];
    let mut r_buf_original: [char; MYBUFSIZE] = ['\0'; MYBUFSIZE];
    let mut s_buf_heap = SymmMem::<char>::new(MYBUFSIZE);
    let mut r_buf_heap = SymmMem::<char>::new(MYBUFSIZE);
    let mut s_buf = SymmMem::<char>::new(MAX_MSG_SIZE_PT2PT); 
    let mut r_buf = SymmMem::<char>::new(MAX_MSG_SIZE_PT2PT);  

    /**************Memory Allocation Done*********************/

    if myid == 0 {
        println!("OpenSHMEM Put Test");
        println!("# Size        Latency (us)");
        println!("# Size        Latency (us)");
    }

    let now;
    let size = 0;
    for size in (1..MAX_MSG_SIZE_PT2PT).step_by(size * 2) {
        /* touch the data */
        for i in 0..size {
            s_buf.set(i, 'a');
            r_buf.set(i, 'b');            
        }

        if size as i32 > large_message_size {
            repeat = 100;
            repeat_large = 100;
            skip = 0;    
            skip_large = 0;        
        }

        barrier_all();  
        
        if myid == 0 {
            for i in 0..repeat+skip {
                if i == skip {
                    now =  Instant::now();
                }
                s_buf.getmem(r_buf, size, 1);
            }

            elapsed_time = now.elapsed();
        }
        barrier_all();
        if myid == 0 {
            let latency = (1.0 * (elapsed_time.as_millis())) / repeat;
            println!("{}        {}", size, latency);
        }
    }

    barrier_all();

    finalize();

    exit(0);
}
