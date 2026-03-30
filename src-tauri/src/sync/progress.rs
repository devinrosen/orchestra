use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Clone)]
pub struct CancelToken {
    flag: Arc<AtomicBool>,
}

impl CancelToken {
    pub fn new() -> Self {
        Self {
            flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.flag.store(true, Ordering::Relaxed);
    }

    pub fn flag(&self) -> Arc<AtomicBool> {
        self.flag.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[test]
    fn test_cancel_token_starts_not_cancelled() {
        let token = CancelToken::new();
        assert!(!token.flag().load(Ordering::Relaxed));
    }

    #[test]
    fn test_cancel_token_cancel_sets_flag() {
        let token = CancelToken::new();
        token.cancel();
        assert!(token.flag().load(Ordering::Relaxed));
    }

    #[test]
    fn test_cancel_token_flag_shared_across_arc_clones() {
        let token = CancelToken::new();
        let flag1 = token.flag();
        let flag2 = token.flag();

        token.cancel();

        assert!(flag1.load(Ordering::Relaxed));
        assert!(flag2.load(Ordering::Relaxed));
    }
}
