use shmem::*;
use std::process::exit;
use std::time::{Instant};

/* RustySHMEM Get Latency Test [Heap Mode Only]*/
fn main() {
    let repeat: i32 = 10000;
    const MAX_MSG_SIZE_PT2PT: usize = 1<<20;

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
    let mut s_buf = SymmMem::<u8>::new(MAX_MSG_SIZE_PT2PT); 
    let mut r_buf = SymmMem::<u8>::new(MAX_MSG_SIZE_PT2PT);  

    /**************Memory Allocation Done*********************/

    if myid == 0 {
        println!("OpenSHMEM Get Test");
        println!("# Size        Latency (us)");
    }

    let mut now = Instant::now();
    let mut size = 1;
    while size <= MAX_MSG_SIZE_PT2PT {
        /* touch the data */
        for i in 0..size {
            s_buf.set(i, b'a');
            r_buf.set(i, b'b');            
        }

        barrier_all();  
        
        if myid == 0 {
            for i in 0..repeat {
                if i == 0 {
                    now =  Instant::now();
                }
                s_buf.get_values(&r_buf, size as u64, 1);
            }

            println!("{}        {} ms", size, now.elapsed().as_millis());
        }
        barrier_all();
        size = size * 2;
    }

    barrier_all();

    finalize();

    exit(0);
}
