# tokio-tree-context
Similar to tokio-context, but support multiple level context

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