use shmem;
use uname::uname;

fn main() {
    let node = uname().unwrap().nodename;

    shmem::init();

    let mew = shmem::team_my_pe(shmem::team_world());
    let nw = shmem::team_n_pes(shmem::team_world());

    let mes = shmem::team_my_pe(shmem::team_shared());
    let ns = shmem::team_n_pes(shmem::team_shared());

    println!(
        "{}: PE world: {} of {}: shared {} of {}",
        node, mew, nw, mes, ns
    );

    shmem::finalize();
}
