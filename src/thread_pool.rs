use std::sync::{Arc, Condvar, Mutex};
use std::thread;

pub(crate) struct AsyncMapper;

impl AsyncMapper {
    pub(crate) fn map_async<F, T, R>(items: Vec<T>, func: F, threads: u32) -> Vec<R>
        where
            F: Sync + Send + Fn(T) -> R + 'static,
            T: Send + 'static,
            R: Send + 'static,
    {

        let func = Arc::new(func);
        let pair = Arc::new((Mutex::new(0), Condvar::new()));

        let handlers: Vec<_> = items.into_iter().map(|it| {
            let pair = Arc::clone(&pair);
            let func = Arc::clone(&func);

            thread::spawn(move || {
                let (lock, cvar) = &*pair;
                let mut counter = lock.lock().unwrap();
                while *counter >= threads {
                    counter = cvar.wait(counter).unwrap();
                }
                *counter += 1;
                drop(counter);

                let result = (*func)(it);

                let mut counter = lock.lock().unwrap();
                *counter -= 1;
                cvar.notify_all();

                result
            })
        }).collect();

        let mut out = Vec::new();
        for handle in handlers {
            out.push(handle.join().unwrap());
        }

        out
    }
}