# Iris HTTP 进阶示例

这是独立的本地普通目录，不是新 submodule，也不依赖 `../http-server` 的源码、缓存或构建产物。
基础版 `demo/http-server` 仍在独立 Git 仓库中，本次仅增加英文边界注释并更新根源码摘要，协议行为不变。
本示例在 **127.0.0.1:8081** 展示真实的 Iris 类型、Contract、闭包、装饰器元数据和资源清理。
它不是 Web 框架或生产服务器，没有前端、认证、TLS、并发、keepalive 或请求正文解析。

## 学习路线与文件索引

| 顺序 / 特性 | 文件 | 可以观察到什么 |
| --- | --- | --- |
| 1. `Request` / `Response` 类型与只读属性访问器 | [src/model.ir](src/model.ir) | 构造器参数、访问器返回值有显式类型；不暴露 setter |
| 2. `HandlerContract`、`for`、`impl` | [src/model.ir](src/model.ir)、[src/application.ir](src/application.ir) | Router 声明处理器承诺，未知处理异常仍向上传播 |
| 3. 真实 `@PipelineTag(:advanced_v1)` 类装饰器 | [src/application.ir](src/application.ir) | 只读装饰器参数生成 `X-Iris-Pipeline: advanced_v1` |
| 4. 带类型闭包与模块组合 | [src/application.ir](src/application.ir) | `Closure<(Request) -> Response>` 捕获 handler，用 `.call` 执行，再映射指定异常 |
| 5. JSON 动态值的整数边界 | [src/application.ir](src/application.ir) | `let value: Integer = JSON.decode(...)` 拒绝布尔、字符串和容器 |
| 6. 有界纯协议管线 | [src/http.ir](src/http.ir) | 原始头段 -> Request -> Pipeline -> Response -> UTF-8 字节长度 |
| 7. 原生 TCP 与 `finally` | [src/main.ir](src/main.ir) | 监听器和每个客户端分别归属清理边界；部分写入按实际进度推进 |
| 8. 可执行断言 | [tests/test_http.ir](tests/test_http.ir)、[tests/pure.mjs](tests/pure.mjs)、[tests/live.mjs](tests/live.mjs) | 纯逻辑、元数据变更和真实 TCP 双引擎验证 |

公开纯入口为 `Http.response(rawHeaders: String) -> String`。
`Http.parse`、`Http.wire` 让学习者可以分别观察解析与编码；Response 只接受受信任的应用结果，
不是任意外部字符串的 HTTP 响应模板。网络输入永远是数据，绝不作为 Iris 源码执行。

## 四个路由

| 请求 | 响应 |
| --- | --- |
| `GET /` | 200，`你好，Iris!\n`，UTF-8 Content-Length 为 15 |
| `GET /health` | 200，JSON `{"status":"ok"}` 加换行 |
| `GET /double?value=21` | 200，JSON `{"doubled":42}` 加换行 |
| `GET /fail` | 500，受控的 `DemoFailure` 被转换为 `Controlled Failure\n` |
| 其他 GET 路径 | 404 |
| 非 GET | 405，`Allow: GET`；HEAD 的拒绝响应无正文 |

每个响应（包括 400/405/422/500）都有装饰器标记头。
`/double` 仅支持一个原样 `value=<JSON 标量>` 参数，不做百分号解码或 `+` 转空格。
值最多 128 字节，JSON 深度上限为 4；只接受 -1000 到 1000 的整数。
解码错误、缺失/多余查询字段返回 400；已解码的错误类型或越界整数返回 422。
当前核心 JSON 解码器把 `1.5` 视为语法错误，所以此例返回 400，而不是假装有完整 JSON 数值支持。
其他路由忽略查询串，路径大小写敏感，不进行规范化。

## 注解究竟做了什么

`PipelineTag` 是声明 `for ClassDecorator` 的真实 Iris 类；源码显式提供 `plan -> Plan.empty`
和 `transform -> Transformation.empty`。类声明上的 `@PipelineTag(:advanced_v1)`
被引擎记录为可反射的只读元数据，`Pipeline.marker()` 从 `Router.decorator_arguments` 读取 Symbol 参数，
由 `Http.wire` 输出响应头。它**不是**自动注册路由的 `@Get`，也不声称包装方法体。

纯测试只把受信任源码中的应用改成 `@PipelineTag(:experiment_v2)`，双引擎输出都随之变为
`X-Iris-Pipeline: experiment_v2`。这证明它不是无作用的装饰，也不是另一个硬编码字符串。
反射数组禁止 `.append`，测试验证 `ReadonlyMutationError`。

原型实际确认了当前实现限制：VM 拒绝方法装饰器（`the machine does not cover method decorator`）；
参考引擎能执行 `Transformation.empty.add_method`，VM 不执行这一运行期变换。
所以本例只使用已验证的**类装饰器元数据**，没有为示例修改语言实现。
两个空阶段只是明确声明不做变换，不代表本例验证了 VM 执行这两个阶段。

## 安装、授权构建、运行

需要兼容 Iris CLI、Git、Rust 1.93 或更新版本，自动测试另需 Node.js。
在 Iris-Language 根目录构建 CLI：

```sh
# macOS 本机如 cargo 不在 PATH：export PATH="$HOME/.cargo/bin:$PATH"
cargo build -p iris-cli --locked
```

下面命令从本示例目录执行，`IRIS` 使用绝对路径：

```sh
IRIS=/absolute/path/to/Iris-Language/target/debug/iris
"$IRIS" package install .
# 原生构建会执行受信任的第三方代码；审阅固定版本后再明确授权。
"$IRIS" package build . --allow-native-build
"$IRIS" package run . --allow native.load,native.blocking,network.tcp
# 换用 VM（不要同时启动两份占用 8081）：
"$IRIS" package run . --vm --allow native.load,native.blocking,network.tcp
```

`Network` 固定为 `c34aeb1044e999bd40f930dfde0ef898557ce1e0`。
`install` 只抓取并校验源码，不构建；`build` 单独授权；`run` 离线校验摘要后加载。
原生代码是可信进程代码，不是沙箱。不要把监听地址改成公网用于生产服务。
Windows 使用对应 `iris.exe` 绝对路径；`.gitattributes` 保持锁定文件 LF。

在另一个终端、30 秒空闲预算到期前：

```sh
curl -i http://127.0.0.1:8081/
curl -i 'http://127.0.0.1:8081/double?value=21'
curl -i 'http://127.0.0.1:8081/double?value=true'
curl -i http://127.0.0.1:8081/fail
```

## 测试与当前限制

```sh
node tests/pure.mjs "$IRIS"
node tests/live.mjs "$IRIS"
```

纯测试每引擎 **75** 个断言，另验证注解参数改变响应头，并记录非法 Contract 接收者的引擎诊断差异。
测试拼接的是本目录受信任源码，不包含 `main.ir`，不会启动 socket。
Live 每引擎 **42** 个实际 TCP 场景，逐字节比较整个响应，包括中文 UTF-8 长度、所有路由、
类型/范围错误、失败后恢复、方法/Host/长度/编码错误、分次写入、尾随数据、8192/8193 字节、
64/65 字段、EOF、100 ms 读取超时。完成后只向自己创建的服务器发送 SIGTERM，确认退出。
另起进程发送 **64 个短健康请求**，验证正常退出与 `finally` 清理。没有每次等待 30 秒的测试。
启动超时默认 60 秒，可用 `IRIS_STARTUP_TIMEOUT_MS` 正整数毫秒调整；启动/协议/退出失败均为非零状态。

当前 VM 有整次运行的 1,000,000 指令预算。大头段边界测试后再填满 64 个请求会耗尽预算，
因此边界套件和正常 64 连接退出套件分进程验证。基础版原有 `tests/live.mjs` 把这两者混在一起，
当前 VM 上会报 `StepBudgetExhausted`；本次未改动它。基础版 47 个纯测试双引擎通过，
原 17 个实际协议场景在测试拥有的进程主动关闭模式下双引擎通过，参考引擎原 64 连接套件通过。
基础版单独 64 个短健康请求正常退出也在双引擎通过；这些验证没有修改基础版测试文件。

Contract 声明与 `impl` 在源码中真实存在，但当前 VM 对非法 Contract 参数尚不完全执行边界检查：
`Pipeline.run("not a handler", request)` 在参考引擎返回 `TypeContractError`，VM 返回 `MessageNotFound`。
测试保留并显式报告这一差异。正常路由只传本地 Router 实例，网络数据无法选择 handler。
当前 VM 也未完整检查自动属性 setter；本例使用无 setter 的类型化访问器和已验证的构造器参数检查。
不能把此 demo 解读成所有 Iris 类型特性已完整实现。JSON 整数绑定错误在两引擎均经过实测。

没有配置 `.ir` LSP，JS 的 TypeScript LSP 在验证环境中也未安装；实际执行而非 LSP 是验收依据。
小型回环响应并不保证触发原生部分写入，客户端分次 write 也不保证形成多个 TCP 包。
SIGTERM 关闭不等于 Iris finally 执行证明，后者由独立正常退出套件覆盖。

## 协议与资源边界

- 最多 8192 字节（含 CRLFCRLF），64 个字段；先按字节检查，再分配解析集合。
- 请求行严格为 `METHOD SP origin-target SP HTTP/1.1`；拒绝绝对 URI、fragment、裸 CR/LF、控制字符、DEL 和非 ASCII 请求头。
- 恰好一个非空、大小写不敏感的 Host；拒绝明显空白、列表、用户信息/路径分隔符，不声称完整 authority 校验。
- 禁止 Transfer-Encoding；Content-Length 只允许单个十进制值。GET 必须为零；非 GET 直接 405，不等正文。
- 纯入口拒绝头终止符后的数据；socket 仅传第一个完整头段并丢弃已读尾部，每连接关闭，不支持流水线。
- listen/accept 位于 Iris，Network 只提供 TCP 原语。最多 64 个串行客户端，空闲 accept 30 秒后正常退出。
- 最多 64 次 read 和 64 次 write，每次超时 100 ms；约 6.4 秒/阶段的操作等待预算，不是严格墙钟 deadline。
- 只解码完整头段的 UTF-8；空输入、解码/读取失败成为 400，未知 native 异常继续传播。对端 reset 可能使响应无法送达。

## 锁文件与作者修改

`iris.lock` 的根记录覆盖 `iris.toml` 和 manifest 列出的四个源码文件，依赖记录覆盖固定提交的所有文件。
注释也改变精确字节摘要。本次基础版只按 `crates/iris-package/src/files.rs::tree_digest` 的格式
更新了根文件 SHA-256 和根树摘要，Network 记录、metadata 和 native artifact 摘要均不变；
随后 `package install` 和 `package run` 按正常完整性路径验证。进阶版先由 `package install` 生成锁。

此 CLI 没有 update-lock 命令，已有锁的 install 只校验，不会自动接纳修改。
作为消费者遇到 Integrity 错误时应检查字节、LF 与工作区变更，不要删除锁来绕过检查。
作者有意修改源码时，应独立生成/审核新的根快照，保持依赖记录不变，再运行 install 校验；
不要修改 native 摘要来让不匹配产物“通过”。`.iris/` 缓存和本机构建产物不入版本控制。
