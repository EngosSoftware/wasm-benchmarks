/// Returns a normalized result value from a vector of input values.
#[allow(clippy::manual_is_multiple_of)]
pub fn norm(mut input: Vec<u64>) -> u64 {
  input.sort_unstable();
  let n = input.len();
  let median = if n % 2 == 0 { (input[n / 2 - 1] + input[n / 2]) / 2 } else { input[n / 2] };
  let mut filtered = vec![];
  for value in input {
    let diff = (value.abs_diff(median) as f64 / median as f64) * 100.0;
    if diff < 50.0 {
      filtered.push(value);
    }
  }
  let sum: u64 = filtered.iter().sum();
  sum.checked_div(filtered.len() as u64).unwrap_or_default()
}

pub fn human_friendly_seconds(nanos: u64) -> String {
  if nanos < 1_000 {
    return format!("{}.0 ns", nanos);
  }
  if nanos < 1_000_000 {
    return format!("{:.1} µs", (nanos as f64) / 1_000.0);
  }
  if nanos < 1_000_000_000 {
    return format!("{:.1} ms", (nanos as f64) / 1_000_000.0);
  }
  format!("{:.1} s", (nanos as f64) / 1_000_000_000.0)
}
