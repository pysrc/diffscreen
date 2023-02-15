// 传输压缩水平0-21 0消耗资源最小但是压缩率小（需要带宽大） 21消耗资源最大，但但是压缩率大（需要带宽小）
pub const COMPRESS_LEVEL: i32 = 3;

// 当开启SKIP选项时，传输长度大于SKIP_LENGTH时，暂停SKIP_TIME毫秒
pub const SKIP: bool = true;
pub const SKIP_LENGTH: usize = 1024 * 5;
pub const SKIP_TIME: u64 = 100;
