use shmem::*;
use uname::uname;
use std::env;

struct pe_vars {
    me: i32,
    npes: i32,
    pairs: i32,
    nxtpe: i32,
}

#[repr(C)]
union global_msg_buffer {
    int: i32;
    long: i64;
    longlong: i128;
    float: f32;
    double: f64:
}

fn main() {
    const MEMORY_SELECTION: i32 = 1;

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

}

double pwrk1[_SHMEM_REDUCE_MIN_WRKDATA_SIZE];
double pwrk2[_SHMEM_REDUCE_MIN_WRKDATA_SIZE];

long psync1[_SHMEM_REDUCE_SYNC_SIZE];
long psync2[_SHMEM_REDUCE_SYNC_SIZE];

struct pe_vars
init_openshmem (void)
{
    struct pe_vars v;

    v.pairs = v.npes / 2;
    v.nxtpe = v.me < v.pairs ? v.me + v.pairs : v.me - v.pairs;

    return v;
}

static void
print_usage (int myid)
{
    if (myid == 0) {
        if (MEMORY_SELECTION) {
            fprintf(stderr, "Usage: osu_oshm_atomics <heap|global>\n");
        }

        else {
            fprintf(stderr, "Usage: osu_oshm_atomics\n");
        }
    }
}

void
check_usage (int me, int npes, int argc, char * argv [])
{
    if (MEMORY_SELECTION) {
        if (2 == argc) {
            /*
             * Compare more than 4 and 6 characters respectively to make sure
             * that we're not simply matching a prefix but the entire string.
             */
            if (strncmp(argv[1], "heap", 10)
                    && strncmp(argv[1], "global", 10)) {
                print_usage(me);
                exit(EXIT_FAILURE);
            }
        }

        else {
            print_usage(me);
            exit(EXIT_FAILURE);
        }
    }

    if (2 > npes) {
        if (0 == me) {
            fprintf(stderr, "This test requires at least two processes\n");
        }

        exit(EXIT_FAILURE);
    }
}

void
print_header_local (int myid)
{
    if (myid == 0) {
        fprintf(stdout, HEADER);
        fprintf(stdout, "%-*s%*s%*s\n", 20, "# Operation", FIELD_WIDTH,
                "Million ops/s", FIELD_WIDTH, "Latency (us)");
        fflush(stdout);
    }
}

union data_types *
allocate_memory (int me, int use_heap)
{
    union data_types * msg_buffer;

    if (!use_heap) {
        return global_msg_buffer;
    }

#ifdef OSHM_1_3
    msg_buffer = (union data_types *)shmem_malloc(sizeof(union data_types
                [OSHM_LOOP_ATOMIC]));
#else
	msg_buffer = (union data_types *)shmalloc(sizeof(union data_types
                [OSHM_LOOP_ATOMIC]));
#endif
				
    if (NULL == msg_buffer) {
        fprintf(stderr, "Failed to shmalloc (pe: %d)\n", me);
        exit(EXIT_FAILURE);
    }

    return msg_buffer;
}

void
print_operation_rate (int myid, char * operation, double rate, double lat)
{
    if (myid == 0) {
        fprintf(stdout, "%-*s%*.*f%*.*f\n", 20, operation, FIELD_WIDTH,
                FLOAT_PRECISION, rate, FIELD_WIDTH, FLOAT_PRECISION, lat);
        fflush(stdout);
    }
}

double
benchmark_fadd (struct pe_vars v, union data_types *buffer,
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
        int value = 1;
        int old_value;

        begin = TIME();
        for (i = 0; i < iterations; i++) { 
            old_value = shmem_int_fadd(&(buffer[i].int_type), value, v.nxtpe);
        }
        end = TIME();

        rate = ((double)iterations * 1e6) / (end - begin);
        lat = (end - begin) / (double)iterations;
    }

    shmem_double_sum_to_all(&sum_rate, &rate, 1, 0, 0, v.npes, pwrk1, psync1);
    shmem_double_sum_to_all(&sum_lat, &lat, 1, 0, 0, v.npes, pwrk2, psync2);    
    print_operation_rate(v.me, "shmem_int_fadd", sum_rate/1e6, sum_lat/v.pairs);

    return 0;
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
benchmark_finc (struct pe_vars v, union data_types *buffer,
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
        int old_value;

        begin = TIME();
        for (i = 0; i < iterations; i++) {
            old_value = shmem_int_finc(&(buffer[i].int_type), v.nxtpe);
        }
        end = TIME();

        rate = ((double)iterations * 1e6) / (end - begin);
        lat = (end - begin) / (double)iterations;
    }

    shmem_double_sum_to_all(&sum_rate, &rate, 1, 0, 0, v.npes, pwrk1, psync1);
    shmem_double_sum_to_all(&sum_lat, &lat, 1, 0, 0, v.npes, pwrk2, psync2);
    print_operation_rate(v.me, "shmem_int_finc", sum_rate/1e6, sum_lat/v.pairs);

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
benchmark_add (struct pe_vars v, union data_types *buffer,
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
        int value = INT_MAX * drand48();

        begin = TIME();
        for (i = 0; i < iterations; i++) {
            shmem_int_add(&(buffer[i].int_type), value, v.nxtpe);
        }
        end = TIME();

        rate = ((double)iterations * 1e6) / (end - begin);
        lat = (end - begin) / (double)iterations;
    }

    shmem_double_sum_to_all(&sum_rate, &rate, 1, 0, 0, v.npes, pwrk1, psync1);
    shmem_double_sum_to_all(&sum_lat, &lat, 1, 0, 0, v.npes, pwrk2, psync2);
    print_operation_rate(v.me, "shmem_int_add", sum_rate/1e6, sum_lat/v.pairs);

    return 0;
}

double
benchmark_add_longlong (struct pe_vars v, union data_types *buffer,
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

        begin = TIME();
        for (i = 0; i < iterations; i++) {
            shmem_longlong_add(&(buffer[i].longlong_type), value, v.nxtpe);
        }
        end = TIME();

        rate = ((double)iterations * 1e6) / (end - begin);
        lat = (end - begin) / (double)iterations;
    }

    shmem_double_sum_to_all(&sum_rate, &rate, 1, 0, 0, v.npes, pwrk1, psync1);
    shmem_double_sum_to_all(&sum_lat, &lat, 1, 0, 0, v.npes, pwrk2, psync2);
    print_operation_rate(v.me, "shmem_longlong_add", sum_rate/1e6, sum_lat/v.pairs);

    return 0;
}

double
benchmark_inc (struct pe_vars v, union data_types *buffer,
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
            shmem_int_inc(&(buffer[i].int_type), v.nxtpe);
        }
        end = TIME();

        rate = ((double)iterations * 1e6) / (end - begin);
        lat = (end - begin) / (double)iterations;
    }

    shmem_double_sum_to_all(&sum_rate, &rate, 1, 0, 0, v.npes, pwrk1, psync1);
    shmem_double_sum_to_all(&sum_lat, &lat, 1, 0, 0, v.npes, pwrk2, psync2);
    print_operation_rate(v.me, "shmem_int_inc", sum_rate/1e6, sum_lat/v.pairs);

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
benchmark_swap (struct pe_vars v, union data_types *buffer,
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
        int value = INT_MAX * drand48();
        int old_value;

        begin = TIME();
        for (i = 0; i < iterations; i++) {
            old_value = shmem_int_swap(&(buffer[i].int_type), value, v.nxtpe);
        }
        end = TIME();

        rate = ((double)iterations * 1e6) / (end - begin);
        lat = (end - begin) / (double)iterations;        
    }

    shmem_double_sum_to_all(&sum_rate, &rate, 1, 0, 0, v.npes, pwrk1, psync1);
    shmem_double_sum_to_all(&sum_lat, &lat, 1, 0, 0, v.npes, pwrk2, psync2);
    print_operation_rate(v.me, "shmem_int_swap", sum_rate/1e6, sum_lat/v.pairs);

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
benchmark_cswap (struct pe_vars v, union data_types *buffer,
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
        int cond = v.nxtpe;
        int value = INT_MAX * drand48();
        int old_value;

        begin = TIME();
        for (i = 0; i < iterations; i++) {
            old_value = shmem_int_cswap(&(buffer[i].int_type), cond, value, v.nxtpe);
        }
        end = TIME();
		
        rate = ((double)iterations * 1e6) / (end - begin);
        lat = (end - begin) / (double)iterations;        
    }
	
    shmem_double_sum_to_all(&sum_rate, &rate, 1, 0, 0, v.npes, pwrk1, psync1);
    shmem_double_sum_to_all(&sum_lat, &lat, 1, 0, 0, v.npes, pwrk2, psync2);
    print_operation_rate(v.me, "shmem_int_cswap", sum_rate/1e6, sum_lat/v.pairs);

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
benchmark_set (struct pe_vars v, union data_types *buffer,
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
        int value = 1;

        begin = TIME();
        for (i = 0; i < iterations; i++) { 
            shmem_int_set(&(buffer[i].int_type), value, v.nxtpe);
        }
        end = TIME();

        rate = ((double)iterations * 1e6) / (end - begin);
        lat = (end - begin) / (double)iterations;
    }

    shmem_double_sum_to_all(&sum_rate, &rate, 1, 0, 0, v.npes, pwrk1, psync1);
    shmem_double_sum_to_all(&sum_lat, &lat, 1, 0, 0, v.npes, pwrk2, psync2);    
    print_operation_rate(v.me, "shmem_int_set", sum_rate/1e6, sum_lat/v.pairs);

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

int
main (int argc, char *argv[])
{
    int i;
    struct pe_vars v;
    union data_types * msg_buffer;
    int use_heap;

    /*
     * Initialize
     */
    v = init_openshmem();
    check_usage(v.me, v.npes, argc, argv);

    for (i = 0; i < _SHMEM_REDUCE_SYNC_SIZE; i += 1) {
        psync1[i] = _SHMEM_SYNC_VALUE;
        psync2[i] = _SHMEM_SYNC_VALUE;
    }
    shmem_barrier_all();

    print_header_local(v.me);

    /*
     * Allocate Memory
     */
    use_heap = !strncmp(argv[1], "heap", 10);
    msg_buffer = allocate_memory(v.me, use_heap);
    memset(msg_buffer, 0, sizeof(union data_types [OSHM_LOOP_ATOMIC]));

    /*
     * Time Put Message Rate
     */
    benchmark(v, msg_buffer);

    /*
     * Finalize
     */
    if (use_heap) { 
#ifdef OSHM_1_3
        shmem_free(msg_buffer);
#else 
        shfree(msg_buffer);
#endif
    }

#ifdef OSHM_1_3  	
	shmem_finalize();
#endif

    return EXIT_SUCCESS;
}
