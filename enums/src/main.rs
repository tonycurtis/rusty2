use std::fmt::Display;
// use enum_dispatch::enum_dispatch;

struct Int {
    v: i32,
}

trait RDMAOps<T: Display> {
    fn p(&self, v: T);
}

impl dyn RDMAOps<Int> {
    fn p(&self, v: Int) {
        println!("INT   {}", self.v);
    }
}

fn main() {
    p(55);
}
