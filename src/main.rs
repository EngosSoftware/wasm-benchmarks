mod utils;

use crate::utils::human_friendly_seconds;
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

fn bench_table_grow(initial: i32, grow: i32, iterations: u64) -> (u64, u64, u64) {
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
  utils::norm(ticks)
}

/// Initial table size.
#[rustfmt::skip]
const INITIAL: &[i32] = &[
  1, 10, 100, 1_000, 10_000, 100_000, 1_000_000, 10_000_000
];

/// Grow sizes.
#[rustfmt::skip]
const GROW: &[i32] = &[
  0, 1, 2, 5,
  10, 20, 30, 50, 60, 70, 80, 90,
  100, 200, 300, 400, 500, 600, 700, 800, 900,
  1_000, 2_000, 3_000, 4_000, 5_000, 6_000, 7_000, 8_000, 9_000,
  10_000, 20_000, 30_000, 40_000, 50_000, 60_000, 70_000, 80_000, 90_000,
  100_000, 200_000, 300_000, 400_000, 500_000, 600_000, 700_000, 800_000, 900_000,
  1_000_000, 2_000_000, 3_000_000, 4_000_000, 5_000_000, 6_000_000, 7_000_000, 8_000_000, 9_000_000,
  10_000_000, 20_000_000, 30_000_000, 40_000_000, 50_000_000, 60_000_000, 70_000_000, 80_000_000, 90_000_000,
  100_000_000, 200_000_000, 300_000_000, 400_000_000, 500_000_000, 600_000_000, 700_000_000, 800_000_000, 900_000_000,
  1_000_000_000,
];

/// Minimum number of samples.
const MIN_SAMPLES: u64 = 20;

/// Maximum number of samples.
const MAX_SAMPLES: u64 = 1000;

/// Maximum measurement time in seconds.
#[cfg(target_os = "macos")]
const MEASUREMENT_TIME: u64 = 1; // ca. 3 samples inside
#[cfg(target_os = "linux")]
const MEASUREMENT_TIME: u64 = 10; // ca. 3 samples inside

fn main() {
  core_affinity::set_for_current(core_affinity::get_core_ids().unwrap()[1]);
  for initial in INITIAL {
    for grow in GROW {
      let (_, time_nanos, _) = bench_table_grow(*initial, *grow, 5);
      let iterations = (MEASUREMENT_TIME * 1_000_000_000 / time_nanos).clamp(MIN_SAMPLES, MAX_SAMPLES);
      let (low, mid, high) = bench_table_grow(*initial, *grow, iterations);
      let low_gas = low * 1_000;
      let mid_gas = mid * 1_000;
      let high_gas = high * 1_000;
      println!(
        "{:14} {:14} {:14} {:14} {:14} {:14} {:>14}",
        initial,
        grow,
        iterations,
        low_gas,
        mid_gas,
        high_gas,
        human_friendly_seconds(mid)
      );
    }
  }
}
