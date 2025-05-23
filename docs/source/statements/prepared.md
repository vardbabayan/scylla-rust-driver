# Prepared statement

Prepared statements provide much better performance than unprepared statements,
but they need to be prepared before use.

Benefits that prepared statements have to offer:
- Type safety - thanks to metadata provided by the server, the driver can verify bound values' types before serialization. This way, we can be always sure that the Rust type provided by the user is compatible (and if not, the error is returned) with the destined native type. The same applies for deserialization.
- Performance - when executing an unprepared statement with non-empty values list, the driver
prepares the statement before execution. The reason for this is to provide type safety for unprepared statements. However, this implies 2 round trips per unprepared statement execution. On the other hand, the cost of prepared statement's execution is only 1 round trip.
- Improved load-balancing - using the statement metadata, the driver can compute a set of destined replicas for current statement execution. These replicas will be preferred when choosing the node (and shard) to send the request to. For more insight on this, see [performance section](#performance).

```rust
# extern crate scylla;
# use scylla::client::session::Session;
# use std::error::Error;
# async fn check_only_compiles(session: &Session) -> Result<(), Box<dyn Error>> {
use scylla::statement::prepared::PreparedStatement;

// Prepare the statement for later execution
let prepared: PreparedStatement = session
    .prepare("INSERT INTO ks.tab (a) VALUES(?)")
    .await?;

// Execute the prepared statement with some values, just like an unprepared statement.
let to_insert: i32 = 12345;
session.execute_unpaged(&prepared, (to_insert,)).await?;
# Ok(())
# }
```

> ***Warning***\
> For token/shard aware load balancing to work properly, all partition key values
> must be sent as bound values (see [performance section](#performance))

> ***Warning***\
> Don't use `execute` to receive large amounts of data.\
> By default the query is unpaged and might cause heavy load on the cluster.
> In such cases set a page size and use a [paged query](paged.md) instead.
>
> When page size is set, `execute` will return only the first page of results.

### `Session::prepare`
`Session::prepare` takes statement text and prepares the statement on all nodes and shards.
If at least one succeeds returns success.

### `Session::execute`
`Session::execute` takes a prepared statement and bound values and executes the statement.
Passing values and the result is the same as in [unprepared statement](unprepared.md).

### Statement options

To specify custom options, set them on the `PreparedStatement` before execution.
For example to change the consistency:

```rust
# extern crate scylla;
# use scylla::client::session::Session;
# use std::error::Error;
# async fn check_only_compiles(session: &Session) -> Result<(), Box<dyn Error>> {
use scylla::statement::prepared::PreparedStatement;
use scylla::statement::Consistency;

// Prepare the statement for later execution
let mut prepared: PreparedStatement = session
    .prepare("INSERT INTO ks.tab (a) VALUES(?)")
    .await?;

// Set prepared statement consistency to One
// This is the consistency with which this statement will be executed
prepared.set_consistency(Consistency::One);

// Execute the prepared statement with some values, just like an unprepared statement.
let to_insert: i32 = 12345;
session.execute_unpaged(&prepared, (to_insert,)).await?;
# Ok(())
# }
```

See [PreparedStatement API documentation](https://docs.rs/scylla/latest/scylla/statement/prepared_statement/struct.PreparedStatement.html)
for more options.

> ***Note***
> Prepared statements can be created from `Statement` structs and will inherit from
> the custom options that the `Statement` was created with.
> This is especially useful when using `CachingSession::execute` for example.

### Performance

Prepared statement have good performance, much better than unprepared statements.
By default they use shard/token aware load balancing.

> **Always** pass partition key values as bound values.
> Otherwise the driver can't hash them to compute partition key
> and they will be sent to the wrong node, which worsens performance.

Let's say we have a table like this:

```sql
TABLE ks.prepare_table (
    a int,
    b int,
    c int,
    PRIMARY KEY (a, b)
)
```

```rust
# extern crate scylla;
# use scylla::client::session::Session;
# use std::error::Error;
# async fn check_only_compiles(session: &Session) -> Result<(), Box<dyn Error>> {
use scylla::statement::prepared::PreparedStatement;

// WRONG - partition key value is passed in statement string
// Load balancing will compute the wrong partition key
let wrong_prepared: PreparedStatement = session
    .prepare("INSERT INTO ks.prepare_table (a, b, c) VALUES(12345, ?, 16)")
    .await?;

session.execute_unpaged(&wrong_prepared, (54321,)).await?;

// GOOD - partition key values are sent as bound values
// Other values can be sent any way you like, it doesn't matter
let good_prepared: PreparedStatement = session
    .prepare("INSERT INTO ks.prepare_table (a, b, c) VALUES(?, ?, 16)")
    .await?;

session.execute_unpaged(&good_prepared, (12345, 54321)).await?;

# Ok(())
# }
```
