use shmemlib::*;
use state::Storage;
use std::collections::HashMap;
use std::mem;
use std::ops::Deref;
use std::ops::DerefMut;
use std::panic;
use std::string::String;
use std::sync::Mutex;

// pass through, will have to look at parsing "pub const" decls.

pub const MAJOR_VERSION: u32 = SHMEM_MAJOR_VERSION;
pub const MINOR_VERSION: u32 = SHMEM_MINOR_VERSION;
pub const VENDOR_STRING: &'static [u8; 25usize] = SHMEM_VENDOR_STRING;

pub type ThreadLevel = i32;

pub const THREAD_SINGLE: ThreadLevel = SHMEM_THREAD_SINGLE as ThreadLevel;
pub const THREAD_FUNNELED: ThreadLevel = SHMEM_THREAD_FUNNELED as ThreadLevel;
pub const THREAD_SERIALIZED: ThreadLevel = SHMEM_THREAD_SERIALIZED as ThreadLevel;
pub const THREAD_MULTIPLE: ThreadLevel = SHMEM_THREAD_MULTIPLE as ThreadLevel;

pub const SYNC_SIZE: usize = SHMEM_SYNC_SIZE as usize;
pub const BCAST_SYNC_SIZE: usize = SHMEM_BCAST_SYNC_SIZE as usize;
pub const REDUCE_MIN_WRKDATA_SIZE: usize = SHMEM_REDUCE_MIN_WRKDATA_SIZE as usize;
pub const REDUCE_SYNC_SIZE: usize = SHMEM_REDUCE_SYNC_SIZE as usize;

pub const SYNC_VALUE: i64 = SHMEM_SYNC_VALUE as i64;

pub type SymmMemAddr = *mut libc::c_void;
//pub type ShmemLock = *mut i64;

// TEAMS: don't like this, can't extend to derived teams.  just a
// stop-gap
//

// pub type TeamType = shmemlib::shmem_team_t;

// pub fn team_world() -> TeamType {
//     unsafe { shmemlib::SHMEM_TEAM_WORLD }
// }

// pub fn team_shared() -> TeamType {
//     unsafe { shmemlib::SHMEM_TEAM_SHARED }
// }

// pub fn team_invalid() -> TeamType {
//     unsafe { shmemlib::SHMEM_TEAM_INVALID }
// }

//
// == initialize and finalize ============================================
//

pub fn init() {
    unsafe {
        GM.set(Mutex::new(HashMap::new()));
        shmem_init();
    }
}

pub fn init_thread(req: ThreadLevel) -> ThreadLevel {
    unsafe {
        GM.set(Mutex::new(HashMap::new()));

        let mut prov: i32 = -1;

        shmem_init_thread(req, &mut prov);

        prov as ThreadLevel
    }
}

pub fn finalize() {
    clear();
    unsafe {
        shmem_finalize();
    }
}

//
// == Library query ======================================================
//

pub fn info_get_version() -> (u32, u32) {
    let mut a: i32 = 0;
    let mut b: i32 = 0;

    unsafe {
        shmem_info_get_version(&mut a, &mut b);
    }

    (a as u32, b as u32)
}

pub fn info_get_name() -> String {
    // unpack into UFT vector
    let vvs = SHMEM_VENDOR_STRING.to_vec();

    // turn UTF vector into string
    String::from_utf8(vvs).unwrap()
}

//
// == Global ranks =======================================================
//

pub fn my_pe() -> i32 {
    unsafe { shmem_my_pe() }
}

pub fn n_pes() -> i32 {
    unsafe { shmem_n_pes() }
}

// pub fn team_my_pe(t: shmemlib::shmem_team_t) -> i32 {
//     unsafe { shmemlib::shmem_team_my_pe(t) }
// }

// pub fn team_n_pes(t: shmemlib::shmem_team_t) -> i32 {
//     unsafe { shmemlib::shmem_team_n_pes(t) }
// }

pub fn pe_accessible(pe: i32) -> bool {
    unsafe { shmem_pe_accessible(pe) == 1 }
}

pub fn addr_accessible(addr: SymmMemAddr, pe: i32) -> bool {
    unsafe { shmem_addr_accessible(addr, pe) == 1 }
}

// etc.

// pub trait RDMAApi<T> {
//     fn p(&self, dest: *mut T, src: T, pe: i32);
// }

// impl dyn RDMAApi<i32> {
//     fn p(&self, dest: *mut i32, src: i32, pe: i32) {
//         unsafe {
//             shmemlib::shmem_int_p(dest, src, pe);
//         }
//     }
// }

//
// == memory management ==================================================
//

static GM: Storage<Mutex<HashMap<usize, usize>>> = Storage::new();

fn insert(ptr: usize, num_bytes: usize) {
    let mut map = GM.get().lock().unwrap();
    map.insert(ptr, num_bytes);
}

fn remove(ptr: usize) {
    let mut map = GM.get().lock().unwrap();

    if map.get(&ptr) != None {
        map.remove(&ptr);
        free(ptr as SymmMemAddr);
    }
}

fn clear() {
    let mut map = GM.get().lock().unwrap();

    for key in map.keys() {
        free(*key as SymmMemAddr);
    }
    map.clear();
}

fn _show() {
    println!("{}", "=".repeat(48));

    let map = GM.get().lock().unwrap();

    for (ptr, _) in map.iter() {
        println!("{}", ptr);
    }
}

fn is_aligned<T>(ptr: *mut T) -> bool {
    use core::mem::align_of;
    (ptr as usize) % align_of::<T>() == 0
}

fn validate_ptr<T>(ptr: *mut T) {
    if ptr.is_null() {
        panic!("Found null pointer");
    }

    if !is_aligned(ptr) {
        panic!("Found unaligned pointer");
    }
}

pub struct SymmMem<T> {
    ptr: *mut T,
    length: usize,
}

impl<T> SymmMem<T> {
    pub fn new(x: usize) -> SymmMem<T> {
        let num_bytes = x * mem::size_of::<T>() as usize;
        let symm_ptr = malloc(num_bytes);
        insert(symm_ptr as usize, num_bytes);
        SymmMem {
            ptr: symm_ptr as *mut T,
            length: x,
        }
    }
    // pub fn new_with_hints(x: usize, hints: u64) -> SymmMem<T> {
    //     let num_bytes = x * mem::size_of::<T>() as usize;
    //     let symm_ptr = malloc_with_hints(num_bytes, hints);
    //     insert(symm_ptr as usize, num_bytes);
    //     SymmMem { ptr: symm_ptr as *mut T, length: x }
    // }
    pub fn set(&mut self, offset: usize, value: T) {
        if offset < self.length {
            unsafe {
                validate_ptr(self.ptr);
                *(self.ptr.offset(offset as isize)) = value;
            }
        } else {
            panic!(
                "Offset is out of bounds, offset: {}, pointer length: {}",
                offset, self.length
            );
        }
    }
    pub fn get(&mut self, offset: usize) -> &T {
        if offset < self.length {
            unsafe {
                validate_ptr(self.ptr);
                return &*(self.ptr.offset(offset as isize));
            }
        } else {
            panic!(
                "Offset is out of bounds, offset: {}, pointer length: {}",
                offset, self.length
            );
        }
    }
    pub fn realloc(&mut self, new_length: usize) {
        validate_ptr(self.ptr);
        let num_bytes = mem::size_of::<T>() * new_length;
        self.ptr = realloc(self.ptr as SymmMemAddr, num_bytes) as *mut T;
        self.length = new_length;
    }
    pub fn get_ptr(&mut self) -> *mut T
    {
        self.ptr
    }
}

impl<T> Deref for SymmMem<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe {
            validate_ptr(self.ptr);
            &*self.ptr
        }
    }
}

impl<T> DerefMut for SymmMem<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            validate_ptr(self.ptr);
            &mut *self.ptr
        }
    }
}

impl<T> Drop for SymmMem<T> {
    fn drop(&mut self) {
        remove((self.ptr as SymmMemAddr) as usize);
    }
}

// so `sizeof` gives us `usize` as the amount of memory to allocate.
// this doesn't match the type that bindgen dumped out for us, so we
// have to convert

pub fn malloc(n: usize) -> SymmMemAddr {
    unsafe { shmem_malloc(n as u64) }
}

pub fn calloc(n: usize, s: usize) -> SymmMemAddr {
    unsafe { shmem_calloc(n as u64, s as u64) }
}

pub fn realloc(m: SymmMemAddr, n: usize) -> SymmMemAddr {
    unsafe { shmem_realloc(m, n as u64) }
}

pub fn align(a: u64, n: usize) -> SymmMemAddr {
    unsafe { shmem_align(a, n as u64) }
}

pub fn free(m: SymmMemAddr) {
    unsafe {
        shmem_free(m);
    }
}

// pub(crate) fn malloc_with_hints(n: usize, h: u64) -> SymmMemAddr {
//     unsafe { shmemlib::shmem_malloc_with_hints(n as u64, h as i64) }
// }

pub fn ptr(m: SymmMemAddr, pe: i32) -> SymmMemAddr {
    unsafe { shmem_ptr(m, pe) }
}

//
// == broadcast ============================================
//

pub fn broadcast32(
    target: &SymmMem<i32>,
    source: &SymmMem<i32>,
    nlong: u64,
    pe_root: i32,
    pe_start: i32,
    log_pe_stride: i32,
    pe_size: i32,
    p_sync: &SymmMem<i64>,
) {
    unsafe {
        shmem_broadcast32(
            target.ptr as *mut libc::c_void,
            source.ptr as *mut libc::c_void,
            nlong,
            pe_root,
            pe_start,
            log_pe_stride,
            pe_size,
            p_sync.ptr,
        )
    }
}

//
// == ordering and completion ============================================
//

pub fn fence() {
    unsafe {
        shmem_fence();
    }
}

pub fn quiet() {
    unsafe {
        shmem_quiet();
    }
}

pub fn barrier_all() {
    unsafe {
        shmem_barrier_all();
    }
}
//
// == locks ==============================================================
//

pub fn set_lock(lk: &SymmMem<i64>) {
    unsafe {
        shmem_set_lock(lk.ptr);
    }
}

pub fn clear_lock(lk: &SymmMem<i64>) {
    unsafe {
        shmem_clear_lock(lk.ptr);
    }
}

pub fn test_lock(lk: &SymmMem<i64>) -> bool {
    unsafe { shmem_test_lock(lk.ptr) != 0 }
}

//
// == error handling =======================================================
//
/*fn abort_on_unwind<F: FnOnce() -> R, R>(f: F) -> R {
    std::panic::catch_unwind(
        // Catching a panic will always immediately abort the program, so there is never a chance
        // that any non-UnwindSafe value will be observed afterwards.
        std::panic::AssertUnwindSafe(f),
    )
    .unwrap_or_else(|_| {
        println!("Error unwinding across FFI boundary");
        std::process::abort;
    })
}*/

pub trait SymmMemTrait<T> {
    /* puts and gets */
    fn put(&mut self, dest: &SymmMem<T>, n: u64, pe: i32);
    fn p(&mut self, src: T, pe: i32);
    fn put_nbi(&mut self, dest: &SymmMem<T>, n: u64, pe: i32);
    fn putmem(&mut self, _dest: &SymmMem<T>, _len: size_t, _pe: i32) {}
    fn get_values(&mut self, src: &SymmMem<T>, n: u64, pe: i32);
    fn g(&mut self, pe: i32) -> T;
    fn get_nbi(&mut self, src: &SymmMem<T>, n: u64, pe: i32);
    fn getmem(&mut self, _src: &SymmMem<T>, _len: size_t, _pe: i32) {}

    /* atomics */
    fn atomic_fetch_add(&mut self, _val: T, _pe: i32) -> i32 {
        return 0;
    }
    fn atomic_add(&mut self, _val: T, _pe: i32) {}

    /* collectives */
    fn sum_to_all(
        &mut self,
        _target: &SymmMem<T>,
        _nreduce: i32,
        _start: i32,
        _stride: i32,
        _size: i32,
        _pwrk: &SymmMem<T>,
        _psync: &SymmMem<i64>,
    ) {}
}

impl SymmMemTrait<i32> for SymmMem<i32> {
    /* puts and gets */
    fn p(&mut self, src: i32, pe: i32) {
        unsafe { shmem_int_p(self.ptr, src, pe) }
    }
    fn put(&mut self, dest: &SymmMem<i32>, n: u64, pe: i32) {
        unsafe { shmem_int_put(dest.ptr, self.ptr, n, pe) }
    }
    fn put_nbi(&mut self, dest: &SymmMem<i32>, n: u64, pe: i32) {
        unsafe { shmem_int_put_nbi(dest.ptr, self.ptr, n, pe) }
    }
    fn get_values(&mut self, src: &SymmMem<i32>, n: u64, pe: i32) {
        unsafe { shmem_int_get(self.ptr, src.ptr, n, pe) }
    }
    fn g(&mut self, pe: i32) -> i32 {
        unsafe { shmem_int_g(self.ptr, pe) }
    }
    fn get_nbi(&mut self, src: &SymmMem<i32>, n: u64, pe: i32) {
        unsafe { shmem_int_get_nbi(self.ptr, src.ptr, n, pe) }
    }

    /* atomics */
    fn atomic_fetch_add(&mut self, val: i32, pe: i32) -> i32 {
        unsafe { shmem_int_atomic_fetch_add(self.ptr, val, pe) }
    }
    fn atomic_add(&mut self, val: i32, pe: i32) {
        unsafe { shmem_int_atomic_add(self.ptr, val, pe) }
    }

    /* collectives */
    fn sum_to_all(
        &mut self,
        target: &SymmMem<i32>,
        nreduce: i32,
        start: i32,
        stride: i32,
        size: i32,
        pwrk: &SymmMem<i32>,
        psync: &SymmMem<i64>,
    ) {
        unsafe {
            shmem_int_sum_to_all(
                target.ptr, self.ptr, nreduce, start, stride, size, pwrk.ptr, psync.ptr,
            );
        }
    }
}

impl SymmMemTrait<f32> for SymmMem<f32> {
    /* puts and gets */
    fn p(&mut self, src: f32, pe: i32) {
        unsafe { shmem_float_p(self.ptr, src, pe) }
    }
    fn put(&mut self, dest: &SymmMem<f32>, n: u64, pe: i32) {
        unsafe { shmem_float_put(dest.ptr, self.ptr, n, pe) }
    }
    fn put_nbi(&mut self, dest: &SymmMem<f32>, n: u64, pe: i32) {
        unsafe { shmem_float_put_nbi(dest.ptr, self.ptr, n, pe) }
    }
    fn get_values(&mut self, src: &SymmMem<f32>, n: u64, pe: i32) {
        unsafe { shmem_float_get(self.ptr, src.ptr, n, pe) }
    }
    fn g(&mut self, pe: i32) -> f32 {
        unsafe { shmem_float_g(self.ptr, pe) }
    }
    fn get_nbi(&mut self, src: &SymmMem<f32>, n: u64, pe: i32) {
        unsafe { shmem_float_get_nbi(self.ptr, src.ptr, n, pe) }
    }

    /* collectives */
    fn sum_to_all(
        &mut self,
        target: &SymmMem<f32>,
        nreduce: i32,
        start: i32,
        stride: i32,
        size: i32,
        pwrk: &SymmMem<f32>,
        psync: &SymmMem<i64>,
    ) {
        unsafe {
            shmem_float_sum_to_all(
                target.ptr, self.ptr, nreduce, start, stride, size, pwrk.ptr, psync.ptr,
            );
        }
    }
}

impl SymmMemTrait<f64> for SymmMem<f64> {
    /* puts and gets */
    fn p(&mut self, src: f64, pe: i32) {
        unsafe { shmem_double_p(self.ptr, src, pe) }
    }
    fn put(&mut self, dest: &SymmMem<f64>, n: u64, pe: i32) {
        unsafe { shmem_double_put(dest.ptr, self.ptr, n, pe) }
    }
    fn put_nbi(&mut self, dest: &SymmMem<f64>, n: u64, pe: i32) {
        unsafe { shmem_double_put_nbi(dest.ptr, self.ptr, n, pe) }
    }
    fn get_values(&mut self, src: &SymmMem<f64>, n: u64, pe: i32) {
        unsafe { shmem_double_get(self.ptr, src.ptr, n, pe) }
    }
    fn g(&mut self, pe: i32) -> f64 {
        unsafe { shmem_double_g(self.ptr, pe) }
    }
    fn get_nbi(&mut self, src: &SymmMem<f64>, n: u64, pe: i32) {
        unsafe { shmem_double_get_nbi(self.ptr, src.ptr, n, pe) }
    }

    /* collectives */
    fn sum_to_all(
        &mut self,
        target: &SymmMem<f64>,
        nreduce: i32,
        start: i32,
        stride: i32,
        size: i32,
        pwrk: &SymmMem<f64>,
        psync: &SymmMem<i64>,
    ) {
        unsafe {
            shmem_double_sum_to_all(
                target.ptr, self.ptr, nreduce, start, stride, size, pwrk.ptr, psync.ptr,
            );
        }
    }
}

impl SymmMemTrait<u8> for SymmMem<u8> {
    /* puts and gets */
    fn p(&mut self, src: u8, pe: i32) {
        unsafe { shmem_char_p(self.ptr, src, pe) }
    }
    fn put(&mut self, dest: &SymmMem<u8>, n: u64, pe: i32) {
        unsafe { shmem_char_put(dest.ptr, self.ptr, n, pe) }
    }
    fn put_nbi(&mut self, dest: &SymmMem<u8>, n: u64, pe: i32) {
        unsafe { shmem_char_put_nbi(dest.ptr, self.ptr, n, pe) }
    }
    fn get_values(&mut self, src: &SymmMem<u8>, n: u64, pe: i32) {
        unsafe { shmem_char_get(self.ptr, src.ptr, n, pe) }
    }
    fn g(&mut self, pe: i32) -> u8 {
        unsafe { shmem_char_g(self.ptr, pe) }
    }
    fn get_nbi(&mut self, src: &SymmMem<u8>, n: u64, pe: i32) {
        unsafe { shmem_char_get_nbi(self.ptr, src.ptr, n, pe) }
    }
}

impl SymmMem<libc::c_void> {
    pub fn putmem(&mut self, dest: &SymmMem<libc::c_void>, n: u64, pe: i32) {
        unsafe {
            shmem_putmem(dest.ptr, self.ptr, n, pe);
        }
    }
    pub fn getmem(&mut self, src: &SymmMem<libc::c_void>, n: u64, pe: i32) {
        unsafe {
            shmem_getmem(self.ptr, src.ptr, n, pe);
        }
    }
}
