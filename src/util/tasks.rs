use futures::task::AtomicWaker;
use std::{
	future::{Future, IntoFuture},
	pin::Pin,
	sync::{
		atomic::{AtomicUsize, Ordering},
		Arc,
	},
	task::{Context, Poll},
};
use tokio::task::JoinHandle;

struct Inner {
	waker: AtomicWaker,
	task_count: AtomicUsize,
}

/// A set of tasks.
///
/// This is used to gracefully wait for multiple tasks to finish without having to join all of them.
#[derive(Clone)]
pub struct Tasks(Arc<Inner>);

/// A future that resolves when all [`Tasks`] are finished running.
///
/// If no tasks are running, this resolves immediately.
pub struct TasksFinished(Arc<Inner>);

impl Future for TasksFinished {
	type Output = ();

	fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
		self.0.waker.register(cx.waker());
		if self.0.task_count.load(Ordering::Acquire) == 0 {
			Poll::Ready(())
		} else {
			Poll::Pending
		}
	}
}

impl Tasks {
	pub fn new() -> Self {
		Self(Arc::new(Inner {
			waker: AtomicWaker::new(),
			task_count: AtomicUsize::new(0),
		}))
	}

	pub fn spawn<T>(&self, future: T) -> JoinHandle<T::Output>
	where
		T: Future + Send + 'static,
		T::Output: Send + 'static,
	{
		self.0.task_count.fetch_add(1, Ordering::Release);

		struct TaskGuard(Arc<Inner>);
		impl Drop for TaskGuard {
			fn drop(&mut self) {
				if self.0.task_count.fetch_sub(1, Ordering::AcqRel) == 1 {
					self.0.waker.wake();
				}
			}
		}
		let guard = TaskGuard(Arc::clone(&self.0));

		tokio::spawn(async move {
			let _ = guard;
			future.await
		})
	}

	pub fn finished(&self) -> TasksFinished {
		TasksFinished(Arc::clone(&self.0))
	}
}

impl IntoFuture for Tasks {
	type IntoFuture = TasksFinished;
	type Output = ();

	fn into_future(self) -> Self::IntoFuture {
		TasksFinished(self.0)
	}
}

impl Default for Tasks {
	fn default() -> Self {
		Self::new()
	}
}
