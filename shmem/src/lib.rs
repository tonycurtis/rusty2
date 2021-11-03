use shmemlib;
use std::string::String;
use std::mem;
use std::ops::Deref;
use std::ops::DerefMut;
use std::collections::HashMap;
use std::sync::Mutex;
use state::Storage;

// pass through, will have to look at parsing "pub const" decls.

pub const MAJOR_VERSION: u32 = shmemlib::SHMEM_MAJOR_VERSION;
pub const MINOR_VERSION: u32 = shmemlib::SHMEM_MINOR_VERSION;
pub const VENDOR_STRING: &'static [u8; 9usize] = shmemlib::SHMEM_VENDOR_STRING;

pub type ThreadLevel = i32;

pub const THREAD_SINGLE: ThreadLevel = shmemlib::SHMEM_THREAD_SINGLE as ThreadLevel;
pub const THREAD_FUNNELED: ThreadLevel = shmemlib::SHMEM_THREAD_FUNNELED as ThreadLevel;
pub const THREAD_SERIALIZED: ThreadLevel = shmemlib::SHMEM_THREAD_SERIALIZED as ThreadLevel;
pub const THREAD_MULTIPLE: ThreadLevel = shmemlib::SHMEM_THREAD_MULTIPLE as ThreadLevel;

pub const SYNC_SIZE: usize = shmemlib::SHMEM_SYNC_SIZE as usize;
pub const BCAST_SYNC_SIZE: usize = shmemlib::SHMEM_BCAST_SYNC_SIZE as usize;
pub const REDUCE_MIN_WRKDATA_SIZE: usize = shmemlib::SHMEM_REDUCE_MIN_WRKDATA_SIZE as usize;

pub const SYNC_VALUE: i64 = shmemlib::SHMEM_SYNC_VALUE as i64;

pub type SymmMemAddr = *mut libc::c_void;
//pub type ShmemLock = *mut i64;

// TEAMS: don't like this, can't extend to derived teams.  just a
// stop-gap
//

pub type TeamType = shmemlib::shmem_team_t;

pub fn team_world() -> TeamType {
    unsafe { shmemlib::SHMEM_TEAM_WORLD }
}

pub fn team_shared() -> TeamType {
    unsafe { shmemlib::SHMEM_TEAM_SHARED }
}

pub fn team_invalid() -> TeamType {
    unsafe { shmemlib::SHMEM_TEAM_INVALID }
}

//
// == initialize and finalize ============================================
//

pub fn init() {
    unsafe {
        GM.set(Mutex::new(HashMap::new()));
        shmemlib::shmem_init();
    }
}

pub fn init_thread(req: ThreadLevel) -> ThreadLevel {
    unsafe {
        let mut prov: i32 = -1;

        shmemlib::shmem_init_thread(req, &mut prov);

        prov as ThreadLevel
    }
}

pub fn finalize() {
    clear();
    unsafe {
        shmemlib::shmem_finalize();
    }
}

//
// == Library query ======================================================
//

pub fn info_get_version() -> (u32, u32) {
    let mut a: i32 = 0;
    let mut b: i32 = 0;

    unsafe {
        shmemlib::shmem_info_get_version(&mut a, &mut b);
    }

    (a as u32, b as u32)
}

pub fn info_get_name() -> String {
    // unpack into UFT vector
    let vvs = shmemlib::SHMEM_VENDOR_STRING.to_vec();

    // turn UTF vector into string
    String::from_utf8(vvs).unwrap()
}

//
// == Global ranks =======================================================
//

pub fn my_pe() -> i32 {
    unsafe { shmemlib::shmem_my_pe() }
}

pub fn n_pes() -> i32 {
    unsafe { shmemlib::shmem_n_pes() }
}

pub fn team_my_pe(t: shmemlib::shmem_team_t) -> i32 {
    unsafe { shmemlib::shmem_team_my_pe(t) }
}

pub fn team_n_pes(t: shmemlib::shmem_team_t) -> i32 {
    unsafe { shmemlib::shmem_team_n_pes(t) }
}

pub fn pe_accessible(pe: i32) -> bool {
    unsafe { shmemlib::shmem_pe_accessible(pe) == 1 }
}

pub fn addr_accessible(addr: SymmMemAddr, pe: i32) -> bool {
    unsafe { shmemlib::shmem_addr_accessible(addr, pe) == 1 }
}

//
// == puts and gets ======================================================
//

pub fn int_p(dest: &SymmMem<i32>, src: i32, pe: i32) {
    unsafe {
        shmemlib::shmem_int_p(dest.ptr, src, pe);
    }
}

pub fn int_put(dest: &SymmMem<i32>, src: &SymmMem<i32>, n: u64, pe: i32) {
    unsafe {
        shmemlib::shmem_int_put(dest.ptr, src.ptr, n, pe);
    }
}

pub fn int_put_nbi(dest: &SymmMem<i32>, src: &SymmMem<i32>, n: u64, pe: i32) {
    unsafe {
        shmemlib::shmem_int_put_nbi(dest.ptr, src.ptr, n, pe);
    }
}

pub fn float_p(dest: &SymmMem<f32>, src: f32, pe: i32) {
    unsafe {
        shmemlib::shmem_float_p(dest.ptr, src, pe);
    }
}

pub fn float_put(dest: &SymmMem<f32>, src: &SymmMem<f32>, n: u64, pe: i32) {
    unsafe {
        shmemlib::shmem_float_put(dest.ptr, src.ptr, n, pe);
    }
}

// etc.

pub fn int_g(dest: &SymmMem<i32>, pe: i32) -> i32 {
    unsafe {
        shmemlib::shmem_int_g(dest.ptr, pe)
    }
}

pub fn int_get(dest: &SymmMem<i32>, src: &SymmMem<i32>, n: u64, pe: i32) {
    unsafe {
        shmemlib::shmem_int_get(dest.ptr, src.ptr, n, pe);
    }
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

    if map.get(&ptr) != None
    {
        map.remove(&ptr);
        free(ptr as SymmMemAddr);
    }
}

fn clear()
{
   let mut map = GM.get().lock().unwrap();

    for key in map.keys() {
        free(*key as SymmMemAddr);
    }  
    map.clear();
}

fn show() {
    println!("{}", "=".repeat(48));

    let map = GM.get().lock().unwrap();

    for(ptr, _) in map.iter() {
        println!("{}", ptr);
    }
}

fn validate_ptr<T>(ptr: *mut T, num_bytes: usize) {
    if ptr.is_null() {
        panic!("Found null pointer");
    }
    
    if (ptr as usize) % num_bytes != 0 {
        panic!("Found unaligned pointer");
    }
}

pub struct SymmMem<T> {
    ptr: *mut T,
    length: usize
}

impl<T> SymmMem<T> {
    pub fn new(x: usize) -> SymmMem<T> {
        let num_bytes = x * mem::size_of::<T>() as usize;
        let symm_ptr = malloc(num_bytes);
        insert(symm_ptr as usize, num_bytes);
        SymmMem { ptr: symm_ptr as *mut T, length: x }
    }
    pub fn new_with_hints(x: usize, hints: u64) -> SymmMem<T> {
        let num_bytes = x * mem::size_of::<T>() as usize;
        let symm_ptr = malloc_with_hints(num_bytes, hints);
        insert(symm_ptr as usize, num_bytes);
        SymmMem { ptr: symm_ptr as *mut T, length: x }
    }
    pub fn set(&mut self, offset: usize, value: T) {
        if offset < self.length {
            unsafe {
                *(self.ptr.offset((offset as isize))) = value;
            }
        }
        else {
            panic!("Offset is out of bounds, offset: {}, pointer length: {}", offset, self.length);
        }
    }
    pub fn get(&mut self, offset: usize) -> &T {
        if offset < self.length {
            unsafe {
                return &*(self.ptr.offset((offset as isize)));
            }
        }
        else {
            panic!("Offset is out of bounds, offset: {}, pointer length: {}", offset, self.length);
        }
    }
    pub fn realloc(&mut self, newLength: usize) {
        let num_bytes = mem::size_of::<T>() * newLength;
        self.ptr = realloc(self.ptr as SymmMemAddr, num_bytes) as *mut T;
        self.length = newLength;
    }
}

impl<T> Deref for SymmMem<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe {
            &*self.ptr
        }
    }
}

impl<T> DerefMut for SymmMem<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
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

pub(crate) fn malloc(n: usize) -> SymmMemAddr {
    unsafe { shmemlib::shmem_malloc(n as u64) }
}

pub(crate) fn calloc(n: usize, s: usize) -> SymmMemAddr {
    unsafe { shmemlib::shmem_calloc(n as u64, s as u64) }
}

pub(crate) fn realloc(m: SymmMemAddr, n: usize) -> SymmMemAddr {
    unsafe { shmemlib::shmem_realloc(m, n as u64) }
}

pub(crate) fn align(a: u64, n: usize) -> SymmMemAddr {
    unsafe { shmemlib::shmem_align(a, n as u64) }
}

pub(crate) fn free(m: SymmMemAddr) {
    unsafe {
        shmemlib::shmem_free(m);
    }
}

pub(crate) fn malloc_with_hints(n: usize, h: u64) -> SymmMemAddr {
    unsafe { shmemlib::shmem_malloc_with_hints(n as u64, h as i64) }
}

pub fn ptr(m: SymmMemAddr, pe: i32) -> SymmMemAddr {
    unsafe { shmemlib::shmem_ptr(m, pe) }
}

//
// == ordering and completion ============================================
//

pub fn fence() {
    unsafe {
        shmemlib::shmem_fence();
    }
}

pub fn quiet() {
    unsafe {
        shmemlib::shmem_quiet();
    }
}

pub fn barrier_all() {
    unsafe {
        shmemlib::shmem_barrier_all();
    }
}

//
// == atomics ============================================================
//

pub fn int_atomic_add(dest: &SymmMem<i32>, val: i32, pe: i32) {
    unsafe {
        shmemlib::shmem_int_atomic_add(dest.ptr, val,pe);
    }
}

pub fn int_atomic_fetch_add(dest: &SymmMem<i32>, val: i32, pe: i32) -> i32 {
    unsafe {
        shmemlib::shmem_int_atomic_fetch_add(dest.ptr, val,pe)
    }
}

// and so on for other types

//
// == locks ==============================================================
//

pub fn set_lock(lk: &SymmMem<i64>) {
    unsafe {
        shmemlib::shmem_set_lock(lk.ptr);
    }
}

pub fn clear_lock(lk: &SymmMem<i64>) {
    unsafe {
        shmemlib::shmem_clear_lock(lk.ptr);
    }
}

pub fn test_lock(lk: &SymmMem<i64>) -> bool {
    unsafe {
        shmemlib::shmem_test_lock(lk.ptr) != 0
    }
}

//
// == collectivess =======================================================
//

pub fn int_sum_to_all(target: &SymmMem<i32>, source: &SymmMem<i32>,
                      nreduce: i32,
                      start: i32, stride: i32, size: i32,
                      pwrk: &SymmMem<i32>, psync: &SymmMem<i64>) {
    unsafe {
        shmemlib::shmem_int_sum_to_all(target.ptr, source.ptr,
                                       nreduce,
                                       start, stride, size,
                                       pwrk.ptr, psync.ptr);
    }
}

pub fn double_sum_to_all(target: &SymmMem<f64>, source: &SymmMem<f64>,
                      nreduce: i32,
                      start: i32, stride: i32, size: i32,
                      pwrk: &SymmMem<f64>, psync: &SymmMem<i64>) {
    unsafe {
        shmemlib::shmem_double_sum_to_all(target.ptr, source.ptr,
                                          nreduce,
                                          start, stride, size,
                                          pwrk.ptr, psync.ptr);
    }
}

// and so on for other types
