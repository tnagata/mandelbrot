#[allow(unused)]

#[cfg(test)]
mod atomic_counter {
    use crossbeam::scope;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn bang_on_counter() {
        let n = AtomicUsize::new(0);

        scope(|scope| {
            for _ in 0..100 {
                scope.spawn(|_| {
                    for _ in 0..100_000 {
                        n.fetch_add(1, Ordering::Relaxed);
                    }
                });
            }
        }).unwrap();

        assert_eq!(n.load(Ordering::SeqCst), 10_000_000);
    }
}

#[cfg(test)]
mod atomic_iterator {
    use crossbeam::scope;
    mod counter {
        use std::sync::atomic::{AtomicUsize, Ordering};
        pub struct Counter {
            count: AtomicUsize
        }

        impl Counter {
            pub fn new(count: usize) -> Counter {
                Counter { count: AtomicUsize::new(count) }
            }

            fn next(&self) -> Option<usize> {
                let mut current;
                loop {
                    current = self.count.load(Ordering::SeqCst);
                    if current == 0 {
                        return None;
                    }
                    if self.count.compare_exchange(current, current - 1,
                                                   Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                        return Some(current - 1);
                    }
                }
            }
        }

        impl<'a> Iterator for &'a Counter {
            type Item = usize;
            fn next(&mut self) -> Option<usize> { (*self).next() }
        }
    }

    #[test]
    fn test_ai() {
        for _ in 0..100 {
            let c = counter::Counter::new(10000);

            scope(|scope| {
                let mut threads = vec![];
                for _ in 0..100 {
                    threads.push(scope.spawn(|_| { c.collect::<Vec<_>>() }));
                }

                let mut seen = [false; 10000];
                for thread in threads {
                    for i in thread.join().unwrap() {
                        assert!(!seen[i]);
                        seen[i] = true;
                    }
                }
            }).unwrap();
        }
    }
}

mod atomic_chunks_mut {
    use std;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering::*;

    pub struct AtomicChunksMut<'a, T> {
        slice: &'a [T],
        step: usize,
        next: AtomicUsize
    }

    impl<'a, T> AtomicChunksMut<'a, T> {
        pub fn new(slice: &'a mut [T], step: usize) -> AtomicChunksMut<'a, T> {
            AtomicChunksMut {
                slice: slice,
                step: step,
                next: AtomicUsize::new(0)
            }
        }

        #[allow(mutable_transmutes)]
        unsafe fn next(&self) -> Option<(usize, &'a mut [T])> {
            loop {
                let current = self.next.load(SeqCst);
                assert!(current <= self.slice.len());
                if current == self.slice.len() {
                    return None;
                }
                let end = std::cmp::min(current + self.step, self.slice.len());
                if self.next.compare_exchange(current, end, SeqCst, SeqCst).is_ok() {
                    return Some((current / self.step, std::mem::transmute(&self.slice[current..end])));
                }
            }
        }
    }

    impl<'a, 'b, T> Iterator for &'b AtomicChunksMut<'a, T> {
        type Item = (usize, &'a mut [T]);
        fn next(&mut self) -> Option<Self::Item> { unsafe { (*self).next() } }
    }
}

pub use self::atomic_chunks_mut::AtomicChunksMut;

#[test]
fn test_ait() {
    let mut v = vec![0,1,2,3,4,5,6,7,8,9,10];
    let c : Vec<_> = (&AtomicChunksMut::new(&mut v[..], 3)).collect();

    assert_eq!((&c).iter().map(|&(i, _)| i).collect::<Vec<_>>(), vec![0,1,2,3]);
    assert_eq!((&c).iter().map(|&(_, ref s)| s[0]).collect::<Vec<_>>(), vec![0,3,6,9]);
}

#[test]
fn stress_test_ait() {
    use crossbeam::scope;
    let mut v : Vec<usize> = (0..10000).collect();
    let it = AtomicChunksMut::new(&mut v[..], 3);

    scope(|scope| {
        let mut threads = vec![];
        for _ in 0..10 {
            threads.push(scope.spawn(|_| {
                let mut v = vec![];
                for (_, chunk) in &it { v.push(chunk[0]); }
                v
            }));
        }

        let mut seen = vec![false; 10000];
        for thread in threads {
            for first in thread.join().unwrap() {
                assert!(first % 3 == 0);
                assert!(!seen[first]);
                seen[first] = true;
            }
        }
    }).unwrap();
}
