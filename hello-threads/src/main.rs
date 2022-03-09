use shmem::*;
use std::env;
use std::string::String;
use uname::uname;

fn decode(tl: shmem::ThreadLevel) -> String {
    let res: &str = match tl {
        THREAD_SINGLE => "SINGLE",
        THREAD_FUNNELED => "FUNNELED",
        THREAD_SERIALIZED => "SERIALIZED",
        THREAD_MULTIPLE => "MULTIPLE",
        _ => "unknown",
    };

    res.to_string()
}

fn encode(ts: &str) -> ThreadLevel {
    let res: ThreadLevel = match ts {
        "SINGLE" => THREAD_SINGLE,
        "FUNNELED" => THREAD_FUNNELED,
        "SERIALIZED" => THREAD_SERIALIZED,
        "MULTIPLE" => THREAD_MULTIPLE,
        _ => THREAD_SINGLE - 1,
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
        requested = THREAD_MULTIPLE;
    }

    let provided: ThreadLevel = init_thread(requested);

    let me = my_pe();

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

    finalize();
}
