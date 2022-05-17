use shmem::*;

fn power2NodesRandomAccessUpdate(
    u64 logTableSize, 
    u64 tableSize, 
    u64 localTableSize, 
    u64 minLocalTableSize, 
    u64 globalStartMyProc, 
    u64 top,
    i32 logNumProcs,
    i32 npes, 
    i32 remainder,
    i32 me,
    i64 procNumUpdates)
{

}

fn main() {
    init();

    let me = my_pe();
    let npes = n_pes();

    /* Initialize Collective Operation Parameters */
    let mut llpSync = SymmMem::<i64>::new(BCAST_SYNC_SIZE);
    let mut llpWrk = SymmMem::<i64>::new(REDUCE_SYNC_SIZE);
    let mut ipSync = SymmMem::<i64>::new(BCAST_SYNC_SIZE);
    let mut ipWrk = SymmMem::<i64>::new(REDUCE_SYNC_SIZE);
    for i in 0usize..BCAST_SYNC_SIZE {
        ipSync.set(i, SYNC_VALUE);
        llpSync.set(i, SYNC_VALUE);
    }

    /* Calculate TableSize (update array size must be a power of 2) */
    let totalMem = ((200000 * npes) / 8) * 0.5;
    let logTableSize = 0;
    let logNumProcs = 0;
    let tableSize = 1;
    while totalMem >= 1.0 {
        totalMem *= 0.5;
        logTableSize++;
        tableSize <<= 1;
    }

    /* Check if number of processors is a power of 2 */
    let powerOfTwoPEs = false;
    if npes && (npes - 1) == 0 {
        powerOfTwoPEs = true;
        let remainder = 0;
        let top = 0;
        let minLocalTableSize = (tableSize / npes);
        let localTableSize = minLocalTableSize;
        let globalStartMyProc = (minLocalTableSize * me);
    } 
    else {
        if(me == 0) {
            println!("Number of processes must be a power of 2");
        }
        return;
    }

    let HPCC_Table = SymmMem::<u64>::new(localTableSize);

    /* Default number of global updates to table: 4x number of table entries */
    let numUpdatesDefault = 4 * tableSize;
    let procNumUpdates = 4 * localTableSize;
    let numUpdates = numUpdatesDefault;

    if me == 0 {
        println!("Running on {} processors {}\n", npes, powerOfTwoPEs ? "(PowerofTwo)" : "");
        println!("Total Main Table size = 2^{} = {} words\n", logTableSize, tableSize);
        if powerOfTwoPEs {
            println!("PE main table size = 2^{} = {} words/PE\n", (logTableSize - logNpes), tableSize/npes); //check logNpes is 0 here
        }
        else {
            println!("PE main table size = (2^{})/{} = {} words/PE MAX\n", logTableSize, npes, localTableSize);
        }

        println!("Default number of updates (RECOMMENDED) = {}\n", numUpdatesDefault);
    }

    /* Initialize main table */
    for idx in 0..localTableSize {
        HPCC_Table.set(idx, idx + globalStartMyProc);
    }
    barrier_all();
    
    power2NodesRandomAccessUpdate(logTableSize, tableSize, localTableSize, minLocalTableSize, globalStartMyProc, top, logNumProcs, 
        npes, remainder, me, procNumUpdates);

    finalize();
}




