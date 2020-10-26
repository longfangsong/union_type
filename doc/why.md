# Why we need this crate.

## Why `enum` is not enough

Take code from one of the most successful Rust project,
TiKV as an [example](https://github.com/tikv/tikv/blob/fcb2791312203f40167a92bc9bf4c4421e72ecad/src/storage/txn/commands/mod.rs#L75):

```rust
pub enum Command {
    Prewrite(Prewrite),
    PrewritePessimistic(PrewritePessimistic),
    AcquirePessimisticLock(AcquirePessimisticLock),
    Commit(Commit),
    Cleanup(Cleanup),
    Rollback(Rollback),
    PessimisticRollback(PessimisticRollback),
    TxnHeartBeat(TxnHeartBeat),
    CheckTxnStatus(CheckTxnStatus),
    CheckSecondaryLocks(CheckSecondaryLocks),
    ScanLock(ScanLock),
    ResolveLockReadPhase(ResolveLockReadPhase),
    ResolveLock(ResolveLock),
    ResolveLockLite(ResolveLockLite),
    Pause(Pause),
    MvccByKey(MvccByKey),
    MvccByStartTs(MvccByStartTs),
}
```

It seems silly to make a lot of items with same arm name and data type name.

There is a crate called [sum_type](https://docs.rs/sum_type/0.2.0/sum_type), 
but it won't help when we also need to "sum" the methods:

```rust
pub fn process_write<S: Snapshot, L: LockManager, P: PdClient + 'static>(
    self,
    snapshot: S,
    context: WriteContext<'_, L, P>,
) -> Result<WriteResult> {
    match self {
        Command::Prewrite(t) => t.process_write(snapshot, context),
        Command::PrewritePessimistic(t) => t.process_write(snapshot, context),
        Command::AcquirePessimisticLock(t) => t.process_write(snapshot, context),
        Command::Commit(t) => t.process_write(snapshot, context),
        Command::Cleanup(t) => t.process_write(snapshot, context),
        Command::Rollback(t) => t.process_write(snapshot, context),
        Command::PessimisticRollback(t) => t.process_write(snapshot, context),
        Command::ResolveLock(t) => t.process_write(snapshot, context),
        Command::ResolveLockLite(t) => t.process_write(snapshot, context),
        Command::TxnHeartBeat(t) => t.process_write(snapshot, context),
        Command::CheckTxnStatus(t) => t.process_write(snapshot, context),
        Command::CheckSecondaryLocks(t) => t.process_write(snapshot, context),
        Command::Pause(t) => t.process_write(snapshot, context),
        _ => panic!("unsupported write command"),
    }
}
```

Every arm in this `match` call to `process_write` with the same arguments. Which seems repeatitive.

## So how about using traits and dynamic dispatch?

We do tried to do dynamic dispatch in TiKV,
but it turns out that there are so many limitations for traits,
for example, sometimes methods in traits cannot handle type parameters when return a dyn reference,
see the discussion [here](https://github.com/tikv/tikv/pull/8296#discussion_r462040210).

## What's wrong with these solutions?

Well, I think the enums in Rust is kind of sum type, 
or **tagged** union type, which
needs to be `match`ed before using any method on 
its children type, even if there exists such a method on all its children type.
This is reasonable but inconvinient in some situation.

On the other hand, traits and dynamic dispatch are language features that 
aimed for "extending", it do describe shared behaviour between types,
but its more like something to describe a "protocol" to make some "unknown"
custom type working with your existing code, and **open** for other parts of the program.
But sometimes we don't really want the users to extend a type,
expecially for types which all its variants are known in the first place
and are used internally or not exposing to other crates, 

## So what do we need?

Union type!

Or we can say "**untagged**", "**closed**" union type.

And we can uses a certain method if and only if it exists on all these types,
these methods should be "automatically" become usable, without using `match` to get the actual child type.

By using macros, we can bring part of this feature to rust. And we come to this crate.
