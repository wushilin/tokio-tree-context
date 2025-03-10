use std::{future::Future, time::Duration};
use tokio::sync::broadcast::Sender;
use tokio::{sync::broadcast, time::Instant};

/// A context that can be used to spawn tokio tasks
/// Cancelling the context (or dropping it) will cancel all async tasks spawn by this context
/// You can create child context too.
/// ```rust, no_run
///    use std::time::Duration;
///    use tokio_tree_context::Context;
///    async fn testing() {
///        let mut ctx = Context::new();
///        
///        let mut ctx1 = ctx.new_child_context();
///        let mut ctx12 = ctx1.new_child_context();
///        
///        ctx.spawn(async move {
///            sleep("ctx".into(), 100).await;
///        });
///        ctx1.spawn(async move {
///            sleep("ctx1".into(), 100).await;
///        });
///        ctx12.spawn(async move {
///            sleep("ctx12".into(), 100).await;
///        });
///        println!("Cancelling CTX 1");
///        drop(ctx1);
///        sleep("main".into(), 5).await;
///        println!("Cancelling CTX 12");
///        drop(ctx12);
///        sleep("main".into(), 5).await;
///        
///        println!("Cancelling CTX");
///        drop(ctx);
///        
///        sleep("main".into(), 5).await;
///        
///    }
///        
///    async fn sleep(name:String, what: u64) {
///        for i in 0..what {
///            println!("Task {} sleeping {} out of {} seconds", name, i + 1, what);
///            tokio::time::sleep(Duration::from_secs(1)).await;
///            println!("Task {} awake", name);
///        }
///    }
/// ```
pub struct Context {
    cancel_sender: Sender<()>,
}

impl Context {
    /// Cancel all tasks under this context
    pub fn cancel(self) {}

    /// Create a new context
    pub fn new() -> Context {
        let (tx, _) = broadcast::channel(1);
        Context {
            cancel_sender: tx,
        }
    }

    /// Create a new Context from a parent. Same as `parent.new_child_context()`
    pub fn with_parent(parent: &mut Context) -> Context {
        return parent.new_child_context();
    }

    /// Create a new child context, where cancelling the parent context, will also cancel the child context.
    /// Child context can have their own child context too.
    /// 
    /// The new context has a logical relationship with the parent. Cancelling parent will cancel child too.
    pub fn new_child_context(&mut self) -> Context {
        let (new_tx, _) = broadcast::channel(1);
        let new_tx_clone = new_tx.clone();
        let wsender= new_tx_clone.downgrade();
        drop(new_tx_clone);
        let mut rx = self.cancel_sender.subscribe();
        tokio::spawn(async move {
            let _ = rx.recv().await;
            wsender.upgrade().map(|x| x.send(()))
        });
        Context {
            cancel_sender: new_tx
        }
    }

    /// Run a task with at timeout. If timeout is None, then no timeout is used
    /// Task will run until:
    ///     The task is completed
    ///     The timeout is reached
    ///     The context is cancelled
    ///     Any of the parent/ancestor context is cancelled
    /// 
    /// Which ever is earlier.
    /// For example
    /// ```rust,no_run
    /// use std::time::Duration;
    /// use tokio_tree_context::Context;
    /// 
    /// let mut ctx = Context::new();
    /// ctx.spawn_with_timeout(async move {
    ///     // do your work here
    /// }, Some(Duration::from_secs(3))); // task cancels after 3 seconds
    /// // wait sometime
    /// ctx.cancel();
    /// ```
    pub fn spawn_with_timeout<T>(&mut self, future: T, timeout: Option<Duration>) -> tokio::task::JoinHandle<Option<T::Output>>
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static,
    {
        let mut rx = self.cancel_sender.subscribe();
        if let Some(duration) = timeout {
            tokio::task::spawn(async move {
                tokio::select! {
                    res = future => Some(res),
                    _ = rx.recv() => None,
                    _ = tokio::time::sleep_until(Instant::now() + duration) => None,
                }
            })
        } else {
            tokio::task::spawn(async move {
                tokio::select! {
                    res = future => Some(res),
                    _ = rx.recv() => None,
                }
            })
        }
    }

    /// Spawn task without tiemout
    /// Task is cancelled when you call this context's cancel or drop the context
    /// 
    /// For example
    /// ```rust, no_run
    /// use std::time::Duration;
    /// use tokio_tree_context::Context;
    /// 
    /// let mut ctx = Context::new();
    /// ctx.spawn(async move {
    ///     // do your work here
    /// });
    /// // wait sometime
    /// ctx.cancel();
    /// ```
    pub fn spawn<T>(&mut self, future: T) -> tokio::task::JoinHandle<Option<T::Output>>
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static,
    {
        self.spawn_with_timeout(future, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() {
        let mut ctx = Context::new();

        let mut ctx1 = ctx.new_child_context();
        let mut ctx12 = ctx1.new_child_context();

        ctx.spawn(async move {
            sleep("ctx".into(), 100).await;
        });
        ctx1.spawn(async move {
            sleep("ctx1".into(), 100).await;
        });
        ctx12.spawn(async move {
            sleep("ctx12".into(), 100).await;
        });
        println!("Cancelling CTX 1");
        drop(ctx1);
        sleep("main".into(), 5).await;
        println!("Cancelling CTX 12");
        drop(ctx12);
        sleep("main".into(), 5).await;

        println!("Cancelling CTX");
        drop(ctx);

        sleep("main".into(), 5).await;

    }

    async fn sleep(name:String, what: u64) {
        for i in 0..what {
            println!("Task {} sleeping {} out of {} seconds", name, i + 1, what);
            tokio::time::sleep(Duration::from_secs(1)).await;
            println!("Task {} awake", name);
        }
    }
}
