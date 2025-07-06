# tracing

## Reference

[1] <https://course.rs/logs/tracing.html>

## 简介

tracing 中最重要的三个概念就是 Span、Event 和 Collector。

### 1. Span

相比起日志只能记录在某个时间点发生的事件，span 最大的意义就在于它可以记录一个过程，也就是在某一段时间内发生的事件流。既然是记录时间段，那自然有开始和结束:

```rust
use tracing::{span, Level};
fn main() {
    let span = span!(Level::TRACE, "my_span");

    // enter 返回一个 RAII, 被 drop 时会自动结束该 span
    let enter = span.enter();
}
```

### 2. Event

Event 代表了某个时间点发生的事件，这方面它跟日志类似，但是不同的是，Event 还可以产生在 span 的上下文中。

```rust
use tracing::{event, span, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    tracing_subscriber::registry().with(fmt::layer()).init();
    // 在 span 的上下文之外记录一次 event 事件
    event!(Level::INFO, "something happened");

    let span = span!(Level::INFO, "my_span");
    let _guard = span.enter();

    // 在 "my_span" 的上下文中记录一次 event
    event!(Level::DEBUG, "something happened inside my_span");
}
```

打印出来的日志如下：

```bash
2022-04-09T14:51:38.382987Z  INFO test_tracing: something happened
2022-04-09T14:51:38.383111Z DEBUG my_span: test_tracing: something happened inside my_span
```

虽然 event 在哪里都可以使用，但是最好只在 span 的上下文中使用：用于代表一个时间点发生的事件，例如记录 HTTP 请求返回的状态码，从队列中获取一个对象，等等。

### 3. Collector

当 Span 或 Event 发生时，它们会被实现了 Collect 特征的收集器所记录或聚合。这个过程是通过通知的方式实现的：当 Event 发生或者 Span 开始/结束时，会调用 Collect 特征的相应方法通知 Collector。

我们前面提到只有使用了 tracing-subscriber 后，日志才能输出到控制台中。

之前大家可能还不理解，现在应该明白了，它是一个 Collector，可以将记录的日志收集后，再输出到控制台中。

## 使用方法

### 1. `span!` 宏

span! 宏可以用于创建一个 Span 结构体，然后通过调用结构体的 enter 方法来开始，再通过超出作用域时的 drop 来结束。

```rust
use tracing::{span, Level};
fn main() {
    let span = span!(Level::TRACE, "my_span");

    // `enter` 返回一个 RAII ，当其被 drop 时，将自动结束该 span
    let enter = span.enter();
    // 这里开始进入 `my_span` 的上下文
    // 下面执行一些任务，并记录一些信息到 `my_span` 中
    // ...
} // 这里 enter 将被 drop，`my_span` 也随之结束
```

### 2. `#[instrument]`

如果想要将某个函数的整个函数体都设置为 span 的范围，最简单的方法就是为函数标记上 `#[instrument]`，此时 tracing 会自动为函数创建一个 span，span 名跟函数名相同，在输出的信息中还会自动带上函数参数。

```rust
use tracing::{info, instrument};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[instrument]
fn foo(ans: i32) {
    info!("in foo");
}

fn main() {
    tracing_subscriber::registry().with(fmt::layer()).init();
    foo(42);
}
```

输出如下：

```bash
2022-04-10T02:44:12.885556Z  INFO foo{ans=42}: test_tracing: in foo
```

### 3. `in_scope`

对于没有内置 tracing 支持或者无法使用 `#instrument` 的函数，例如外部库的函数，我们可以使用 Span 结构体的 `in_scope` 方法，它可以将同步代码包裹在一个 span 中：

```rust
use tracing::info_span;

let json = info_span!("json.parse").in_scope(|| serde_json::from_slice(&buf))?;
```

### 4. 在 async 中使用 span

需要注意，如果是在异步编程时使用，要避免以下使用方式：

```rust
async fn my_async_function() {
    let span = info_span!("my_async_function");

    // WARNING: 该 span 直到 drop 后才结束，因此在 .await 期间，span 依然处于工作中状态
    let _enter = span.enter();

    // 在这里 span 依然在记录，但是 .await 会让出当前任务的执行权，然后运行时会去运行其它任务，此时这个 span 可能会记录其它任务的执行信息，最终记录了不正确的 trace 信息
    some_other_async_function().await

    // ...
}
```

我们建议使用如下方式，简单又有效：

```rust
use tracing::{info, instrument};
use tokio::{io::AsyncWriteExt, net::TcpStream};
use std::io;

#[instrument]
async fn write(stream: &mut TcpStream) -> io::Result<usize> {
    let result = stream.write(b"hello world\n").await;
    info!("wrote to stream; success={:?}", result.is_ok());
    result
}
```

那有同学可能要问了，是不是我们无法在异步代码中使用 span.enter 了，答案是：是也不是。

是，你无法直接使用 span.enter 语法了，原因上面也说过，但是可以通过下面的方式来曲线使用：

```rust
use tracing::Instrument;

let my_future = async {
    // ...
};

my_future
    .instrument(tracing::info_span!("my_future"))
    .await
```

### 5. span 嵌套

tracing 的 span 不仅仅是上面展示的基本用法，它们还可以进行嵌套！

```rust
use tracing::{debug, info, span, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    tracing_subscriber::registry().with(fmt::layer()).init();

    let scope = span!(Level::DEBUG, "foo");
    let _enter = scope.enter();
    info!("Hello in foo scope");
    debug!("before entering bar scope"); 
    {
        let scope = span!(Level::DEBUG, "bar", ans = 42);
        let _enter = scope.enter();
        debug!("enter bar scope");
        info!("In bar scope");
        debug!("end bar scope");
    }
    debug!("end bar scope");
}
```

日志输出如下：

```bash
INFO foo: log_test: Hello in foo scope
DEBUG foo: log_test: before entering bar scope
DEBUG foo:bar{ans=42}: log_test: enter bar scope
INFO foo:bar{ans=42}: log_test: In bar scope
DEBUG foo:bar{ans=42}: log_test: end bar scope
DEBUG foo: log_test: end bar scope
```

## 对宏进行配置

### 1. 日志级别和目标

span! 和 event! 宏都需要设定相应的日志级别，而且它们支持可选的 target 或 parent 参数( 只能二者选其一 )，该参数用于描述事件发生的位置，如果父 span 没有设置，target 参数也没有提供，那这个位置默认分别是当前的 span 和 当前的模块。

```rust
use tracing::{debug, info, span, Level,event};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    tracing_subscriber::registry().with(fmt::layer()).init();

    let s = span!(Level::TRACE, "my span");
    // 没进入 span，因此输出日志将不会带上 span 的信息
    event!(target: "app_events", Level::INFO, "something has happened 1!");

    // 进入 span ( 开始 )
    let _enter = s.enter();
    // 没有设置 target 和 parent
    // 这里的对象位置分别是当前的 span 名和模块名
    event!(Level::INFO, "something has happened 2!");
    // 设置了 target
    // 这里的对象位置分别是当前的 span 名和 target
    event!(target: "app_events",Level::INFO, "something has happened 3!");

    let span = span!(Level::TRACE, "my span 1");
    // 这里就更为复杂一些，留给大家作为思考题
    event!(parent: &span, Level::INFO, "something has happened 4!");
}
```

### 2. 记录字段

我们可以通过语法 field_name = field_value 来输出结构化的日志：

```rust
// 记录一个事件，带有两个字段:
//  - "answer", 值是 42
//  - "question", 值是 "life, the universe and everything"
event!(Level::INFO, answer = 42, question = "life, the universe, and everything");

// 日志输出 -> INFO test_tracing: answer=42 question="life, the universe, and everything"
```

### 3. 捕获环境变量

还可以捕获环境中的变量：

```rust
let user = "ferris";

// 下面的简写方式
span!(Level::TRACE, "login", user);
// 等价于:
span!(Level::TRACE, "login", user = user);
```

```rust
use tracing::{info, span, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    tracing_subscriber::registry().with(fmt::layer()).init();

    let user = "ferris";
    let s = span!(Level::TRACE, "login", user);
    let _enter = s.enter();

    info!(welcome="hello", user);
    // 下面一行将报错，原因是这种写法是格式化字符串的方式，必须使用 info!("hello {}", user)
    // info!("hello", user);
}

// 日志输出 -> INFO login{user="ferris"}: test_tracing: welcome="hello" user="ferris"
```

### 4. 字段名的多种形式

字段名还可以包含 `.`：

```rust
let user = "ferris";
let email = "ferris@rust-lang.org";
event!(Level::TRACE, user, user.email = email);

// 还可以使用结构体
let user = User {
    name: "ferris",
    email: "ferris@rust-lang.org",
};

// 直接访问结构体字段，无需赋值即可使用
span!(Level::TRACE, "login", user.name, user.email);

// 字段名还可以使用字符串
event!(Level::TRACE, "guid:x-request-id" = "abcdef", "type" = "request");

// 日志输出 -> 
// TRACE test_tracing: user="ferris" user.email="ferris@rust-lang.org"
// TRACE test_tracing: user.name="ferris" user.email="ferris@rust-lang.org"
// TRACE test_tracing: guid:x-request-id="abcdef" type="request"
```

`?` 表示该字段将使用 `fmt::Debug` 来格式化：

```rust
 #[derive(Debug)]
struct MyStruct {
    field: &'static str,
}

let my_struct = MyStruct {
    field: "Hello world!",
};

// `my_struct` 将使用 Debug 的形式输出
event!(Level::TRACE, greeting = ?my_struct);
// 等价于:
event!(Level::TRACE, greeting = tracing::field::debug(&my_struct));

// 下面代码将报错, my_struct 没有实现 Display
// event!(Level::TRACE, greeting = my_struct);

// 日志输出 -> TRACE test_tracing: greeting=MyStruct { field: "Hello world!" }
```

`%` 表示该字段将使用 `fmt::Display` 来格式化：

```rust
// `my_struct.field` 将使用 `fmt::Display` 的格式化形式输出
event!(Level::TRACE, greeting = %my_struct.field);
// 等价于:
event!(Level::TRACE, greeting = tracing::field::display(&my_struct.field));

// 作为对比，大家可以看下 Debug 和正常的字段输出长什么样
event!(Level::TRACE, greeting = ?my_struct.field);
event!(Level::TRACE, greeting = my_struct.field);

// 下面代码将报错, my_struct 没有实现 Display
// event!(Level::TRACE, greeting = %my_struct);
```

```bash
2022-04-10T03:49:00.834330Z TRACE test_tracing: greeting=Hello world!
2022-04-10T03:49:00.834410Z TRACE test_tracing: greeting=Hello world!
2022-04-10T03:49:00.834422Z TRACE test_tracing: greeting="Hello world!"
2022-04-10T03:49:00.834433Z TRACE test_tracing: greeting="Hello world!"
```

### 5. Empty

字段还能标记为 Empty，用于说明该字段目前没有任何值，但是可以在后面进行记录。

```rust
use tracing::{trace_span, field};

let span = trace_span!("my_span", greeting = "hello world", parting = field::Empty);

// ...

// 现在，为 parting 记录一个值
span.record("parting", &"goodbye world!");
```

### 6. 格式化字符串

除了以字段的方式记录信息，我们还可以使用格式化字符串的方式( 同 println! 、format! )。

> 注意，当字段跟格式化的方式混用时，必须把格式化放在最后，如下所示。

```rust
let question = "the ultimate question of life, the universe, and everything";
let answer = 42;
event!(
    Level::DEBUG,
    question.answer = answer,
    question.tricky = true,
    "the answer to {} is {}.", question, answer
);

// 日志输出 -> DEBUG test_tracing: the answer to the ultimate question of life, the universe, and everything is 42. question.answer=42 question.tricky=true
```

## 文件输出

截至目前，我们上面的日志都是输出到控制台中。

针对文件输出，tracing 提供了一个专门的库 `tracing-appender`。
