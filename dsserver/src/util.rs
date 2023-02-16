use crate::config::{SKIP, SKIP_LENGTH, SKIP_TIME};
#[inline]
pub async fn skip(clen: usize) {
    if SKIP {
        if clen > SKIP_LENGTH {
            tokio::time::sleep(std::time::Duration::from_millis(SKIP_TIME)).await;
        }
    }
}
