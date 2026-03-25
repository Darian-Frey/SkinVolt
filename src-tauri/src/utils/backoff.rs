use std::time::Duration;
use tokio::time::sleep;

/// Retries a fallible operation with exponential backoff.
pub async fn retry_with_backoff<F, T, E, Fut>(
    mut operation: F,
    max_retries: usize,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut backoff = 1;
    for i in 0..max_retries {
        match operation().await {
            Ok(val) => return Ok(val),
            Err(err) => {
                if i == max_retries - 1 {
                    return Err(err);
                }
                println!("⚠️ Attempt {} failed: {}. Retrying in {}s...", i + 1, err, backoff);
                sleep(Duration::from_secs(backoff)).await;
                backoff *= 2;
            }
        }
    }
    unreachable!()
}
