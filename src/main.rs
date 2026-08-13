mod norm;

use std::alloc::{GlobalAlloc, Layout, System};

struct CopyingRealloc;

unsafe impl GlobalAlloc for CopyingRealloc {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    unsafe { System.alloc(layout) }
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    unsafe { System.dealloc(ptr, layout) }
  }

  unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
    unsafe { System.alloc_zeroed(layout) }
  }

  unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
    unsafe {
      let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());
      let new_ptr = System.alloc(new_layout);
      if !new_ptr.is_null() {
        std::ptr::copy_nonoverlapping(ptr, new_ptr, layout.size().min(new_size));
        System.dealloc(ptr, layout);
      }
      new_ptr
    }
  }
}

#[global_allocator]
static ALLOC: CopyingRealloc = CopyingRealloc;

const TEMPLATE: &str = r#"
(module
  (table (export "tab") <INITIAL> funcref)
  (elem func $f1)
  (func $f1)
  (func (export "fun") (result i32)
    ref.func $f1
    i32.const <GROW>
    table.grow 0
  )
)
"#;

fn wat_source(initial: i32, grow: i32) -> String {
  TEMPLATE.replace("<INITIAL>", &initial.to_string()).replace("<GROW>", &grow.to_string())
}

fn bench_table_grow(initial: i32, grow: i32, iterations: usize) {
  assert!(iterations > 0);
  let clock = quanta::Clock::new();
  let wasm_bytes = wat::parse_str(wat_source(initial, grow)).unwrap();
  let store = wasmer::Store::new(wasmer::sys::Singlepass::default());
  let module = wasmer::Module::from_binary(&store, &wasm_bytes).unwrap();
  let mut ticks = vec![];
  for _ in 1..=iterations {
    let compiler = wasmer::sys::Singlepass::default();
    let mut store = wasmer::Store::new(compiler);
    let instance = wasmer::Instance::new(&mut store, &module, &wasmer::imports! {}).unwrap();
    let fun = instance.exports.get_typed_function::<(), i32>(&store, "fun").unwrap();
    let start = clock.raw();
    let size = fun.call(&mut store).unwrap();
    let end = clock.raw();
    assert_eq!(initial, size);
    ticks.push(clock.delta(start, end).as_nanos() as u64);
  }
  let time_nanos = norm::calc(ticks);
  println!("initial = {}, grow = {}, time = {}", initial, grow, time_nanos);
}

fn main() {
  core_affinity::set_for_current(core_affinity::get_core_ids().unwrap()[1]);
  let args = std::env::args().skip(1).collect::<Vec<String>>();
  if args.len() != 3 {
    eprintln!("invalid number of arguments");
    return;
  }
  let initial = args[0].replace("_", "").parse::<i32>().unwrap();
  let grow = args[1].replace("_", "").parse::<i32>().unwrap();
  let iterations = args[2].replace("_", "").parse::<usize>().unwrap();
  bench_table_grow(initial, grow, iterations);
}
