use shmem;
use std::env;
use std::string::String;
use uname::uname;

fn decode(tl: shmem::ThreadLevel) -> String {
    let res: &str = match tl {
        shmem::THREAD_SINGLE => "SINGLE",
        shmem::THREAD_FUNNELED => "FUNNELED",
        shmem::THREAD_SERIALIZED => "SERIALIZED",
        shmem::THREAD_MULTIPLE => "MULTIPLE",
        _ => "unknown",
    };

    res.to_string()
}

fn encode(ts: &str) -> shmem::ThreadLevel {
    let res: shmem::ThreadLevel = match ts {
        "SINGLE" => shmem::THREAD_SINGLE,
        "FUNNELED" => shmem::THREAD_FUNNELED,
        "SERIALIZED" => shmem::THREAD_SERIALIZED,
        "MULTIPLE" => shmem::THREAD_MULTIPLE,
        _ => shmem::THREAD_SINGLE - 1,
    };

    res
}

fn main() {
    let argv: Vec<String> = env::args().collect();

    let node = uname().unwrap().nodename;

    let requested;

    if argv.len() > 1 {
        requested = encode(&argv[1]);
    } else {
        requested = shmem::THREAD_MULTIPLE;
    }

    let provided: shmem::ThreadLevel = shmem::init_thread(requested);

    let me = shmem::my_pe();

    if me == 0 {
        println!(
            "{}: requested {} ({}), got {} ({})",
            node,
            decode(requested),
            requested,
            decode(provided),
            provided
        );
    }

    shmem::finalize();
}
