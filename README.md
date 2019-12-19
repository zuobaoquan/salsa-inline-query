# salsa-inline-query
Support inline query definitions in [salsa](https://github.com/salsa-rs/salsa) query group trait

## Example
```rust
use salsa_inline_query::salsa_inline_query;

#[salsa_inline_query]
#[salsa::query_group(SourceDatabaseStorage)]
trait SourceDatabase: std::fmt::Debug {
    #[salsa::input]
    fn source_text(&self, file_id: u32) -> Arc<String>;
    fn source_len(&self, file_id: u32) -> usize {
        let text = self.source_text(file_id);
        text.len()
    }
}
```

The above code will be transformed to:

```rust
#[salsa::query_group(SourceDatabaseStorage)]
trait SourceDatabase: std::fmt::Debug {
    #[salsa::input]
    fn source_text(&self, file_id: u32) -> Arc<String>;
    fn source_len(&self, file_id: u32) -> usize;
}

fn source_len(__salsa_db: &impl SourceDatabase, file_id: u32) -> usize {
    let text = __salsa_db.source_text(file_id);
    text.len()
}
```
