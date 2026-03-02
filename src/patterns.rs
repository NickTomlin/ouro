pub trait PatternSet: Send + Sync {
    fn prefix(&self) -> &str;
    fn args_inline(&self) -> &str;
    fn args_open(&self) -> &str;
    fn args_close(&self) -> &str;
    fn stdout_inline(&self) -> &str;
    fn stdout_open(&self) -> &str;
    fn stdout_close(&self) -> &str;
    fn stderr_inline(&self) -> &str;
    fn stderr_open(&self) -> &str;
    fn stderr_close(&self) -> &str;
    fn exit(&self) -> &str;
}

pub struct DefaultPatterns {
    pub prefix: String,
}

impl DefaultPatterns {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self { prefix: prefix.into() }
    }
}

impl PatternSet for DefaultPatterns {
    fn prefix(&self) -> &str { &self.prefix }
    fn args_inline(&self) -> &str { "args:" }
    fn args_open(&self) -> &str { "args:" }
    fn args_close(&self) -> &str { ":args" }
    fn stdout_inline(&self) -> &str { "out:" }
    fn stdout_open(&self) -> &str { "out:" }
    fn stdout_close(&self) -> &str { ":out" }
    fn stderr_inline(&self) -> &str { "err:" }
    fn stderr_open(&self) -> &str { "err:" }
    fn stderr_close(&self) -> &str { ":err" }
    fn exit(&self) -> &str { "exit:" }
}
