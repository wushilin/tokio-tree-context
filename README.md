# tokio-tree-context
Similar to tokio-context, but support multiple level context

# Why?

You can use it as a context that can cancel as if it is a normal context.

You can spawn child context from it, where child context is a context, and cancelling context also cancels all child contexts. And if child context has other contexts, they are cancelled too.

Imagine you want to allow an API to spawn new tasks by giving them a context.
Then on timeout of the task, you want to cancel that context and all new tasks started by it, you can use this context.


# Usage


# Example
```rust
        let mut ctx = tokio_tree_context::Context::new();

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
```
