#define BENCHMARK "OpenSHMEM Put Loop Test Using OSU Benchmarks"

#include <shmem.h>
#include <osu_util_pgas.h>

int main(int argc, char *argv[])
{
    int me, npes, num_elems;
    int *s_buf, *r_buf;
    double t_start = 0, t_end = 0;
    int NUM_ELEMS = 10000;
    int NUM_ITERS = 100000;

    shmem_init();
    me = shmem_my_pe(); 
    npes = shmem_n_pes();

    int nextpe = (me + 1) % npes;
        
	s_buf = (int *)shmem_malloc(NUM_ELEMS * sizeof(int));
    r_buf = (int *)shmem_malloc(NUM_ELEMS * sizeof(int));

    for(int i = 0; i < NUM_ELEMS; i++)
    {
        s_buf[i] = i * (me + 1);
    }

    shmem_barrier_all();

    t_start = TIME();

    for(int i = 0; i < NUM_ITERS; i++)
    {
        shmem_put(r_buf, s_buf, NUM_ELEMS, nextpe);
    }

    shmem_barrier_all();

    t_end = TIME();

    if(me == 0)
    {
        for(int i = 0; i < 5; i++)
        {
            fprintf(stdout, "PE %d/%d, idx %d, value = %d\n", me, npes, i, r_buf[i]);
        }
    
        fprintf(stdout, "Elapsed Time: %lf ms\n", (t_end - t_start) * 0.001);
    }

    shmem_free(s_buf);
    //shmem_free(r_buf);

    shmem_barrier_all();
    shmem_finalize();

    return EXIT_SUCCESS;
}
