use std::sync::atomic::{AtomicBool, Ordering};
#[derive(Default)]
pub struct ExitGate {
    requested: AtomicBool,
    complete: AtomicBool,
}
impl ExitGate {
    pub fn begin(&self) -> bool {
        !self.requested.swap(true, Ordering::SeqCst)
    }
    pub fn may_exit(&self) -> bool {
        self.complete.load(Ordering::SeqCst)
    }
    pub fn finish(&self) {
        self.complete.store(true, Ordering::SeqCst);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn repeated_exit_requests_wait_until_cleanup_finishes() {
        let gate = ExitGate::default();
        assert!(!gate.may_exit());
        assert!(gate.begin());
        assert!(!gate.begin());
        assert!(!gate.may_exit(), "重复退出不能绕过尚未完成的清理");
        gate.finish();
        assert!(gate.may_exit());
    }
}
