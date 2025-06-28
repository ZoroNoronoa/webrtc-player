# just

## 官网

<https://github.com/casey/just>

## 简介

just 应该可以在任何有合适的 sh 系统上运行，包括 Linux、MacOS 和 BSD。在 Windows 上，just 可以使用 Git for Windows、Github Desktop 或 Cygwin 所提供的 sh。

如果你不愿意安装 sh，也可以使用 shell 设置来指定你要使用的 shell。

比如 PowerShell：

```bash
# 使用 PowerShell 替代 sh:
set shell := ["powershell.exe", "-c"]

hello:
  Write-Host "Hello, world!"
```

或者 `cmd.exe`：

```bash
# 使用 cmd.exe 替代 sh:
set shell := ["cmd.exe", "/c"]

list:
  dir
```

也可以使用命令行来设置 just：

```bash
# Powershell
--shell powershell.exe --shell-arg -c
```

## 安装

我们直接使用 cargo 安装即可：

```bash
cargo install just
```

## Github Action

```bash
- uses: extractions/setup-just@v1
  with:
    just-version: 0.8 # optional semver specification, otherwise latest
```

或者：

```bash
- uses: taiki-e/install-action@just
```

## 用法

使用 just 时，它会在当前目录和父目录中查找文件 `justfile`，所以你可以在你项目的任何子目录中调用它。

### 1. 默认运行的 recipe

运行 just 不传参数时，会运行 justfile 中第一个 recipe。

一个例子：

```bash
build:
  cc main.c foo.c bar.c -o main

test: build
  ./test
```

不传参数的话会运行 `build`。

你也可以使用依赖关系来默认运行多个 recipe：

```bash
default: lint build test

build:
  echo Building…

test:
  echo Testing…

lint:
  echo Linting…
```

不传参数的话会同时执行 lint、build 和 test。

### 2. 列出可用的 recipe

```bash
just -l
```

### 3. 别名

别名允许你用其他名称来调用 recipe：

```bash
alias b := build

build:
  echo 'Building!'
```

运行：

```bash
$ just b
build
echo 'Building!'
Building!
```

### 4. 文档注释

紧接着配方前面的注释将出现在 just --list 中：

```bash
# build stuff
build:
  ./bin/build

# test stuff
test:
  ./bin/test
```

用法：

```bash
$ just --list
Available recipes:
    build # build stuff
    test # test stuff
```

### 5. 变量和替换

```bash
tmpdir  := `mktemp -d`
version := "0.2.7"
tardir  := tmpdir / "awesomesauce-" + version
tarball := tardir + ".tar.gz"

publish:
  rm -f {{tarball}}
  mkdir {{tardir}}
  cp README.md *.c {{tardir}}
  tar zcvf {{tarball}} {{tardir}}
  scp {{tarball}} me@server.com:release/
  rm -rf {{tarball}} {{tardir}}
```

### 6. 路径拼接

/ 操作符可用于通过斜线连接两个字符串：

```bash
foo := "a" / "b"
```

### 7. 错误忽略

通常情况下，如果一个命令返回一个非零的退出状态，将停止执行。要想在一个命令之后继续执行，即使它失败了，需要在命令前加上 `-`：

```bash
foo:
  -cat foo
  echo 'Done!'
```

用法：

```bash
$ just foo
cat foo
cat: foo: No such file or directory
echo 'Done!'
Done!
```

### 8. 函数

just 提供了一些内置函数，在编写 recipe 时可能很有用。

* 系统信息
  * `arch()`：指令集结构，可能是 aarch64、arm、wasm32、x86、x86_64 等
  * `os()`：操作系统，可能是 android、freebsd、ios、linux、macos、netbsd、openbsd、windows 等
  * `os_family()`：操作系统系列，可能是 unix 或 windows
* 环境变量
  * `env_var(key)`：获取名称为 key 的环境变量，不存在则终止
  * `env_var_or_default(key, default)`：获取名称为 key 的环境变量，不存在则返回 default
* 调用目录
  * `invocation_directory()`：获取 just 被调用时当前目录对应的绝对路径，在 just 改变路径并执行相应命令前
  * `justfile()`：取得当前 `justfile` 的路径
  * `justfile_directory()`：取得当前 `justfile` 文件父目录的路径
* `just_executable()`：just 可执行文件的绝对路径
* 字符串处理
  * `quote(s)`：用 `'\''` 替换所有的单引号，并在 `s` 的首尾添加单引号
  * `replace(s, from, to)`：将 `s` 中所有的 `from` 替换为 `to`
  * `replace_regex(s, regex, replacement)`：将 `s` 中所有匹配 `regex` 的子串替换为 `replacement`
  * `trim(s)`：去掉 `s` 两端的空白字符
  * `trim_end(s)`：去掉 `s` 结尾的空白字符
  * `trim_end_match(s, substr)`：删除与 `substr` 匹配的 `s` 的后缀
  * `trim_end_matches(s, substr)`：反复删除与 `substr` 匹配的 `s` 的后缀
  * `trim_start(s)`：去掉 `s` 开头的空白字符
  * `trim_start_match(s, substr)`：删除与 `substr` 匹配的 `s` 的前缀
  * `trim_start_matches(s, substr)`：反复删除与 `substr` 匹配的 `s` 的前缀
* 大小写转换
  * `capitalize(s)`：将 s 的第一个字符转成大写字母，其余的转成小写字母
  * `kebabcase(s)`：将 s 转成 kebab-case
  * `lowercamelcase(s)`：将 s 转换成小驼峰形式：`lowerCamelCase`
  * `lowercase(s)`：将 s 转成全小写
  * `shoutykebabcase(s)`：将 s 转换为 SHOUTY-KEBAB-CASE
  * `shoutysnakecase(s)`：将 s 转换为 SHOUTY_SNAKE_CASE
  * `snakecase(s)`：将 s 转换成 `snake_case`
  * `titlecase(s)`：将 s 转换成 `Title Case`
  * `uppercamelcase(s)`：将 s 转换成 `UpperCamelCase`
  * `uppercase(s)`：将 s 转成全大写
* 路径操作
  * 非可靠的（可能会失败）
    * `absolute_path(path)`：将当前目录中到相对路径 `path` 的路径转成绝对路径
    * `extension(path)`：获取 path 的扩展名
    * `file_name(path)`：获取 path 的文件名
    * `file_stem(path)`：获取 path 的文件名（不带扩展名）
    * `parent_directory(path)`：获取 path 的父目录
    * `without_extension(path)`：获取 path 不含扩展名的部分
  * 可靠的
    * `clean(path)`：删除多余的路径分隔符、中间的 `.` 和 `..` 来简化 path
    * `join(a, b...)`：将路径 a 和 b 拼接到一起
* 文件系统访问
  * `path_exists(path)`：如果路径指向一个存在的文件或者目录，就返回 true，否则返回 false
* 错误报告
  * `error(message)`：终止执行并向用户报告错误 message
* UUID 和哈希值生成
  * `sha256(string)`：以十六进制字符串返回 string 的 SHA-256 哈希值
  * `sha256_file(path)`：以十六进制字符串返回 path 处的文件的 SHA-256 哈希值
  * `uuid()`：返回一个随机生成的 uuid

### 8. receipt 属性

receipt 可以通过添加属性注释来改变其行为。

* `[no-cd]`：在执行配方前不要改变目录（just 通常在执行配方时将当前目录设置为包含 justfile 的目录，你可以通过这个属性禁用此行为）
* `[no-exit-message]`：如果配方执行失败，不要打印错误信息
* `[linux]`：在 Linux 上启用配方
* `[macos]`：在 MacOS 上启用配方
* `[unix]`：在 Unixes 上启用配方
* `[windows]`：在 Windows 上启用配方
* `[private]`：私有配方

### 9. 使用反引号求值

反引号可以用来存储命令的求值结果：

```bash
localhost := `dumpinterfaces | cut -d: -f2 | sed 's/\/.*//' | sed 's/ //g'`

serve:
  ./serve {{localhost}} 8080
```

缩进的反引号，以三个反引号为界，与字符串缩进的方式一样，会被去掉缩进：

````just
# This backtick evaluates the command `echo foo\necho bar\n`, which produces the value `foo\nbar\n`.
stuff := ```
    echo foo
    echo bar
  ```
````

### 10. 条件表达式

if / else 表达式评估不同的分支，取决于两个表达式是否评估为相同的值：

```bash
# 测试相等
foo := if "2" == "2" { "Good!" } else { "1984" }

bar:
  @echo "{{foo}}"

# 测试不相等
foo := if "hello" != "goodbye" { "xyz" } else { "abc" }

bar:
  @echo {{foo}}

# 测试正则表达式匹配
foo := if "hello" =~ 'hel+o' { "match" } else { "mismatch" }

bar:
  @echo {{foo}}
```

多个条件语句可以被连起来：

```bash
foo := if "hello" == "goodbye" {
  "xyz"
} else if "a" == "a" {
  "abc"
} else {
  "123"
}

bar:
  @echo {{foo}}
```

### 11. 从命令行设置变量

变量可以从命令行进行覆盖。

```bash
os := "linux"

test: build
  ./test --test {{os}}

build:
  ./build {{os}}
```

任何数量的 NAME=VALUE 形式的参数都可以在配方前传递：

```bash
$ just os=plan9
./build plan9
./test --test plan9
```

也可以使用 `--set` 标记：

```bash
$ just --set os bsd
./build bsd
./test --test bsd
```

### 12. 导出 just 变量

以 export 关键字为前缀的赋值将作为环境变量导出到配方中：

```bash
# 以 export 关键字为前缀的赋值将作为环境遍历导出到 recipe 中
export RUST_BACKTRACE := "1"

test:
  # 如果它崩溃了，将打印一个堆栈追踪
  cargo test
```

以 $ 为前缀的参数将被作为环境变量导出：

```bash
test $RUST_BACKTRACE="1":
  # 如果它崩溃了，将打印一个堆栈追踪
  cargo test
```

导出的变量和参数不会被导出到同一作用域内反引号包裹的表达式里：

```bash
export WORLD := "world"
# This backtick will fail with "WORLD: unbound variable"
BAR := `echo hello $WORLD`

# Running `just a foo` will fail with "A: unbound variable"
a $A $B=`echo $A`:
  echo $A $B
```

### 13. 从环境中获取环境变量

环境变量会自动传递给 recipe：

```bash
print_home_folder:
  echo "HOME is: '${HOME}'"
```

### 14. 从 .env 文件加载环境变量

如果 dotenv-load, dotenv-filename, dotenv-path, or dotenv-required 中任意一项被设置, just 会尝试从文件中加载环境变量。

如果没有找到环境变量文件也不会报错，除非设置了 dotenv-required。

从文件中加载的变量是环境变量，而非 just 变量，所以在配方和反引号中需要必须通过 $VARIABLE_NAME 来调用。

比如，如果你的 .env 文件包含以下内容：

```bash
# a comment, will be ignored
DATABASE_ADDRESS=localhost:6379
SERVER_PORT=1337
```

并且你的 justfile 包含：

```bash
set dotenv-load

serve:
  @echo "Starting server with database $DATABASE_ADDRESS on port $SERVER_PORT…"
  ./server --database $DATABASE_ADDRESS --port $SERVER_PORT
```

just serve 将会输出：

```bash
$ just serve
Starting server with database localhost:6379 on port 1337…
./server --database $DATABASE_ADDRESS --port $SERVER_PORT
```

### 15. recipe 参数

配方可以有参数。这里的配方 build 有一个参数叫 target:

```bash
build target:
  @echo 'Building {{target}}…'
  cd {{target}} && make
```

要在命令行上传递参数，请把它们放在配方名称后面：

```bash
$ just build my-awesome-project
Building my-awesome-project…
cd my-awesome-project && make
```

要向依赖配方传递参数，请将依赖配方和参数一起放在括号里：

```bash
default: (build "main")

build target:
  @echo 'Building {{target}}…'
  cd {{target}} && make
```

变量也可以作为参数传递给依赖：

```bash
target := "main"

_build version:
  @echo 'Building {{version}}…'
  cd {{version}} && make

build: (_build target)
```

命令的参数可以通过将依赖与参数一起放在括号中的方式传递给依赖：

```bash
build target:
  @echo "Building {{target}}…"

push target: (build target)
  @echo 'Pushing {{target}}…'
```

参数可以有默认值：

```bash
default := 'all'

test target tests=default:
  @echo 'Testing {{target}}:{{tests}}…'
  ./test --tests {{tests}} {{target}}
```

有默认值的参数可以省略：

```bash
$ just test server
Testing server:all…
./test --tests all server
```

或者提供：

```bash
$ just test server unit
Testing server:unit…
./test --tests unit server
```

默认值可以是任意的表达式，但字符串或路径拼接必须放在括号内：

```bash
arch := "wasm"

test triple=(arch + "-unknown-unknown") input=(arch / "input.dat"):
  ./test {{triple}}
```

配方的最后一个参数可以是变长的，在参数名称前用 + 或 * 表示：

```bash
backup +FILES:
  scp {{FILES}} me@server.com:
```

以 + 为前缀的变长参数接受 一个或多个 参数，并展开为一个包含这些参数的字符串，以空格分隔：

```bash
$ just backup FAQ.md GRAMMAR.md
scp FAQ.md GRAMMAR.md me@server.com:
FAQ.md                  100% 1831     1.8KB/s   00:00
GRAMMAR.md              100% 1666     1.6KB/s   00:00
```

以 * 为前缀的变长参数接受 0个或更多 参数，并展开为一个包含这些参数的字符串，以空格分隔，如果没有参数，则为空字符串：

```bash
commit MESSAGE *FLAGS:
  git commit {{FLAGS}} -m "{{MESSAGE}}"
```

变长参数可以被分配默认值。这些参数被命令行上传递的参数所覆盖：

```bash
test +FLAGS='-q':
  cargo test {{FLAGS}}
```

{{…}} 的替换可能需要加引号，如果它们包含空格。例如，如果你有以下配方：

```bash
search QUERY:
  lynx https://www.google.com/?q={{QUERY}}
```

{{…}} 的替换可能需要加引号，如果它们包含空格。例如，如果你有以下配方：

```bash
search QUERY:
  lynx https://www.google.com/?q={{QUERY}}
```

然后你输入：

```bash
just search "cat toupee"
```

just 将运行 lynx <https://www.google.com/?q=cat> toupee 命令，这将被 sh 解析为lynx、<https://www.google.com/?q=cat> 和 toupee，而不是原来的 lynx 和 <https://www.google.com/?q=cat> toupee。

你可以通过添加引号来解决这个问题：

```bash
search QUERY:
  lynx 'https://www.google.com/?q={{QUERY}}'
```

以 $ 为前缀的参数将被作为环境变量导出：

```bash
foo $bar:
  echo $bar
```

### 16. 在配方的末尾运行配方

一个配方的正常依赖总是在配方开始之前运行。也就是说，被依赖方总是在依赖方之前运行。这些依赖被称为 "前期依赖"。

一个配方也可以有后续的依赖，它们在配方之后运行，用 && 表示：

```bash
a:
  echo 'A!'

b: a && c d
  echo 'B!'

c:
  echo 'C!'

d:
  echo 'D!'
```

运行 b 输出：

```bash
$ just b
echo 'A!'
A!
echo 'B!'
B!
echo 'C!'
C!
echo 'D!'
D!
```

### 17. 在配方中间运行配方

just 不支持在配方的中间运行另一个配方，但你可以在一个配方的中间递归调用 just。例如以下 justfile：

```bash
a:
  echo 'A!'

b: a
  echo 'B start!'
  just c
  echo 'B end!'

c:
  echo 'C!'
```

运行 b 输出：

```bash
$ just b
echo 'A!'
A!
echo 'B start!'
B start!
echo 'C!'
C!
echo 'B end!'
B end!
```

这有局限性，因为配方 c 是以一个全新的 just 调用来运行的，赋值将被重新计算，依赖可能会运行两次，命令行参数不会被传入到子 just 进程。
