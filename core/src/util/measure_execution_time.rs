use std::time::{Duration, Instant};

pub fn measure_execution_time<T, F: FnOnce() -> T>(closure: F) -> (T, Duration) {
    let start_time = Instant::now();
    let v = closure();
    let duration = start_time.elapsed();
    (v, duration)
}

pub fn measure_execution_time_result<T, E, F: FnOnce() -> Result<T, E>>(
    closure: F,
) -> Result<(T, Duration), E> {
    let start_time = Instant::now();
    let v = closure()?;
    let duration = start_time.elapsed();
    Ok((v, duration))
}
