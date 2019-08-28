mod thread_scheduler;
use thread_scheduler::new_thread_schedule;
mod thread_pool_scheduler;
use thread_pool_scheduler::thread_pool_schedule;

/// A Scheduler is an object to order task and schedule their execution.
pub trait Scheduler {
  fn schedule<T: Send + Sync + 'static, R: Send + Sync + 'static>(
    &self,
    task: impl FnOnce(Option<T>) -> R + Send + 'static,
    state: Option<T>,
  ) -> R;
}

pub enum Schedulers {
  /// Sync Scheduler execute task immediately in current thread.
  Sync,
  /// NewThread Scheduler always creates a new thread for each unit of work.
  NewThread,
  /// ThreadPool dispatch task to the thread pool to execute task.
  ThreadPool,
}

impl Scheduler for Schedulers {
  fn schedule<T: Send + Sync + 'static, R: Send + Sync + 'static>(
    &self,
    task: impl FnOnce(Option<T>) -> R + Send + 'static,
    state: Option<T>,
  ) -> R {
    match self {
      Schedulers::NewThread => new_thread_schedule(task, state),
      Schedulers::ThreadPool => thread_pool_schedule(task, state),
      Schedulers::Sync => task(state),
    }
  }
}

#[cfg(test)]
mod test {
  extern crate test;
  use crate::ops::ObserveOn;
  use crate::prelude::*;
  use crate::scheduler::Schedulers;
  use std::f32;
  use std::sync::{Arc, Mutex};
  use test::Bencher;

  #[bench]
  fn pool(b: &mut Bencher) { b.iter(|| sum_of_sqrt(Schedulers::ThreadPool)) }

  #[bench]
  fn new_thread(b: &mut Bencher) {
    b.iter(|| sum_of_sqrt(Schedulers::NewThread))
  }

  #[bench]
  fn sync(b: &mut Bencher) { b.iter(|| sum_of_sqrt(Schedulers::Sync)) }

  fn sum_of_sqrt(scheduler: Schedulers) {
    let sum = Arc::new(Mutex::new(0.));
    observable::from_range(0..1000)
      .observe_on(scheduler)
      .subscribe(move |v| {
        *sum.lock().unwrap() += (*v as f32).sqrt();
      });
  }
}
