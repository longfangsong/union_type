# union_type

Add union type support to rust!

## Why `enum` is not enough

Take one of the most successful Rust project, TiKV as an [example](https://github.com/tikv/tikv/blob/fcb2791312203f40167a92bc9bf4c4421e72ecad/src/storage/txn/commands/mod.rs#L75):

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

On the other hand, traits and dynamic dispatch are language features that 
aim for "extending", it do describe shared behaviour between types,
but its more like something to describe a "protocol" to make some "unknown"
custom type working with your existing code, and **open** for other parts of the program.
There could be unlimited different types
to implement some certain trait.

## What do we need?

Union type!

Or we can say "untagged" union type.

It is a **closed** type, we know some object is one of some defined types.

And we can uses a certain method if and only if it exists on all these types,
these methods should be "automatically" become usable, with out using `match` to get the actual child type.

By using macros, we can bring part of this feature to rust.

## What's the result look like?


```rust
struct A(String);

impl A {
    fn f(&self, a: i32) -> i32 {
        println!("from A {}", a + 1);
        a + 1
    }
}

struct B(i32);

impl B {
    fn f(&self, a: i32) -> i32 {
        println!("from B {}", a + &self.0);
        a + &self.0
    }
}

union_type! {
    enum C {
        A,
        B
    }

    impl C {
        fn f(&self, a: i32) -> i32;
    }
}

fn main() {
    let a = A("abc".to_string());
    let mut c:C = a.into();
    c.f(1);
    let b = B(99);
    c = b.into();
    c.f(2);
}
```

The output is: 
```shell
from A 2
from B 101
```


