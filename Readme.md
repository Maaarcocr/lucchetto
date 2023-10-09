# lucchetto

easily call a rust function without holding the GVL lock

lucchetto = lock in italian

## How to install

add this to your `Cargo.toml`:

```toml
[dependencies]
lucchetto = "0.2.0"
```

or

```bash
cargo add lucchetto
```

## Why?

let's say that you have written a rust function that is called from ruby, using
[magnus](https://github.com/matsadler/magnus) and [rb-sys](https://github.com/oxidize-rb/rb-sys).

as an example let's take this simple function (from our simple example):

```rust
use magnus::{define_global_function, function};

fn slow_func(input: String) -> String {
    std::thread::sleep(std::time::Duration::from_secs(2));
    input.len().to_string()
}

#[magnus::init]
fn init() {
    define_global_function("slow_func", function!(slow_func, 1));
}

```

This allows you to write a ruby script like this:

```ruby
require_relative "./lib/simple"

t = Thread.new do
  puts slow_func("hello")
end

1..10.times do
  sleep 0.1
  puts "main thread"
end

t.join
```

but you'll notice that because your rust function takes a long time, all ruby threads will be blocked until the rust function returns. This would be the output of the above script:

```
main thread
5
main thread
main thread
main thread
main thread
main thread
main thread
main thread
main thread
main thread
```

this shows that the main thread is blocked until the rust function returns (which is running a different thread). This is because the GVL lock is held by the rust thread, and no other ruby thread can run.

## Can we fix this?

yes! we can use a simple attribute macro from lucchetto to tell the ruby VM that we are not going to use the GVL lock, and that it can run other ruby threads while we are running our rust function.

```rust
use lucchetto::without_gvl;
use magnus::{define_global_function, function};

#[without_gvl]
fn slow_func(input: String) -> String {
    std::thread::sleep(std::time::Duration::from_secs(2));
    input.len().to_string()
}

#[magnus::init]
fn init() {
    define_global_function("slow_func", function!(slow_func, 1));
}
```

If we run the same ruby script as before, we'll see that the main thread is not blocked anymore:

```
main thread
main thread
main thread
main thread
main thread
main thread
main thread
main thread
main thread
main thread
5
```

as you can see, the main thread is not blocked anymore, and the rust function is still running in the background.

## Safety concerns

In order to not allow the possibility of running a function that would be not safe, as it would have access
to ruby objects, a trait called `GvlSafe` has been introduced. Functions can only take and return types that
implement this trait if they want to use the `#[without_gvl]` attribute macro.

It's an empty trait, so it's easy to implement on your own types if you think they are safe to use. Example:

```rust
use lucchetto::GvlSafe;

struct MyStruct;

impl GvlSafe for MyStruct {}
```

You can think of `GvlSafe` as `Send` and `Sync` for the GVL lock.

## Is this good code?

Honestly? I don't know. It may contain memory bugs (I've had to do lots of pointer-y things to make this work), and it may be unsafe. I have not spent a lot of time on this, so I'm not sure if this is the best way to do this. But does it seem to work? Yes.
