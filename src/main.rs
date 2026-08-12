use medians::*;

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

fn wat_source(initial: usize, grow: usize) -> String {
  TEMPLATE.replace("<INITIAL>", &initial.to_string()).replace("<GROW>", &grow.to_string())
}

fn bench_table_grow(initial: usize, grow: usize, iterations: usize) {
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
    _ = fun.call(&mut store);
    let end = clock.raw();
    ticks.push(clock.delta(start, end).as_nanos() as u64);
  }
  let median = match medianu64(&mut ticks).unwrap() {
    Medians::Odd(m) => *m,
    Medians::Even((l, r)) => (l + r) / 2,
  };
  println!("initial = {}, grow = {}, time = {}", initial, grow, median);
}

fn main() {
  core_affinity::set_for_current(core_affinity::get_core_ids().unwrap()[0]);
  let args = std::env::args().skip(1).collect::<Vec<String>>();
  if args.len() != 3 {
    eprintln!("invalid number of arguments");
    return;
  }
  let initial = args[0].replace("_", "").parse::<usize>().unwrap();
  let grow = args[1].replace("_", "").parse::<usize>().unwrap();
  let iterations = args[2].replace("_", "").parse::<usize>().unwrap();
  bench_table_grow(initial, grow, iterations);
}
