use crate::msg::Msg;
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, PartialEq, Eq)]
pub struct Empty;

pub struct Lane<const N: usize> {
    head: AtomicUsize,
    tail: AtomicUsize,
    slots: [UnsafeCell<MaybeUninit<Msg>>; N],
}

unsafe impl<const N: usize> Sync for Lane<N> {}

impl<const N: usize> Lane<N> {
    pub const fn new() -> Self {
        assert!(N > 0);
        Self {
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            slots: [const { UnsafeCell::new(MaybeUninit::uninit()) }; N],
        }
    }

    pub fn try_send(&self, msg: Msg) -> Result<(), Msg> {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Relaxed);
        if tail.wrapping_sub(head) >= N {
            return Err(msg);
        }
        unsafe {
            (*self.slots[tail % N].get()).write(msg);
        }
        self.tail.store(tail.wrapping_add(1), Ordering::Release);
        Ok(())
    }

    pub fn try_recv(&self) -> Result<Msg, Empty> {
        let tail = self.tail.load(Ordering::Acquire);
        let head = self.head.load(Ordering::Relaxed);
        if head == tail {
            return Err(Empty);
        }
        let msg = unsafe { (*self.slots[head % N].get()).assume_init_read() };
        self.head.store(head.wrapping_add(1), Ordering::Release);
        Ok(msg)
    }

    pub fn len(&self) -> usize {
        let tail = self.tail.load(Ordering::Acquire);
        let head = self.head.load(Ordering::Acquire);
        tail.wrapping_sub(head)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let lane = Lane::<4>::new();
        assert!(lane.try_recv() == Err(Empty));
        for i in 0..4u64 {
            assert!(lane.try_send(Msg::new(i as u32, i)).is_ok());
        }
        assert!(lane.try_send(Msg::new(9, 9)).is_err());
        assert_eq!(lane.len(), 4);
        for i in 0..4u64 {
            let m = lane.try_recv().unwrap();
            assert_eq!(m.label, i as u32);
            assert_eq!(m.seq, i);
        }
        assert!(lane.try_recv() == Err(Empty));
        assert!(lane.try_send(Msg::new(1, 1)).is_ok());
        assert_eq!(lane.try_recv().unwrap().seq, 1);
    }

    #[test]
    fn threads() {
        let lane = Lane::<64>::new();
        std::thread::scope(|s| {
            s.spawn(|| {
                for i in 0..10_000u64 {
                    while lane.try_send(Msg::new(1, i)).is_err() {
                        core::hint::spin_loop();
                    }
                }
            });
            let consumer = s.spawn(|| {
                let mut last = None;
                for _ in 0..10_000 {
                    loop {
                        match lane.try_recv() {
                            Ok(m) => {
                                last = Some(m.seq);
                                break;
                            }
                            Err(_) => core::hint::spin_loop(),
                        }
                    }
                }
                last
            });
            assert_eq!(consumer.join().unwrap(), Some(9_999));
        });
    }
}
