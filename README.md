# tokio-tree-context
Similar to tokio-context, but support multiple level context.

You can launch as many as async tasks from it. Dropping the context, or calling context.cancel(), will cancel all of them.

Better thing is that you can create a new child context, and child context can create further child context, all of them can be used the exactly same way.

Cancelling a context, cancels all tasks launched by it directly, and also cancels all child context, and child context of child context, as so on. The tasks launched
by descendent context, are also cancelled.

This making task scheduling super each. You have root context, which you can cancel at any time. Each iteration can use a new child context, which at
end of it, you can just cancel it. so all resources are cleaned up.


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

# Common pitfalls
Note that if a context is cancelled, or simply dropped, the tasks launched by it will cancel too.

For example
```
async fn test(mut context:Context) {
        let mut child_context = context.new_child_context();
        child_context.spawn(async move {
            tokio::time::sleep(Duration::from_secs(100)).await;
            println!("I am done")
        });
}

async fn main() {
    let mut context = tokio_tree_context::Context::new();

    test(context);
}
```

Will you see "I am done" after 100 seconds? no. because the child context is dropped after you spawn the task. That will cancel your task too.

You can revise it by either using the main context to spawn, or join the launched tasks's join handle so child_context is not dropped.
