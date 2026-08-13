/// Returns a normalized result value from a vector of input values.
#[allow(clippy::manual_is_multiple_of)]
pub fn calc(mut input: Vec<u64>) -> u64 {
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
