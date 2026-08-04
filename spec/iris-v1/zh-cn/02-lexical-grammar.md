# Iris v1 词法语法

状态：Iris v1.2，冻结语义并有所有者批准的勘误。

IRIS-V1-GRAMMAR-C001：本章定义了 Iris v1 的规范源编码、词法标记集、字面量、保留关键字、上下文标记规则、优先级、结合性、声明头部、调用、块和 EBNF 语法。后面的语义章节 MUST 使用本章中的语法锚点和标记名称。

IRIS-V1-GRAMMAR-C002：历史文件（包括 `legacy/Document/IrisLangHighLight(for NP++).xml` 和旧版 `.ir` 脚本）仅作为证据。它们 MUST NOT 添加保留字、运算符、字面量形式或解析规则，超出批准草案中冻结的决定。

## 源文本

IRIS-V1-GRAMMAR-C003：Iris 源文本 MUST 是严格 UTF-8。源文件 MAY 以 UTF-8 BOM 开头。其他 BOM、UTF-16、本地编码和畸形的 UTF-8 MUST 被诊断为具有字节偏移量和源位置的 `LEX_INVALID_UTF8`。词法分析器 MUST NOT 将畸形的输入替换为 U+FFFD。

IRIS-V1-GRAMMAR-C004：物理换行符序列 LF、CRLF 和孤立的 CR 被识别为换行符。格式化程序 SHOULD 发出 LF。换行符参与语句终止，除非它被显式延续删除或者解析器位于仍需要更多标记的语法上下文中。

IRIS-V1-GRAMMAR-C005：以 `#!` 开头的物理第一行（可选地在 UTF-8 BOM 之后）是一个 shebang 注释。 `#!` 在任何其他源位置 MUST 都被诊断为 `LEX_SHEBANG_NOT_FIRST`。

IRIS-V1-GRAMMAR-C006：`//` 开始一个普通的行注释，在物理换行符之前结束。 `///` 启动文档行注释。 `/*` 和 `/**` 分别启动普通注释和文档块注释。块注释 MUST 嵌套，并且 MUST 以匹配的 `*/` 结尾。未终止的块注释 MUST 被诊断为 `LEX_UNTERMINATED_COMMENT`。

IRIS-V1-GRAMMAR-C007：仅当紧跟 LF、CRLF 或 CR 时，字面量外部的反斜杠才是显式行延续。词法分析器删除反斜杠和换行符。反斜杠后跟空格、注释或字面量 MUST 之外的任何其他标记将被诊断为 `LEX_BAD_CONTINUATION`。

IRIS-V1-GRAMMAR-C008：完整语句 MAY 以一个尾随分号结束，分号 MAY 分隔两个同一行语句。独立分号、重复分号，或语句前的旧版前导分号，MUST 按适用情况诊断为 `PARSE_EMPTY_STATEMENT` 或 `PARSE_LEGACY_LEADING_SEMICOLON`。

## 标识符与名称

IRIS-V1-GRAMMAR-C009：普通标识符区分大小写，并且在分配名称、选择器和 Type 身份之前，MUST 规范化为 Unicode NFC。标识符起始字符是 Unicode XID_Start 或 `_`。标识符继续字符是 Unicode XID_Continue 或 `_`。Pattern_Syntax、Pattern_White_Space、控制字符和默认可忽略码点即使由 Unicode 另行分类，也禁止出现在标识符中。

IRIS-V1-GRAMMAR-C010：可调用和属性选择器标识符 MAY 恰好以 `?` 或 `!` 结尾。该后缀是选择器标识的一部分。普通本地名称、参数名称、常量、Class 名称、Module 名称、Contract 名称、类型参数名称和原始 ivar 名称 MUST NOT 使用选择器后缀标点符号。

IRIS-V1-GRAMMAR-C011：原始 ivar 源语法为 `@`，后跟不带选择器后缀的普通标识符。 `@@` 后跟普通标识符表示在允许的语法位置中的共享或类级存储名称。 `$` 后跟普通标识符表示语法位置允许的全局名称。

IRIS-V1-GRAMMAR-C012：以 `:` 开头的简单 Symbol 字面量接受普通标识符、选择器标识符、IRIS-V1-GRAMMAR-C028 中列出的运算符符号或 `@` 形式的 ivar 名称加普通标识符。带引号的 Symbol 字面量 `:"..."` 使用 String 转义规则，不执行插值，并且在没有标识符 NFC 标准化的情况下准确保留转义后标量序列。

## 保留关键字

IRIS-V1-GRAMMAR-C013：根据 IRIS-V1-TRACE-C021 就地取代 v1.0 计数，v1.1 保留关键字集恰好包含 49 个小写单词。仅当完整的规范化标识符文本等于关键字时，符合要求的词法分析器 MUST 才会发出这些拼写的关键字标记。

| 关键字     | 关键字      | 关键字      | 关键字       |
| ----------- | ------------ | ------------ | ------------- |
| `class`   | `module`   | `contract` | `open`      |
| `extends` | `for`      | `mixin`    | `where`     |
| `meta`    | `deny`|`public`   | `protected` |
| `private` | `override` | `impl`     | `property`  |
| `shared`  | `key`      | `async`    | `await`     |
| `fun`     | `let`|`mut`      | `const`     |
| `global`  | `import`   | `from`     | `as`        |
| `export`  | `type`     | `if`       | `else`      |
| `while`   | `in`|`break`    | `continue`  |
| `match`   | `try`      | `catch`    | `finally`   |
| `raise`   | `return`   | `is`       | `nil`       |
| `true`    | `false`|`self`     | `super`     |
| `typeof`  |          |           |             |

IRIS-V1-GRAMMAR-C014：单词 `and`、`or`、`not`、`repeat`、`switch`、`when`、`groan`、 `order`、`serve`、`ignore`、`defer`、`implements`、`satisfies`、`interface`、`goto`、 `retry`、`redo`、`static`、`alias` 和 `undef` 并不只因历史而成为保留字。`new`、`using`、`close` 和 `method_missing` 是普通 Method 名称。词法分析器必须 (MUST) 将这些单词分类为标识符，除非本规范后续的另一条规则赋予它们上下文角色。

## 固定标记

IRIS-V1-GRAMMAR-C015：词法分析器 MUST 识别以下 69 个固定标记名称。某些名称会在不同语法上下文中共享同一拼写，例如除法斜杠与 Regex 开启斜杠。对于多字符固定标记，MUST 在发出较短标记前应用最长匹配。

| 标记名称              | 拼写 | 规则                                                                                               |
| ----------------------- | -------- | -------------------------------------------------------------------------------------------------- |
| `LPAREN`              | `(`    | 打开参数、分组、调用和 Tuple 上下文。                                               |
| `RPAREN`              | `)`    | 关闭 `LPAREN`。                                                                                  |
| `LBRACKET`            | `[`|打开索引、切片和 Array 字面量上下文。                                                    |
| `RBRACKET`            | `]`    | 关闭 `LBRACKET`。                                                                                |
| `LBRACE`              | `{`    | 打开块、声明体、Closure 字面量，以及上下文开启符之后的类 Hash 主体。 |
| `RBRACE`              | `}`    | 关闭 `LBRACE`。                                                                                  |
| `COMMA`               | `,`|分隔列表元素。                                                                           |
| `SEMICOLON`           | `;`    | 按 IRIS-V1-GRAMMAR-C008 分隔语句或作为语句尾随符。                                         |
| `COLON`               | `:`    | 按语法上下文开始 Symbol 字面量，或分隔标签、类型、键和约束。       |
| `DOUBLE_COLON`        | `::`   | 形成限定名称和声明槽。                                                       |
| `DOT`                 | `.`|开始属性/成员后缀语法，或参与数值字面量识别。              |
| `CONTRACT_VIEW`       | `..`   | 在 primary 上下文中开始 Contract-view 后缀语法。                                            |
| `HASH_OPEN`           | `%{`   | 打开 Hash 字面量。                                                                              |
| `INTERPOLATION_OPEN`  | `${`   | 仅在插值 String 或 Regex 文本内打开插值。                                 |
| `ARROW`               | `->`|分隔可调用头部与返回类型。                                                        |
| `MATCH_ARROW`         | `=>`   | 分隔 match 分支模式与主体。                                                             |
| `AT`                  | `@`    | 后跟普通标识符时开始原始 ivar 标记；在声明前缀位置，按 IRIS-V1-GRAMMAR-C058 开始 decorator。 |
| `DOUBLE_AT`           | `@@`   | 后跟普通标识符时开始共享存储标记。                             |
| `DOLLAR`              | `$`|后跟普通标识符时开始全局存储标记。                             |
| `QUESTION`            | `?`    | 仅作为选择器后缀出现，或出现在上下文相关的 `as?` 中。                                           |
| `BANG`                | `!`    | 不属于更长标记的一部分时，表示前缀逻辑否定或选择器后缀。                        |
| `PLUS`                | `+`    | 前缀或二元运算符。                                                                         |
| `MINUS`               | `-`|前缀或二元运算符。                                                                         |
| `STAR`                | `*`    | 按语法上下文表示二元运算符或位置 rest 标记。                                      |
| `STAR_STAR`           | `**`   | 指数运算符。                                                                                 |
| `SLASH`               | `/`    | 按语法上下文表示除法运算符或 Regex 分隔符。                                           |
| `AMP`                 | `&`|按语法上下文表示按位运算符、块通道标记或类型交集。                   |
| `PIPE`                | `&#124;` | 按语法上下文表示按位或运算符、块通道标记或类型并集。                                            |
| `CARET`               | `^`    | 按位异或运算符。                                                                              |
| `TILDE`               | `~`    | 前缀按位非运算符。                                                                       |
| `LT_LT`               | `<<`|左移运算符。                                                                               |
| `GT_GT`               | `>>`   | 右移运算符，或类型语法中的两个泛型闭合符。                                       |
| `LT`                  | `<`    | 关系运算符，或类型语法中的泛型开启符。                                             |
| `LT_EQ`               | `<=`   | 关系运算符。                                                                               |
| `GT`                  | `>`|关系运算符，或类型语法中的泛型闭合符。                                             |
| `GT_EQ`               | `>=`   | 关系运算符。                                                                               |
| `SPACESHIP`           | `<=>`  | 三路比较运算符。                                                                     |
| `REGEX_MATCH`         | `=~`   | Regex 匹配运算符。                                                                              |
| `REGEX_NOT_MATCH`     | `!~`|Regex 不匹配运算符。                                                                          |
| `EQ_EQ`               | `==`   | 相等运算符。                                                                                 |
| `BANG_EQ`             | `!=`   | 不等运算符。                                                                               |
| `AND_AND`             | `&&`   | 短路逻辑运算符。                                                                    |
| `PIPE_PIPE`           | `&#124;&#124;` | 短路逻辑或运算符。                                                                                 |
|`RANGE_INCLUSIVE`     | `..=`  | 包含 Range 运算符。                                                                          |
| `RANGE_EXCLUSIVE`     | `..<`  | 排除端点的 Range 运算符。                                                                          |
| `ASSIGN`              | `=`    | 按声明上下文表示赋值运算符或 setter 后缀。                                       |
| `PLUS_EQ`             | `+=`   | 复合赋值。|
|`MINUS_EQ`            | `-=`   | 复合赋值。                                                                               |
| `STAR_EQ`             | `*=`   | 复合赋值。                                                                               |
| `SLASH_EQ`            | `/=`   | 复合赋值。                                                                               |
| `STAR_STAR_EQ`        | `**=`  | 复合赋值。|
|`AMP_EQ`              | `&=`   | 复合赋值。                                                                               |
| `PIPE_EQ`             | `&#124;=` | 复合赋值。                                                                                         |
| `CARET_EQ`            | `^=`   | 复合赋值。                                                                               |
| `LT_LT_EQ`            | `<<=`  | 复合赋值。|
|`GT_GT_EQ`            | `>>=`  | 复合赋值。                                                                               |
| `AND_AND_EQ`          | `&&=`  | 逻辑赋值。                                                                                |
| `PIPE_PIPE_EQ`        | `&#124;&#124;=` | 逻辑赋值。                                                                                         |
| `AS_QUERY`            | `as?`  | 强制转换运算符被识别为上下文最长匹配。                                              |
| `RAW_PREFIX`|`r`    | 仅当紧跟有效引号或原始栅栏时才是字面量前缀。                       |
| `MUTABLE_PREFIX`      | `m`    | 可变字面量前缀仅在允许的字面量前缀序列中。                                   |
| `BYTES_PREFIX`        | `b`    | Bytes 字面量前缀仅出现在允许的字面量前缀序列中。                                     |
| `REGEX_RAW_PREFIX`    | `r/`   | Regex 字面量上下文中的原始 Regex 开启符。                                                         |
| `REGEX_OPEN`|`/`    | Regex 字面量上下文中的 Regex 开启符。                                                             |
| `QUOTE_DOUBLE`        | `"`    | String 字面量上下文中的分隔符。                                                               |
| `QUOTE_SINGLE`        | `'`    | String 字面量上下文中的分隔符。                                                               |
| `QUOTE_TRIPLE_DOUBLE` | `"""`  | 字面量上下文中的多行 String 分隔符。                                                     |
| `QUOTE_TRIPLE_SINGLE`|`'''`  | 字面量上下文中的多行 String 分隔符。                                                     |
| `BACKSLASH`           | `\`    | 在转义字面量内转义引入符或在字面量外显式延续。               |

IRIS-V1-GRAMMAR-C016：固定表达式运算符集包含 45 个固定拼写和 47 个表达式运算符形式，因为一元和二进制 `+` 是不同的形式，并且一元和二进制 `-` 是不同的形式。拼写为：`.`、`..`、`(`、`[`、`**`、`+`、`-`、 `~`，`!`，`*`，`/`，`<<`，`>>`，`&`，`^`， `|`，`..=`，`..<`，`<`，`<=`，`>`，`>=`，`<=>`， `=~`，`!~`，`is`，`as`，`as?`，`==`，`!=`，`&&`， `||`，`=`，`+=`，`-=`，`*=`，`/=`，`**=`，`&=`， `|=`、`^=`、`<<=`、`>>=`、`&&=` 和 `||=`。命名中缀 Method 语法是一种由选择器标识符携带的上下文运算符类，而不是固定的拼写。

## 最长匹配与命名冲突

IRIS-V1-GRAMMAR-C017：词法分析器 MUST 更喜欢 `..=` 和 `..<` 而不是 `..`； `!=` 超过 `!` 加上 `=`； `!~` 超过 `!` 加上 `~`； `**=` 超过 `**` 加上 `=`； `<<=` 超过 `<<` 加上 `=`； `>>=` 超过 `>>` 加上 `=`； `&&=` 超过 `&&` 加上 `=`； `||=` 超过 `||` 加上 `=`； `%{` 超过 `%` 加上 `{`； `${` 超过 `$` 加上 `{` 内插字面量文本；以及较短前缀上的所有其他最长固定标记。

IRIS-V1-GRAMMAR-C018：`..identifier` 是仅在主表达式之后的 Contract 视图后缀语法。 `..=` 和 `..<` 是仅在表达式之间的 Range 运算符。无法满足任一上下文 MUST 的标记序列将被诊断为 `PARSE_BAD_DOT_DOT_CONTEXT`，而不是重新解析为两个点。

IRIS-V1-GRAMMAR-C019：在表达式语法中，`a != b` 是不等式。在属性声明语法中，setter 选择器 MAY 在 `=` 之前包含选择器后缀，包括 `ready?=` 和 `value!=`。解析器 MUST 使用声明或属性分配上下文来识别完整的 setter 选择器，并且 MUST NOT 更改 `!=` 的表达式含义。

IRIS-V1-GRAMMAR-C020：通用尖括号仅在声明和类型语法上下文中被识别。表达式 `<`、`>`、`<=`、`>=`、`<<` 和 `>>` 保留运算符标记化。 Type 语法 MUST 允许 `>>` 关闭两个嵌套通用参数列表，而不更改表达式右移标记化。

IRIS-V1-GRAMMAR-C021：`%{` 是唯一的 Hash 字面量开场白。 `%` 本身并不是 v1 运算符。 `${` 仅在插入的 String 或 Regex 文本内才有意义。在该文本之外，`${` MUST 被诊断为 `LEX_INTERPOLATION_OUTSIDE_LITERAL`。

IRIS-V1-GRAMMAR-C022：字面量前缀是上下文相关的。 `m`、`b` 和 `r` 仍然是普通标识符，除非紧随其后的是合法的字面量前缀序列和分隔符。可变 String 原始前缀使用 `mr`。 Bytes 前缀使用 `b` 或 `br`。 ByteArray 前缀使用 `mb` 或 `mbr`。 Regex 原始前缀使用 `r/`。当序列 `rm`、`bm`、`rb`、`rbm` 和 `brm` 后跟字面量分隔符时，这些序列是无效的字面量前缀。

IRIS-V1-GRAMMAR-C023：Regex 字面量识别是上下文相关的。斜杠仅在需要主表达式的表达式开始位置开始 Regex 字面量。在表达式延续位置，斜杠是除法。 Regex 字面量文本 MUST 以带有有效尾随标志的未转义斜杠结尾。

## 数值字面量

IRIS-V1-GRAMMAR-C024：Integer 字面量支持二进制 `0b` 或 `0B`、八进制 `0o` 或 `0O`、不带前缀的十进制和十六进制 `0x` 或`0X`。无前缀十进制整数中的前导零是十进制，MUST NOT 选择八进制。

IRIS-V1-GRAMMAR-C025：数字分隔符仅在同一词汇数字段内有效的两个数字之间使用单个下划线。边界处与基数前缀、小数点、指数标记、指数符号或浮点后缀相邻的分隔符以及连续分隔符 MUST 被诊断为 `LEX_BAD_NUMERIC_SEPARATOR`。

IRIS-V1-GRAMMAR-C026：在基数前缀开始一个候选数值字面量后，词法分析器 MUST 消耗连续的候选片段以进行验证。`0b102`、`0o89` 和 `0xFG` 各自形成一个无效数值字面量标记，并且 MUST 以基数专属的 `LEX_INVALID_RADIX_DIGIT` 诊断。

IRIS-V1-GRAMMAR-C027：十进制浮点字面量 MAY 在小数点任一侧省略数字，只要至少一侧有数字。`.5`、`1.`、`1e3` 和 `2E-4` 是浮点字面量。单独的 `.` 是标点。十进制指数标记 `e` 或 `E` 后面 MUST 是可选的单个符号以及至少一位十进制数字。

IRIS-V1-GRAMMAR-C028：十六进制浮点字面量使用 `0x` 或 `0X` 十六进制有效数以及强制 `p` 或 `P` 二进制指数以及十进制指数位和可选符号。尾数可能是 `0x1p0`、`0x1.p0` 或 `0x.8p0`，但 MUST 至少包含一位十六进制数字。 `0x1.` 无效，`0x1e3` 是十六进制 Integer。

IRIS-V1-GRAMMAR-C029：无后缀的浮动字面量的类型为 `Float64`。小写后缀 `f32` 和 `f64` 选择 `Float32` 和 `Float64`。后缀 `F32` 和 `F64` MUST 诊断为 `LEX_BAD_FLOAT_SUFFIX`。

IRIS-V1-GRAMMAR-C030：十进制和十六进制源浮动字面量 MUST 使用正确舍入的 IEEE-754 `roundTiesToEven` 语义转换为其目标浮点宽度。字面量转换 MUST NOT 取决于主机区域设置、主机浮动环境、当前舍入模式、解释器与 JIT 模式、目标平台或不准确的主机解析器。

IRIS-V1-GRAMMAR-C031：浮点字面量溢出会产生带符号无穷大，足够小的非零字面量可能经由次正规值舍入为带符号零。这些都是有效字面量。当有限非零源字面量舍入为无穷大或零时，编译器 MUST 发出默认启用的精度警告；仅因为产生有限次正规值时，MUST NOT 发出警告。

## 文本、二进制、Symbol、Regex 与集合字面量

IRIS-V1-GRAMMAR-C032：转义双引号 String 字面量处理转义和 `${expr}` 插值。转义单引号 String 字面量会处理转义并且从不进行插值。三重双引号和三重单引号是具有相同插值区别的多行形式。当语句终止不介入时，相邻的 String 字面量段连接为一个表达式。

IRIS-V1-GRAMMAR-C033：转义的 String 字面量完全支持这些转义：`\\`、`\"`、`\'`、`\n`、`\r`、 `\t`、`\0`、`\b`、`\f`、`\v`、`\xNN` 和 `\u{H...}`（1 至 6 个十六进制数字）。 Unicode 转义 MUST 表示有效标量值，不包括代理项和高于 U+10FFFF 的值。未知或不完全的转义 MUST 被诊断为 `LEX_BAD_ESCAPE`。

IRIS-V1-GRAMMAR-C034：String 值仅包含有效的 Unicode 标量。 String 中的 `\xNN` 表示标量 U+0000 到 U+00FF，而不是原始字节。任意二进制数据属于 `Bytes` 或 `ByteArray` 字面量和 API。

IRIS-V1-GRAMMAR-C035：原始 String 字面量使用前缀 `r`，在引号分隔符之前包含 0 到 255 个 `#` 栅栏字符，在结束分隔符之后具有相同的栅栏计数。原始内容不执行转义，也不进行插值。围栏不匹配或超过 255 个围栏字符 MUST 被诊断为 `LEX_BAD_RAW_FENCE`。

IRIS-V1-GRAMMAR-C036：当开始三引号后紧跟换行符并且结束分隔符单独位于其行上时，三引号 String 字面量会应用严格的结束缩进剥离。第一个换行符被省略，最后一个换行符被省略，并且从每个非空内容行中删除结束定界符之前的确切空白前缀。缺少确切前缀 MUST 的非空内容行将被诊断为 `LEX_BAD_MULTILINE_INDENT`。

IRIS-V1-GRAMMAR-C037：MutableString 字面量在任何 String 字面量族前使用前缀 `m`。原始 MutableString 的前缀顺序是 `mr`。当后接 String 定界符时，`rm` 无效。每次求值 `m` 字面量都会在应用底层 String 字面量规则后产生一个新的 MutableString 身份。

IRIS-V1-GRAMMAR-C038：Bytes 字面量使用 `b` 加 String 字面量族，或 `br` 加原始 String 字面量族。ByteArray 字面量使用 `mb` 或 `mbr`，并采用相同的分隔符族。非 ASCII 源文本贡献其 UTF-8 字节。在字节字面量转义模式下，`\xNN` 注入一个原始字节。前缀顺序 MUST 是 mutable `m`、bytes `b`、可选 raw `r`；其他替代顺序无效。

IRIS-V1-GRAMMAR-C039：Regex 字面量使用 `/pattern/flags` 或原始 `r/pattern/flags`。插值 Regex 字面量接受 `${expr}`，并通过 `to_string` 转换插值值，随后应用默认 Regex 转义。原始 Regex 字面量不执行插值。标志是由 ASCII 字母组成的词法尾随标识符，并且 MUST 由 Regex 章节验证。

IRIS-V1-GRAMMAR-C040：Array 字面量使用 `[elements]`，Tuple 字面量使用带括号的逗号形式，Hash 字面量使用 `%{ entries }`，Range 字面量使用`expr ..= expr` 或 `expr ..< expr`。 Hash 字面量键是 `:` 之前的普通表达式；裸标识符键意味着绑定引用，而不是隐式的 Symbol。

## 运算符优先级与结合性

IRIS-V1-GRAMMAR-C041：优先级表是规范性的，并按从最高到最低排序。解析器 MUST 使用本条款引入的每一行来构建无歧义的表达式树。

| 等级 | 运算符或形式                                                                                        | 结合性                              | 链式使用                              |
| ---- | --------------------------------------------------------------------------------------------------------- | ------------------------------------------ | ------------------------------------- |
| 1    | primary、call `()`、index `[]`、property `.`、Contract-view `..`                                    | 左后缀                               | 允许链式                              |
| 2    | 指数 `**`                                                                                            | 右                                   | 右结合允许链式                        |
| 3    | 前缀一元 `+`、`-`、`~`、`!`                                                                          | 前缀                                 | 允许重复                              |
| 4    | 乘法 `*`、`/`                                                                                        | 左                                   | 允许链式                              |
| 5    | 加法 `+`、`-`                                                                                        | 左                                   | 允许链式                              |
| 6    | 移位 `<<`、`>>`                                                                                      | 左                                   | 允许链式                              |
| 7    | 按位 `&`                                                                                             | 左                                   | 允许链式                              |
| 8    | 按位 `^`                                                                                             | 左                                   | 允许链式                              |
| 9    | 按位 `&#124;`                                                                                        | 左                                   | 允许链式                              |
| 10   | Range `..=`、`..<`                                                                                   | 非结合                               | 不允许链式                            |
| 11   | 关系、类型和 Regex `<`、`<=`、`>`、`>=`、`<=>`、`=~`、`!~`、`is`、`as`、`as?`                         | 非结合                               | 不允许链式                            |
| 12   | 相等 `==`、`!=`                                                                                      | 非结合                               | 不允许链式                            |
| 13   | 命名中缀 Method                                                                                       | 左                                   | 允许链式                              |
| 14   | 逻辑 `&&`                                                                                            | 左                                   | 允许链式                              |
| 15   | 逻辑 `&#124;&#124;`                                                                                  | 左                                   | 允许链式                              |
| 16   | 赋值 `=`、`+=`、`-=`、`*=`、`/=`、`**=`、`&=`、`&#124;=`、`^=`、`<<=`、`>>=`、`&&=`、`&#124;&#124;=` | 右                                   | 右结合允许链式                        |

IRIS-V1-GRAMMAR-C042：求幂比前缀一元否定结合得更紧密。 `2 ** 3 ** 2` 解析为 `2 ** (3 ** 2)`。 `-2 ** 2` 解析为 `-(2 ** 2)`。 `2 ** -3` 解析为 `2 ** (-3)`。负基数需要括号。

IRIS-V1-GRAMMAR-C043：Range、关系、相等、`<=>`、Regex 匹配、`is`、`as` 和 `as?` 是非关联的。 `a < b < c`、`a == b == c`、`a ..= b ..= c` 或 `x as T as U` 等源 MUST 被诊断为 `PARSE_NONASSOCIATIVE_CHAIN`，除非括号或逻辑运算符使分组明确。

IRIS-V1-GRAMMAR-C044：命名中缀 Method 语法仅在 `selector` 是选择器标识符，并且周围语法能够提供两个操作数时，才在等级 13 将 `expression selector expression` 解析为中缀形式。Method 声明不能更改命名中缀的优先级或结合性。

IRIS-V1-GRAMMAR-C045：`!`、`&&`、`||`、`&&=` 和 `||=` 是核心控制流语法，MUST NOT被解析为可重载的 Method 选择器。 `&`、`|`、`^` 和 `~` 仍然是可重载操作员消息。

## EBNF 约定

IRIS-V1-GRAMMAR-C046：本章中的 EBNF 使用带引号的文本表示固定标记，小写名称表示词汇类别，大写单词表示标记类别，`?` 表示可选，`*` 表示重复，`+` 表示一个或多个。每个产生式对于解析器的形状来说都是完整的，而运行时的含义属于后面的章节。

IRIS-V1-GRAMMAR-C047：`terminator` 是语法感知换行符、IRIS-V1-GRAMMAR-C008 允许的分号或文件结尾。不匹配的分隔符内或不完整的语法产生式之后的换行符不是 `terminator` 标记。

## 词法 EBNF

IRIS-V1-GRAMMAR-C048：以下词汇产生式定义标识符、数字和字面量标记形状。

IRIS-V1-GRAMMAR-C048A：此表中的词法主体名称是原始扫描器类。它们故意不分解为解析器可见的产品，但每个名称都是为语法参考会计和一致性诊断而定义的。

| 原语                                    | 定义                                                                                                                              |
| -------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| `lexical_primitive`                        | 扫描仪识别的字符类或平衡字面量体，其详细验证由本章中的子句定义。      |
| `xid_start_or_underscore`                  | 在应用 IRIS-V1-GRAMMAR-C009 中的排除后，一个 Unicode XID_S 起始标量或 `_`。                                            |
| `xid_continue_or_underscore`               | 应用 IRIS-V1-GRAMMAR-C009 中的排除后，一个 Unicode XID_C 继续标量或 `_`。                                         |
| `binary_digit`|`0` 或 `1`。                                                                                                                         |
| `octal_digit`                              | `0` 至 `7`。                                                                                                                    |
| `decimal_digit`                            | `0` 至 `9`。                                                                                                                    |
| `hex_digit`                                | `0` 至 `9`、`a` 至 `f` 或 `A` 至 `F`。                                                                       |
| `ascii_letter`                             | `A` 至 `Z` 或 `a` 至 `z`。                                                                                             |
| `not_newline`                              | 不是 LF、CR 或 CRLF 开头的任何源标量。                                                                             |
| `newline`|LF、CRLF 或隔离的 CR。                                                                                                               |
| `eof`                                      | 源文件结束。                                                                                                                     |
| `whitespace`                               | 非换行符分隔空格。                                                                                                |
| `double_string`                            | 完整的转义双引号 String 正文，包括分隔符验证、转义、插值和换行符限制。         |
| `single_string`                            | 完整的转义单引号 String 主体，包括分隔符验证、转义和换行符限制，无插值。 |
| `triple_double_string`                     | 完整的三重双引号多行 String 正文，包括插值和严格的缩进处理。                         |
| `triple_single_string`|完整的三单引号多行 String 正文，包括严格的缩进处理和无插值。                      |
| `string_double_body_without_interpolation` | 引用的 Symbol 主体被 `:"..."` 接受，使用 String 转义但拒绝插值开头。                                 |
| `regex_text`                               | 完整的非原始 Regex 主体，具有转义、平衡插值和终止斜杠识别。                                  |
| `raw_regex_text`                           | 完整的原始 Regex 主体，具有终止斜杠识别且无插值。                                                      |

```ebnf
source_file        ::= bom? shebang? token_stream eof
bom                ::= "U+FEFF"
shebang            ::= "#!" not_newline* newline
token_stream       ::= (lexical_token | whitespace | newline)*
lexical_token      ::= identifier | ivar_name | shared_name | global_name | symbol_literal | integer_literal | float_literal | string_literal | bytes_literal | regex_literal | array_literal | hash_literal | fixed_token
fixed_token        ::= "fixed token from IRIS-V1-GRAMMAR-C015"
xid_start_or_underscore ::= lexical_primitive
xid_continue_or_underscore ::= lexical_primitive
binary_digit       ::= lexical_primitive
octal_digit        ::= lexical_primitive
decimal_digit      ::= lexical_primitive
hex_digit          ::= lexical_primitive
ascii_letter       ::= lexical_primitive
not_newline        ::= lexical_primitive
newline            ::= lexical_primitive
eof                ::= lexical_primitive
whitespace         ::= lexical_primitive
lexical_primitive  ::= "scanner primitive"
identifier         ::= xid_start_or_underscore xid_continue_or_underscore* selector_suffix?
ordinary_name      ::= xid_start_or_underscore xid_continue_or_underscore*
selector_suffix    ::= "?" | "!"
ivar_name          ::= "@" ordinary_name
shared_name        ::= "@@" ordinary_name
global_name        ::= "$" ordinary_name
symbol_literal     ::= ":" (ordinary_name selector_suffix? | ivar_name | operator_symbol | quoted_symbol)
quoted_symbol      ::= string_double_body_without_interpolation
operator_symbol    ::= "+" | "-" | "*" | "**" | "/" | "&" | "|" | "^" | "~" | "<<" | ">>" | "<" | "<=" | ">" | ">=" | "<=>" | "=~" | "!~" | "==" | "!=" | "same?"

integer_literal    ::= binary_integer | octal_integer | decimal_integer | hexadecimal_integer
binary_integer     ::= ("0b" | "0B") binary_digit digit_sep_binary*
octal_integer      ::= ("0o" | "0O") octal_digit digit_sep_octal*
decimal_integer    ::= decimal_digit digit_sep_decimal*
hexadecimal_integer ::= ("0x" | "0X") hex_digit digit_sep_hex*
float_literal      ::= decimal_float float_suffix? | hex_float float_suffix?
float_suffix       ::= "f32" | "f64"
decimal_float      ::= decimal_point_float decimal_exponent? | decimal_digit digit_sep_decimal* decimal_exponent
decimal_point_float ::= decimal_digit digit_sep_decimal* "." decimal_fraction_digits? | decimal_digit digit_sep_decimal* "." | "." decimal_fraction_digits
decimal_fraction_digits ::= decimal_digit digit_sep_decimal*
decimal_exponent   ::= ("e" | "E") ("+" | "-")? decimal_digit digit_sep_decimal*
hex_float          ::= ("0x" | "0X") hex_significand ("p" | "P") ("+" | "-")? decimal_digit digit_sep_decimal*
hex_significand    ::= hex_digit digit_sep_hex* | hex_digit digit_sep_hex* "." hex_fraction_digits? | hex_digit digit_sep_hex* "." | "." hex_fraction_digits
hex_fraction_digits ::= hex_digit digit_sep_hex*
digit_sep_binary   ::= binary_digit | "_" binary_digit
digit_sep_octal    ::= octal_digit | "_" octal_digit
digit_sep_decimal  ::= decimal_digit | "_" decimal_digit
digit_sep_hex      ::= hex_digit | "_" hex_digit

string_literal     ::= string_prefix? string_body
string_prefix      ::= "r" raw_fence? | "m" | "mr" raw_fence?
bytes_literal      ::= bytes_prefix string_body
bytes_prefix       ::= "b" | "br" raw_fence? | "mb" | "mbr" raw_fence?
raw_fence          ::= "#"{0,255}
string_body        ::= double_string | single_string | triple_double_string | triple_single_string
double_string      ::= lexical_primitive
single_string      ::= lexical_primitive
triple_double_string ::= lexical_primitive
triple_single_string ::= lexical_primitive
string_double_body_without_interpolation ::= lexical_primitive
regex_literal      ::= "/" regex_text "/" regex_flags? | "r" raw_fence? "/" raw_regex_text "/" raw_fence? regex_flags?
regex_text         ::= lexical_primitive
raw_regex_text     ::= lexical_primitive
regex_flags        ::= ascii_letter*
array_literal      ::= "[" argument_list? "]"
argument_list      ::= expression ("," expression)* ","?
hash_literal       ::= "%{" hash_entry_list? "}"
hash_entry_list    ::= hash_entry ("," hash_entry)* ","?
hash_entry         ::= expression ":" expression
```

## 解析器 EBNF

IRIS-V1-GRAMMAR-C049：以下解析器产生式为明确的 Iris v1 解析器充分定义了声明、调用、块、语句和表达式。

```ebnf
program            ::= terminator* declaration_or_statement (terminator+ declaration_or_statement)* terminator*
terminator         ::= newline | ";" | eof
declaration_or_statement ::= declaration | statement

declaration        ::= decorated_declaration | import_decl | export_decl | type_alias_decl | global_decl | shared_decl | let_decl
decorated_declaration ::= decorator* (class_decl | module_decl | contract_decl | method_decl | property_decl)
decorator          ::= "@" qualified_type_name "(" call_argument_list? ")"
import_decl        ::= "override"? "import" import_path import_alias? | "override"? "from" import_path "import" import_spec_list
import_path        ::= package_qualified_name | qualified_type_name
package_qualified_name ::= package_name "::" qualified_type_name
package_name       ::= ordinary_name ("." ordinary_name)*
import_alias       ::= "as" ordinary_name
import_spec_list   ::= import_spec ("," import_spec)* ","?
import_spec        ::= ordinary_name import_alias?
export_decl        ::= "export" (declaration | ordinary_name ("," ordinary_name)* ","?)
type_alias_decl    ::= "type" type_name generic_params? "=" type_expr where_clause?
global_decl        ::= "global" ("let" | "mut") global_name type_annotation? "=" expression
shared_decl        ::= "shared" ("let" | "mut") shared_name type_annotation? "=" expression
class_decl         ::= "open"? "class" type_name generic_params? class_extends? class_for? class_mixin? where_clause? meta_clause? declaration_body
class_extends      ::= "extends" type_expr
class_for          ::= "for" type_expr_list
class_mixin        ::= "mixin" mixin_entry_list
module_decl        ::= "open"? "module" type_name generic_params? module_mixin? where_clause? meta_clause? declaration_body
module_mixin       ::= "mixin" mixin_entry_list
mixin_entry_list   ::= mixin_entry ("," mixin_entry)* ","?
mixin_entry        ::= type_expr "private"?
contract_decl      ::= "contract" type_name generic_params? contract_extends? where_clause? meta_clause? declaration_body
contract_extends   ::= "extends" type_expr_list
meta_clause        ::= "meta" "deny" meta_capability_list
meta_capability_list ::= meta_capability ("," meta_capability)* ","?
meta_capability    ::= ordinary_name
declaration_body   ::= "{" terminator* declaration_or_statement* "}"

method_decl        ::= visibility? "override"? "impl"? "async"? ("class" | "module")? "fun" selector generic_params? parameter_list return_type? where_clause? block_body?
property_decl      ::= visibility? "override"? "impl"? "shared"? ("class" | "module")? "property" (stored_property_decl | property_accessor_decl)
stored_property_decl ::= ordinary_name ":" type_expr ("=" expression)? property_accessor_block?
property_accessor_block ::= "{" property_accessor_member* "}"
property_accessor_member ::= visibility? "get" ";" | visibility? "set" ";"
property_accessor_decl ::= "fun" property_selector parameter_list return_type? where_clause? block_body
property_selector  ::= selector "="?
visibility         ::= "public" | "protected" | "private"
parameter_list     ::= "(" parameter_sequence? ")"
parameter_sequence ::= required_positional* optional_positional* rest_positional? required_keyword* optional_keyword* rest_keyword? block_parameter?
required_positional ::= ordinary_name ":" type_expr ","?
optional_positional ::= ordinary_name ":" type_expr "=" expression ","?
rest_positional    ::= "*" ordinary_name ":" type_expr ","?
required_keyword   ::= "key" ordinary_name ":" type_expr ","?
optional_keyword   ::= "key" ordinary_name ":" type_expr "=" expression ","?
rest_keyword       ::= "**" ordinary_name ":" type_expr ","?
block_parameter    ::= "&" ordinary_name ":" callable_type ("=" "nil")? ","?
return_type        ::= "->" type_expr
block_body         ::= "{" terminator* statement_list? "}"

statement_list     ::= statement (terminator+ statement)* terminator*
statement          ::= let_decl | expression_statement | return_statement | break_statement | continue_statement | raise_statement | if_statement | while_statement | for_statement | match_statement | try_statement
let_decl           ::= ("let" | "mut" | "const") binding_pattern type_annotation? ("=" expression)?
type_annotation    ::= ":" type_expr
expression_statement ::= expression
return_statement   ::= "return" expression?
break_statement    ::= "break" break_payload?
break_payload      ::= labeled_break_payload | expression
labeled_break_payload ::= ordinary_name ":" expression
continue_statement ::= "continue" ordinary_name?
raise_statement    ::= "raise" | "raise" expression raise_cause?
raise_cause        ::= "from" expression
if_statement       ::= if_expression
while_statement    ::= loop_label? "while" expression block_body
for_statement      ::= loop_label? "for" binding_pattern "in" expression block_body
loop_label         ::= ordinary_name ":"
match_statement    ::= "match" expression "{" terminator* match_arm+ match_fallback? "}"
match_arm          ::= match_pattern match_guard? "=>" match_body match_separator
match_fallback     ::= "else" "=>" match_body match_separator?
match_guard        ::= "if" expression
match_body         ::= expression | block_body
match_separator    ::= "," terminator* | terminator+
try_statement      ::= "try" block_body catch_clause* finally_clause?
catch_clause       ::= "catch" (catch_binding (":" type_expr)? ("," ordinary_name)?)? block_body
finally_clause     ::= "finally" block_body

expression         ::= assignment_expr
assignment_expr    ::= assignment_target assignment_operator assignment_expr | logical_or_expr
assignment_operator ::= "=" | "+=" | "-=" | "*=" | "/=" | "**=" | "&=" | "|=" | "^=" | "<<=" | ">>=" | "&&=" | "||="
assignment_target  ::= ordinary_name | ivar_name | shared_name | global_name | member_assignment_target | index_assignment_target
member_assignment_target ::= postfix_expr property_suffix
index_assignment_target ::= postfix_expr index_suffix
logical_or_expr    ::= logical_and_expr ("||" logical_and_expr)*
logical_and_expr   ::= named_infix_expr ("&&" named_infix_expr)*
named_infix_expr   ::= equality_expr (selector equality_expr)*
equality_expr      ::= relational_expr (("==" | "!=") relational_expr)?
relational_expr    ::= range_expr (("<" | "<=" | ">" | ">=" | "<=>" | "=~" | "!~" | "is" | "as" | "as?") range_expr)?
range_expr         ::= bitwise_or_expr (("..=" | "..<") bitwise_or_expr)?
bitwise_or_expr    ::= bitwise_xor_expr ("|" bitwise_xor_expr)*
bitwise_xor_expr   ::= bitwise_and_expr ("^" bitwise_and_expr)*
bitwise_and_expr   ::= shift_expr ("&" shift_expr)*
shift_expr         ::= additive_expr (("<<" | ">>") additive_expr)*
additive_expr      ::= multiplicative_expr (("+" | "-") multiplicative_expr)*
multiplicative_expr ::= unary_expr (("*" | "/") unary_expr)*
unary_expr         ::= ("+" | "-" | "~" | "!") unary_expr | exponent_expr
exponent_expr      ::= postfix_expr ("**" unary_expr)?
postfix_expr       ::= primary_expr postfix_part*
postfix_part       ::= call_suffix | index_suffix | property_suffix | contract_view_suffix | trailing_block
call_suffix        ::= call_type_arguments? "(" call_argument_list? ")"
call_type_arguments ::= "<" call_type_argument ("," call_type_argument)* ">"
call_type_argument ::= type_expr | "_"
call_argument_list ::= call_argument ("," call_argument)* ","?
call_argument      ::= expression | ordinary_name ":" expression | "*" expression | "**" expression | "&" expression
index_suffix       ::= "[" call_argument_list? "]"
property_suffix    ::= "." selector
contract_view_suffix ::= ".." selector call_suffix?
trailing_block     ::= closure_literal
primary_expr       ::= literal | ordinary_name | ivar_name | shared_name | global_name | "self" | "super" | "nil" | "true" | "false" | grouped_or_tuple | array_literal | hash_literal | closure_literal | if_expression | closed_generic_name | reified_type_expr
closed_generic_name ::= qualified_type_name generic_args
reified_type_expr  ::= "(" type_expr ")" &"." "type"
if_expression      ::= "if" expression block_body ("else" (if_expression | block_body))?
grouped_or_tuple   ::= "(" expression ("," expression)* ","? ")"
closure_literal    ::= "{" closure_header? closure_body "}"
closure_header     ::= "|" closure_parameters? "|" return_type? (terminator | ";")
closure_parameters ::= parameter_sequence
closure_body       ::= statement_list?

generic_params     ::= "<" ordinary_name ("," ordinary_name)* ">"
where_clause       ::= "where" constraint_assignment ("," constraint_assignment)*
constraint_assignment ::= (ordinary_name | "Self") ":" type_expr
type_expr_list     ::= type_expr ("," type_expr)*
type_expr          ::= type_union
type_union         ::= type_intersection ("|" type_intersection)*
type_intersection  ::= type_postfix ("&" type_postfix)*
type_postfix       ::= type_primary "?"?
type_primary       ::= typeof_type | qualified_type_name generic_args? | callable_type | "(" type_expr ")"
typeof_type        ::= "typeof" "(" expression ")"
qualified_type_name ::= type_name ("::" type_name)*
generic_args       ::= "<" type_expr ("," type_expr)* ">"
function_type      ::= "(" type_expr_list? ")" "->" type_expr
callable_type      ::= ("Closure" | "BoundMethod" | "Block") "<" function_type ">"
binding_pattern    ::= ordinary_name | "_" | "(" binding_pattern ("," binding_pattern)* ","? ")" | "[" binding_pattern ("," binding_pattern)* rest_binding_pattern? ","? "]"
rest_binding_pattern ::= "," "*" binding_pattern
catch_binding      ::= ordinary_name | "_"
pattern            ::= binding_pattern | literal | type_pattern
match_pattern      ::= match_pattern_alternative ("|" match_pattern_alternative)*
match_pattern_alternative ::= literal | "nil" | "true" | "false" | "is" match_type_expr binding_pattern? | binding_pattern | match_tuple_pattern | match_array_pattern
match_type_expr    ::= type_intersection | type_primary
match_tuple_pattern ::= "(" match_pattern ("," match_pattern)+ ","? ")"
match_array_pattern ::= "[" match_pattern ("," match_pattern)* match_rest_pattern? ","? "]"
match_rest_pattern ::= "," "*" binding_pattern
type_pattern       ::= type_expr
selector           ::= ordinary_name selector_suffix?
type_name          ::= ordinary_name
literal            ::= integer_literal | float_literal | string_literal | bytes_literal | symbol_literal | regex_literal
```

IRIS-V1-GRAMMAR-C050：普通调用需要括号。属性 getter 仅在属性/成员语法中省略括号。尾随 Closure 遵循完整的调用或后缀表达式，并通过专用 `&block` 通道进行绑定。调用 MUST 最多包含一个块，要么是参数列表中的 `&expr`，要么是一个尾随的 Closure。

IRIS-V1-GRAMMAR-C051：Closure 字面量使用 `{ |parameters| -> ReturnType body }`。空参数使用 `{ || -> R body }`。单行 Closure 主体在 `;` 之后开始；多行主体在头部终止符之后开始。仅当预期的可调用上下文唯一提供封闭类型时，返回注解 MAY 省略。

IRIS-V1-GRAMMAR-C052：Class 标头子句顺序固定为通用参数，`extends`、`for`、`mixin`、`where`、`meta`，然后是正文。 Module 标头子句顺序是通用参数，`mixin`、`where`、`meta`，然后是主体。 Contract header子句顺序是通用参数，`extends`，`where`，`meta`，然后是主体。每个允许的子句类型 MAY 最多出现一次。 `meta deny` 子句只是静态标头语法，MUST NOT 出现在可执行或条件主体代码中。 `meta` 子句的功能名称被解析为普通名称，其词汇和拒绝语义由元编程章节指定。

IRIS-V1-GRAMMAR-C053：参数声明顺序是必需的位置，可选的位置，至多一个位置休息，必需的仅关键字，可选的仅关键字，最多一个关键字休息，然后是尾随块绑定。调用将关键字参数拼写为 `name: value`，并且 MUST 使用普通调用语法的括号。

IRIS-V1-GRAMMAR-C053A：循环标签只能以 `ordinary_name :` 的源形式表达，并且必须紧挨在 `while` 或 `for` 之前。标签前缀 MUST NOT 应用于非循环语句。单独的 `break` 是裸 break。无标签 break 使用普通的 `break expression`。带标签 break 使用 `break ordinary_name : expression`，例如 `break outer: value`；标签后的冒号使它在语法上不同于不受限制的 `break expr`，包括作为 break 值的命名中缀表达式。解析 MUST NOT 查询已声明标签名或任何符号表。`continue name` 是带标签 continue 形式，裸 `continue` 以最近的循环为目标。标签目标有效性、重复标签错误、Closure 边界限制和结果类型属于控制流语义。

IRIS-V1-GRAMMAR-C053B：match 分支使用 `pattern [if guard] => expression-or-block`，按源顺序排列，并可带一个最终的可选 `else => expression-or-block` fallback。`match_pattern` 顶层的 `|` 分隔模式替代项。`is` 模式使用 `match_type_expr`，它排除顶层并集类型语法；当意图使用并集类型时，`is` 模式中的并集类型 MUST 写成带括号的 `is (A | B)`。fallback 分支 MUST 位于最后，因为 `else` 只由 `match_fallback` 解析，而不由 `match_pattern` 解析。分支分隔符是 `match_separator` 定义的逗号和/或语句终止符。穷尽性、guard 真值测试、绑定兼容性和解构失败行为属于控制流语义。

## 畸形标记与语法的诊断

IRIS-V1-GRAMMAR-C054：符合要求的诊断系统 MUST 至少使用这些稳定类别对畸形词法和语法案例进行分类。工具可以 (MAY) 添加更多详细信息，但主类别 MUST 对符合性向量保持稳定。

| 类别                              | 必需触发条件                                                                                                          |
| ------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `LEX_INVALID_UTF8`                  | UTF-8 畸形、BOM 错误或非 UTF-8 源编码。                                                                 |
| `LEX_SHEBANG_NOT_FIRST`             | `#!` 在第一条物理线之外。                                                                                   |
| `LEX_UNTERMINATED_COMMENT`          | 嵌套块注释到达 EOF，但没有匹配的结束。                                                                |
| `LEX_BAD_CONTINUATION`              | 字面量外部的反斜杠后面不会紧跟着换行符序列。                                            |
| `LEX_INVALID_RADIX_DIGIT`|候选基数字面量包含对该基数无效的数字或标识符字符。                                  |
| `LEX_EMPTY_RADIX_PREFIX`            | 基数前缀没有立即有效的数字。                                                                              |
| `LEX_BAD_NUMERIC_SEPARATOR`         | 数字分隔符放置违反了 IRIS-V1-GRAMMAR-C025。                                                                |
| `LEX_BAD_FLOAT_SUFFIX`              | 浮点后缀不是小写`f32` 或`f64`。                                                                          |
| `LEX_BAD_ESCAPE`                    | 转义字面量包含未知、不完整或无效的标量转义。                                                |
| `LEX_UNTERMINATED_LITERAL`          | String、Bytes、ByteArray、Symbol 或 Regex 字面量在关闭前到达 EOF 或无效换行符。                       |
| `LEX_BAD_RAW_FENCE`|原始栅栏计数不匹配或超过 255。                                                                                |
| `LEX_BAD_MULTILINE_INDENT`          | 不能应用三重字面量严格的结束缩进剥离。                                                         |
| `LEX_INTERPOLATION_OUTSIDE_LITERAL` | `${` 出现在插值字面量文本之外。                                                                         |
| `LEX_BAD_LITERAL_PREFIX`            | 字面量前缀顺序或系列无效，例如`rm`、`bm`、`rb` 或`brm`。                                     |
| `LEX_BAD_REGEX_FLAGS`               | Regex 标志包含无效字符或重复的不受支持的标志。                                                    |
| `PARSE_EMPTY_STATEMENT`             | 独立或重复的分号会创建一个空语句。                                                              |
| `PARSE_LEGACY_LEADING_SEMICOLON`|语句以旧版前导分号开头。                                                                           |
| `PARSE_BAD_DOT_DOT_CONTEXT`         | `..` 不能用作 Contract 视图，也不能是 Range 令牌。                                                         |
| `PARSE_NONASSOCIATIVE_CHAIN`        | 非关联运算符被链接起来，没有括号或逻辑组合。                                         |
| `PARSE_BAD_GENERIC_CONTEXT`         | 通用尖括号出现在需要表达式运算符的位置，或者 `>>` 无法关闭预期的类型参数。 |
| `PARSE_BAD_CALL_BLOCK`              | 一次调用提供多个块通道。                                                                              |
| `PARSE_BAD_HEADER_ORDER`            | Class、Module 或 Contract 标头子句无序或重复。                                                   |
| `PARSE_BAD_PARAMETER_ORDER`|参数类别无序或重复超出允许的休息位置。                                          |

## 正向示例与畸形向量

IRIS-V1-GRAMMAR-EX001：信息示例，数字优先级：

```iris
value = -2 ** 2 + 3 << 1
same = 2 ** -3
```

IRIS-V1-GRAMMAR-EX002：信息示例、声明和调用：

```iris
class Box<T> extends Object for Printable mixin Trace where T: Object {
  property fun ready?() -> Bool { true }
  property fun value!=(next: T) -> Nil { @value = next }
  fun map<U>(value: T, key label: Symbol, &block: Block<(T) -> U>) -> U where U: Object {
    block.call(value)
  }
}
```

IRIS-V1-GRAMMAR-EX003：信息示例、字面量和上下文冲突：

```iris
name = :@slot
text = m"hello ${user}"
bytes = mbr#"\xff"#
range = 0 ..< 10
match = /item-${name}/i
table = %{ :name: name, 1 + 2: "sum" }
view = parser..parse(input)
```

IRIS-V1-GRAMMAR-V001：正向量 `grammar.keyword.inventory` 期望 IRIS-V1-GRAMMAR-C013 中正好有 48 个保留关键字，并拒绝 IRIS-V1-GRAMMAR-C014 中的历史非关键字。

IRIS-V1-GRAMMAR-V002：正向量 `grammar.operator.inventory` 需要来自 IRIS-V1-GRAMMAR-C015 的 69 个固定标记名称、来自 IRIS-V1-GRAMMAR-C016 的 45 个固定表达式运算符拼写和 47 个表达式运算符形式，以及一个上下文命名中缀运算符类。

IRIS-V1-GRAMMAR-V004：正向量 `grammar.conflict.longest-match` 分类 `a ..= b`、`a ..< b`、`view..member`、`a != b`、`property fun value!=(v: T)`、`Box<Array<String>>`、内插字面量内的 `%{}`、`${x}`，表达式开始上下文中的 `m"x"`、`br"x"`、`mbr"x"` 和 `/x/` 根据 IRIS-V1-GRAMMAR-C017 通过IRIS-V1-GRAMMAR-C023。

IRIS-V1-GRAMMAR-V005：畸形的向量 `grammar.bad-numeric` 将 `0b102` 映射到 `LEX_INVALID_RADIX_DIGIT`，将 `0x` 映射到 `LEX_EMPTY_RADIX_PREFIX`，将 `1__0` 映射到 `LEX_BAD_NUMERIC_SEPARATOR`， `1e+_3` 至 `LEX_BAD_NUMERIC_SEPARATOR`，以及 `1.0F32` 至 `LEX_BAD_FLOAT_SUFFIX`。

IRIS-V1-GRAMMAR-V006：畸形的向量 `grammar.bad-literals` 将 `"\q"` 映射到 `LEX_BAD_ESCAPE`，将 `r###"x"##` 映射到 `LEX_BAD_RAW_FENCE`，将 `rm"x"` 映射到 `LEX_BAD_LITERAL_PREFIX`，未终止的 `/abc` Regex 字面量为 `LEX_UNTERMINATED_LITERAL`，`${x}` 外部字面量为 `LEX_INTERPOLATION_OUTSIDE_LITERAL`。

IRIS-V1-GRAMMAR-V007：畸形的向量 `grammar.bad-parse` 将 `;return nil` 映射到 `PARSE_LEGACY_LEADING_SEMICOLON`，将 `a < b < c` 映射到 `PARSE_NONASSOCIATIVE_CHAIN`，将 `x as T as U` 映射到 `PARSE_NONASSOCIATIVE_CHAIN`， `class A for C extends B {}` 至 `PARSE_BAD_HEADER_ORDER`，以及 `fun f(key x: T, y: T) -> Nil {}` 至 `PARSE_BAD_PARAMETER_ORDER`。


## 语法覆盖向量

IRIS-V1-GRAMMAR-C057：以下向量是规范的可追溯性向量，具有具体的源输入和预期的解析或诊断观察结果。

IRIS-V1-GRAMMAR-C058：v1.1 勘误语法允许 `@decorator(arguments)` 位于 Class、Module、Contract、Method 和 property 声明之前；其含义仅由 IRIS-V1-META-C085 到 IRIS-V1-META-C094 拥有。Module 声明可使用 `module fun`，作为 `class fun` 的互斥替代；其安装表面由 IRIS-V1-CONTROL-C074 定义。存储属性简写是 `property name: Type`，带可选 initializer 和可选访问器可见性块，而 `property fun` 仍是显式访问器形式；其存储语义由 IRIS-V1-RUNTIME-C065 和 IRIS-V1-RUNTIME-C161 拥有。`typeof(expression)` 是 Type-expression 产生式，其语义由 IRIS-V1-TYPES-C093 拥有。

IRIS-V1-GRAMMAR-C059：v1.2 勘误语法添加 `shared_decl ::= "shared" ("let" | "mut") shared_name type_annotation? "=" expression` 并将其纳入 `declaration`。这提供了 IRIS-V1-CONTROL-C009 和 IRIS-V1-RUNTIME-C073 已预设的类变量声明形式，并且它镜像 `global_decl`，使两个存储族共享一种形状。自 v1.0 起由 IRIS-V1-GRAMMAR-C013 保留但没有产生式的 `shared` 关键字现在恰好在此使用。`shared_decl` MUST 出现在 Class 或 Module 声明体中；`let` 声明不可变单元，`mut` 声明可赋值单元。已声明单元的语义、层次结构锚定和查找规则仍由 IRIS-V1-RUNTIME-C073 至 IRIS-V1-RUNTIME-C075 拥有。

IRIS-V1-GRAMMAR-C060：v1.3 勘误允许 `if_expression` 出现在表达式位置，并依照该产生式定义 `if_statement`。其值、分支作用域、缺少 `else` 时的结果和可达分支结果类型仍完全由 IRIS-V1-CONTROL-C041 规定。在 `match_arm` 中，`match_guard` 在解析 guard `expression` 之前消耗其开头的 `if`；因此 `if_expression` 只在需要主表达式的位置开始。`if_expression` 后的 `block_body` 以 `{` 开始，而 Hash 字面量使用 IRIS-V1-GRAMMAR-C021 要求的不同 `%{` 开场。

IRIS-V1-GRAMMAR-C061：v1.4 勘误将 `class_mixin` 和 `module_mixin` 替换为 `mixin_entry_list`；每个 `mixin_entry` MAY 携带 `private`，以在该静态组合边缘记录 private authorization。该授权、其作用域和撤销仍完全由 IRIS-V1-RUNTIME-C050、IRIS-V1-META-C056 和 IRIS-V1-META-C060 拥有。原始 current-receiver `@x` 访问独立于该选项，仍完全由 IRIS-V1-RUNTIME-C051 和 IRIS-V1-META-C058 拥有。

IRIS-V1-GRAMMAR-C062：v1.16 勘误将 `method_decl` 中的 `block_body` 改为可选，因此方法声明 MAY 只给出签名而不带方法体。这补上了 IRIS-V1-TYPES-C042 在允许 Contract 体声明实例、类对象、属性与泛型方法要求时已经预设、但此前没有任何产生式能够表达的要求语法。无方法体的 `method_decl` 是一条**要求**：它声明义务而不提供实现。它仅在 `contract_decl` 内部是良构的；出现在 `class_decl` 或 `module_decl` 体内的无体 `method_decl` 会被拒绝。相反的情形仍由 IRIS-V1-TYPES-C042 拥有，该条款禁止 Contract 内出现方法体：该方法体现在能够**解析**，并按 IRIS-V1-TYPES-V258 所观察的那样，作为静态诊断 `CONTRACT_METHOD_BODY_FORBIDDEN` 被拒绝，而不再表现为解析错误。本勘误不新增关键字、不新增词法单元、不新增声明形式，也不改变任何书写了方法体的方法的含义。

IRIS-V1-GRAMMAR-C063：v1.17 勘误向 `primary_expr` 增加 `closed_generic_name ::= qualified_type_name generic_args`，因此**闭合**泛型构造 MAY 出现在表达式位置。这补上了 IRIS-V1-TYPES-C061 在要求普通构造必须命名闭合泛型类型时已经预设的语法，也是 `Box<String>.new()`、`Pair<String, Integer>.new()` 与 `Box<String>.type` 所需要的语法。IRIS-V1-GRAMMAR-C020 并未被削弱：名字之后的 `<` 仅在该名字是 `type_name`、且尖括号对以一个其后跟随 `postfix_part` 的 `>` 闭合时，才开始泛型实参。其余每一个 `<` 都保持其运算符分词，因此 `a < b` 仍是比较，`a >> b` 仍是右移。当两种读法都可良构时，以**运算符**读法为准，从而保持本勘误之前所有可解析程序的含义不变。不带实参的裸泛型名字在 IRIS-V1-TYPES-C061 下仍是定义元数据，此处 NOT 接纳。

IRIS-V1-GRAMMAR-C064：v1.18 勘误向 `property_decl` 增加 `"shared"? ("class" | "module")?`，因此属性 MAY 在类级或模块级声明，并 MAY 标记为 `shared`。这补上了 IRIS-V1-TYPES-C064 在区分「普通泛型类级存储（按闭合构造各自独立）」与「`shared class property`（属于未应用的泛型定义）」时已经预设的语法。其存储语义仍由 IRIS-V1-TYPES-C064 与 IRIS-V1-RUNTIME-C065 拥有；本条款仅补语法。`shared` 是 IRIS-V1-GRAMMAR-C059 已为 `shared_decl` 保留的关键字，因此不新增关键字；未标记的 `property` 保持其原有的实例级含义，完全不变。

IRIS-V1-GRAMMAR-C065：v1.19 勘误向 `primary_expr` 增加 `reified_type_expr ::= "(" type_expr ")" &"." "type"`，因此带括号的类型表达式 MAY 被具体化为值。这补上了 IRIS-V1-TYPES-C016 与 IRIS-V1-TYPES-C076 在要求「可驻留、带标识的 Type 对象」时已经预设的语法，也是 `(String | Nil).type`、`(String & Object).type` 与 `(Object?).type` 所需要的语法。其中 `&"." "type"` 是**前瞻**而非被消费的输入：仅当右括号之后紧跟 `.type` 时，才取类型读法。其余一切位置上，`|`、`&`、`?` 均保持 IRIS-V1-GRAMMAR-C020 与表达式优先级表赋予它们的运算符分词，因此 `(a | b)` 仍是按位或，`(a & b) . 其他选择子` 仍是按位与。当两种读法都可良构时，以**运算符**读法为准，从而保持本勘误之前所有可解析程序的含义不变。本条款不新增关键字、不新增词法单元：`type` 仍是普通选择子。

IRIS-V1-GRAMMAR-C066：v1.20 勘误向 `call_suffix` 增加 `call_type_arguments`，因此调用 MAY 显式给出方法类型实参，如 `choose<String, Integer>(value)`。这补上了 IRIS-V1-TYPES-C071 在要求「使用全元数尖括号并以 `_` 占位」时已经预设的语法，也是 IRIS-V1-TYPES-C060 将缺失的末尾实参报为元数错误（而非推断默认值）所必需的语法。`call_type_argument` 仅在此处接纳 `_`：IRIS-V1-TYPES-C072 在一切持久类型位置上仍禁止它，且它是类型实参占位符，而非绑定、也非新的类型变量。IRIS-V1-GRAMMAR-C020 并未被削弱：名字之后的 `<` 仅在尖括号对以一个其后**紧跟** `(` 的 `>` 闭合时，才开始调用类型实参。其余每一个 `<` 都保持其运算符分词，因此 `a < b` 仍是比较。当两种读法都可良构时，以**运算符**读法为准，从而保持本勘误之前所有可解析程序的含义不变。本条款不新增关键字、也不新增词法单元。

IRIS-V1-GRAMMAR-C067：v1.22 勘误允许闭合泛型名作为完整表达式，因此 `Box<String>` 可用作值，而不仅仅是 `postfix_part` 的接收者。这补充了 D-456 已经预设的语法——它使驻留的 Type 对象区别于 Class 对象，IRIS-V1-TYPES-V257 正是通过比较 `Box<String>.type` 与 `Box<String>` 来观察这一点。本条就地取代 IRIS-V1-GRAMMAR-C063 中的 `postfix_part` 要求；C063 陈述的其余条件均不变。IRIS-V1-GRAMMAR-C020 不被削弱。仅当名称是 `type_name` 且括号对以 `>` 闭合时才采用泛型读法，并且闭合的 `>` 之后现在必须跟随 `postfix_part`，或跟随一个无法延续表达式的 token：语句终结符、`,`、`]`、`)`、`}` 或输入结束。当两种读法本都良构时，仍以运算符读法优先，因此 `a < b` 仍是比较，`a >> b` 仍是右移。不带参数的裸泛型名依据 IRIS-V1-TYPES-C061 仍是定义元数据，此处不予接纳。

IRIS-V1-GRAMMAR-C068：v1.23 勘误新增 `package_qualified_name ::= package_name "::" qualified_type_name` 与 `package_name ::= ordinary_name ("." ordinary_name)*`，并允许其作为 `import_decl` 的路径。这补充了 IRIS-V1-META-C013 已经预设的语法——该条将支持的形式写作 `import pkg::Module`，而 IRIS-V1-META-C003 使每个可发布的 `pkg` 都是形如 `org.dep` 的反向域名式 `package_id`。点分名称仅允许出现在导入路径中 `::` 之前的包段位置：`ordinary_name` 不变，`qualified_type_name` 不变，其他位置的 `.` 保持成员访问含义，因此 `a.b` 仍是成员读取。不含 `::` 的导入路径继续命名当前包中的 Module。IRIS-V1-META-C013 对通配符导入与运行时字符串导入的排除不变，`*` 在导入路径中的任何位置仍不被接纳。

IRIS-V1-GRAMMAR-C069：v1.24 勘误为导入形式增加可选的 `"override"` 标记，因此 `import_decl ::= "override"? "import" import_path import_alias? | "override"? "from" import_path "import" import_spec_list`。这补充了 IRIS-V1-META-C049 已经预设的位置——该条要求使用「语言接受的 `override` 导入标记」，而 D-230 将其留待后续标准化。依据 IRIS-V1-META-C049 与 D-230，该标记授权此导入所贡献的兼容替换；它绝不授权签名或静态契约的不兼容，且 IRIS-V1-META-C049 对未标记替换的拒绝保持不变。标记置于关键字之前而非 `import_spec` 内部，因为 D-230 授权的是**该被导入扩展**所贡献的替换，而非逐名授权。`override` 依据 IRIS-V1-GRAMMAR-C013 已是保留字，且已出现在 `method_decl` 中，因此未新增 token，同模块 open 的 override 标记仍如 D-230 要求保持独立。

IRIS-V1-GRAMMAR-C070：v1.26 勘误将装饰器应用路径放宽为 `decorator ::= "@" qualified_type_name "(" call_argument_list? ")"`，因此在另一 Module 中声明的装饰器以 `@D::Stamp()` 应用。这补充了 IRIS-V1-META-C122 所需的引用形式，也是 IRIS-V1-META-C013 的导入形式在跨 Module 使用时已经预设的。未新增关键字，IRIS-V1-GRAMMAR-C013 的保留字清单不变：依据 IRIS-V1-META-C122，装饰器是普通 Class，因此不需要声明产生式，IRIS-V1-CONTROL-C014 的三种普通可调用运行时种类保持不变。

| 向量 ID | 类别 | 适用性 | 源代码/输入 | 预期可观察结果 | 决策 |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-GRAMMAR-V003`| 正向 |需要解释器；需要 JIT；原生不适用 | Iris 来源：`2 ** 3 ** 2; -2 ** 2; 2 ** -3`。 | 解析形状为 `2 ** (3 ** 2)`、`-(2 ** 2)` 和 `2 ** (-3)`；求幂是右结合的，并且比一元否定结合得更紧密。 | `D-033` |
| `IRIS-V1-GRAMMAR-V010` | 诊断 | 需要解析器；解释器不适用； JIT 不适用；原生不适用 | 保留关键字库存加上历史字词`alias`、`switch`、`when`、`and`、`or`、`not`、`undef`。 | 确切地说，D-509 关键字被保留；历史非关键字是普通标识符或仅被上下文特定语法拒绝。 | `D-509` |
| `IRIS-V1-GRAMMAR-V011` | 正向 | 需要解析器；解释器可选； JIT 可选；原生不适用 | `a ..= b`、`a ..< b`、`view..member`、`%{k: v}`、`${x}` 内插、`Box<Array<String>>`、`/x/`、`m"x"`、 `br"x"`，`mbr"x"`。|上下文最长匹配标记化与每个列出的冲突的特定于子句的解释相匹配。 | `D-510` |
| `IRIS-V1-GRAMMAR-V013` | 诊断 | 需要解析器；解释器不适用； JIT 不适用；原生不适用 | `outer: while ready { break outer: 1 }`、`break outer 1`、`label: return 1`、`continue outer`。 | 带标签的循环形式和 `break outer: expr` 准确解析；非循环标签和缺失的冒号形式将被拒绝。 | `D-440` |

## 可追踪性说明

IRIS-V1-GRAMMAR-C055：本章合并 D-033 至 D-050、D-183、D-277 至 D-283、D-294 至 D-300 中定义源头部语法的部分，以及 D-336 至 D-386、D-395、D-408、D-417 至 D-424、D-461、D-463 和 D-505 至 D-510。后续修订的决策措辞优先于旧历史语法证据。

IRIS-V1-GRAMMAR-C056：涵盖的决定 ID 为 `D-033`、`D-034`、`D-035`、`D-036`、`D-037`、`D-038`、 `D-039`，`D-040`，`D-041`，`D-042`，`D-043`，`D-044`，`D-045`，`D-046`， `D-047`、`D-048`、`D-049`、`D-050`、`D-183`、`D-277`、`D-278`、`D-279`、 `D-280`、`D-281`、`D-282`、`D-283`、`D-294`、`D-295`、`D-296`、`D-297`、 `D-298`、`D-299`、`D-300`、`D-336`、`D-337`、`D-338`、`D-339`、`D-340`、 `D-341`、`D-342`、`D-343`、`D-344`、`D-345`、`D-346`、`D-347`、`D-348`、 `D-349`，`D-350`，`D-351`，`D-352`，`D-354`，`D-355`，`D-356`，`D-357`， `D-358`、`D-359`、`D-360`、`D-361`、`D-362`、`D-363`、`D-364`、`D-365`、 `D-366`、`D-367`、`D-368`、`D-369`、`D-370`、`D-371`、`D-372`、`D-373`、 `D-374`、`D-375`、`D-376`、`D-377`、`D-378`、`D-379`、`D-380`、`D-381`、 `D-382`、`D-383`、`D-384`、`D-385`、`D-386`、`D-388`、`D-395`、`D-408`、 `D-417`，`D-418`，`D-419`，`D-420`，`D-421`，`D-422`，`D-423`，`D-424`， `D-461`、`D-463`、`D-505`、`D-506`、`D-507`、`D-508`、`D-509` 和 `D-510`。

IRIS-V1-GRAMMAR-N001：历史注释：Notepad++ 语法高亮器列出旧单词和旋转运算符，例如 `interface`、`groan`、`order`、`serve`、`ignore`、`alias`、 `retry`、`redo`、`goto`、`static`、`<<<` 和 `>>>`。本章仅将它们记录为迁移证据，并不为 Iris v1 保留它们。

## 具体语法覆盖向量

每行都命名具体的源文本或字节以及其引用的决策所需的确切解析器、词法分析器或运行时观察。

| 向量 ID | 类别 | 适用性 | 源代码/输入 | 预期可观察结果 | 决策 |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-GRAMMAR-V144`| 正向 |需要解释器；需要 JIT；本机不适用 | Iris 来源：`[0b1010, 0o755, 00755, 0xFF]`。 | 值为 `[10, 493, 755, 255]`，每个值都有类型 `Integer`；不带前缀的 `00755` 是十进制。 | `D-039`, `D-041` |
| `IRIS-V1-GRAMMAR-V145` | 诊断 | 解释器不适用； JIT 不适用；本机不适用 | 源案例：`1_000`、`0xFF_FF`、`1.234_567`、`1e1_000`、`1.0e+1_024`；畸形 `_1`、`1_`、`1__0`、`0x_FF`、`1_.0`、`1._0`、`1e_3`、`1e+_3`、 `1_f32`。 | 五个有效的字面量标记为数字；每个畸形的情况都会在词法阶段发出 `LEX_BAD_NUMERIC_SEPARATOR`。 | `D-040` |
| `IRIS-V1-GRAMMAR-V146` | 正向 | 需要解释器；需要 JIT；本机不适用 | Iris 来源：`00755`。|字面量的计算结果为 `Integer(755)`，而不是八进制 `493`。 | `D-041` |
| `IRIS-V1-GRAMMAR-V147` | 诊断 | 解释器不适用； JIT 不适用；本机不适用 | 源案例：`0B1010`、`0O755`、`0Xff`、`1E3`、`1.0F32`、`1.0F64`。 | 前四个解析为 `Integer(10)`、`Integer(493)`、`Integer(255)` 和 `Float64(1000.0)`；每个大写后缀都会发出 `LEX_BAD_FLOAT_SUFFIX`。 | `D-042` |
| `IRIS-V1-GRAMMAR-V148` | 诊断 | 解释器不适用； JIT 不适用；本机不适用 | 源案例：`0b102`、`0o89`、`0xFG`。 | 每个连续候选者都是一个无效的数字标记，并发出 `LEX_INVALID_RADIX_DIGIT`；它没有被分割成数字加标识符。 | `D-043`|
|`IRIS-V1-GRAMMAR-V149` | 诊断 | 解释器不适用； JIT 不适用；本机不适用 | 源案例：`0x`、`0X`、`0b`、`0B`、`0o`、`0O`、`0x_FF`、`0b_1010`、 `0o_755`。 | 每种情况都会将 `LEX_EMPTY_RADIX_PREFIX` 作为一个无效的数字标记发出，而不进行零加标识符恢复。 | `D-044` |
| `IRIS-V1-GRAMMAR-V150` | 诊断 | 需要解释器；需要 JIT；本机不适用 | Iris 源值 `.5` 和 `1.`；源案例 `.`、`._5`、`1._0`、`1_.`。 | `.5` 为 `Float64(0.5)`，`1.` 为 `Float64(1.0)`； `.` 为 `DOT`；其余三个案例发出 `LEX_BAD_NUMERIC_SEPARATOR`。 | `D-045` |
| `IRIS-V1-GRAMMAR-V151` | 诊断 | 需要解释器；需要 JIT；本机不适用|Iris 源值 `1e3`、`2E-4`、`1e+3f32`；源案例 `1e`、`1e+`、`1e_3`。 | 值为 `Float64(1000.0)`、`Float64(0.0002)` 和 `Float32(1000.0)`；畸形的指数在执行前发出 `LEX_BAD_NUMERIC_SEPARATOR` 或词法指数诊断。 | `D-046` |
| `IRIS-V1-GRAMMAR-V152` | 正向 | 需要解释器；需要 JIT；本机不适用 | Iris 来源：`[0x1.fp3, 0x1p0, 0x1.p0, 0x.8p0, 0x1e3]`。 | 值为 `[15.5, 1.0, 1.0, 0.5, 483]`；前四个浮点值的类型为 `Float64`，`0x1e3` 的类型为 `Integer`。 | `D-047`, `D-048` |
| `IRIS-V1-GRAMMAR-V153` | 诊断 | 解释器不适用； JIT 不适用；本机不适用 | 来源案例：`0x1.`。 | 候选者发出词汇畸形的十六进制浮点诊断，并且不会标记为 `Integer(1)` 后跟 `DOT`。|`D-048` |
| `IRIS-V1-GRAMMAR-V154` | 差异 | 需要解释器；需要 JIT；本机不适用 | 源字面量是 `Float32` 和 `Float64` 的十进制和十六进制中点固定装置，在不同的主机区域设置和主机舍入模式下运行。 | 解释器和 JIT 为每个赛程报告相同的 `float32_bits` 和 `float64_bits`，每个都等于所需的平局结果。 | `D-049` |
| `IRIS-V1-GRAMMAR-V155` | 诊断 | 需要解释器；需要 JIT；本机不适用 | 有限非零十进制和十六进制装置，在两种浮点宽度下舍入为 `+infinity`、`-infinity`、次正规、`+0.0` 和 `-0.0`。 | 溢出和零舍入装置生成其签名的 IEEE 值和一个默认启用的精度警告；次正规装置会在没有警告的情况下产生精确的非零位。 | `D-050` |
| `IRIS-V1-GRAMMAR-V161`| 诊断 |解释器不适用； JIT 不适用；本机不适用 | 源对：`class A extends Object for C mixin M where T: Object meta deny shape {}` 和重新排序的 Class、Module 和 Contract 标头。 | 每个规范标头都会解析；每个重新排序或重复的标头都会发出 `PARSE_BAD_HEADER_ORDER`。 | `D-280`, `D-281` |
| `IRIS-V1-GRAMMAR-V163` | 正向 | 解释器不适用； JIT 不适用；本机不适用 | 来源：`class Pair<T, U> where T: A & B, U: C {}`。 | 解析器产生两个约束分配，`T: A & B`和`U: C`；逗号不是交集运算符。 | `D-282` |
| `IRIS-V1-GRAMMAR-V165` | 诊断 | 解释器不适用； JIT 不适用；本机不适用 | 有效来源：`class A meta deny shape {}`；无效来源：`class A { if true { meta deny shape } }`。|头部形式解析；主体形式发出 `PARSE_BAD_HEADER_ORDER` 并且不创建声明候选者。 | `D-290`, `D-298` |
| `IRIS-V1-GRAMMAR-V167` | 诊断 | 解释器不适用； JIT 不适用；本机不适用 | 源案例：`;return nil`、`;` 和 `x;;y`。 | `;return nil` 发出 `PARSE_LEGACY_LEADING_SEMICOLON`；独立和重复的分号发出 `PARSE_EMPTY_STATEMENT`。 | `D-362`, `D-364` |
| `IRIS-V1-GRAMMAR-V168` | 正向 | 需要解释器；需要 JIT；本机不适用 | Iris 源使用 `let a = 1\nlet b = (1\n+ 2)\nlet c = 1 \\\n+ 2\nlet d = 3;`。 | `1` 后的换行符终止其语句，括号内的换行符和转义换行符则不终止，并且接受一个尾随分号。 | `D-363`|
|`IRIS-V1-GRAMMAR-V170` | 诊断 | 解释器不适用； JIT 不适用；本机不适用 | 源包括 `// note`、`/// docs` 和嵌套 `/* outer /* inner */ outer */`；第二个装置是 `/* outer /* inner */`。 | 第一个装置在没有注释的情况下进行标记；第二个发出 `LEX_UNTERMINATED_COMMENT`。 | `D-365` |
| `IRIS-V1-GRAMMAR-V171` | 诊断 | 解释器不适用； JIT 不适用；本机不适用 | 一台 UTF-8 源夹具以 `#! /usr/bin/env iris\nlet x = 1` 开头；另一个包含 `let x = 1\n#! late`。 | 第一个装置接受第一行 shebang 作为注释；第二个发出 `LEX_SHEBANG_NOT_FIRST`。 | `D-366` |
| `IRIS-V1-GRAMMAR-V172` | 诊断 | 解释器不适用； JIT 不适用；本机不适用|字节夹具：UTF-8 BOM 加 `let cafe = "ok"`；畸形 UTF-8 `0xC3 0x28`；和 UTF-16LE BOM `0xFF 0xFE`。 | 有效的fixture解析；畸形的 UTF-8 和 UTF-16 均会发出 `LEX_INVALID_UTF8`，但没有替换字符令牌。 | `D-367` |
| `IRIS-V1-GRAMMAR-V173` | 正向 | 需要解释器；需要 JIT；本机不适用 | Iris 来源：`["a\\n${1 + 1}", 'a\\n${x}', r#"a\\n${x}"#, """\n  a\n  """]`。 | 值为 `["a\n2", "a\n${x}", "a\\n${x}", "a"]`，所有类型为 `String`；转义、单行、原始和多行族按照指定是不同的。 | `D-368` |
| `IRIS-V1-GRAMMAR-V174` | 诊断 | 解释器不适用； JIT 不适用；本机不适用 | 来源使用`"\\\\\\\"\\'\\n\\r\\t\\0\\b\\f\\v\\xFF\\u{1F600}"`；畸形案例使用 `"\\q"`、`"\\u{D800}"` 和 `"\\u{110000}"`。 | 有效字面量具有指定的标量序列；每个畸形的案例都会发出 `LEX_BAD_ESCAPE`。|`D-369` |
| `IRIS-V1-GRAMMAR-V175` | 诊断 | 需要解释器；需要 JIT；本机不适用 | 有效的三重源是`"""\n  alpha\n  beta\n  """`；畸形的固定装置有一个非空行，没有结束缩进。 | 有效值为`"alpha\nbeta"`；畸形的装置会发出 `LEX_BAD_MULTILINE_INDENT` 信号。 | `D-379` |
| `IRIS-V1-GRAMMAR-V176` | 诊断 | 解释器不适用； JIT 不适用；本机不适用 | 三倍字面量的结束缩进为两个空格，内容行的前缀为一个制表符加两个空格。 | 该行与确切的空格前缀不匹配，并发出 `LEX_BAD_MULTILINE_INDENT`；制表符未标准化为空格。 | `D-380` |
| `IRIS-V1-GRAMMAR-V177`| 诊断 |需要解释器；需要 JIT；本机不适用 | 源字面量：`r"a\\n${x}"`、`r##"a\\n${x}"##` 以及具有 256 个栅栏字符的装置。 | 匹配零和非零栅栏保留字面量反斜杠和 `${x}`； 256 栅栏灯具发出 `LEX_BAD_RAW_FENCE`。 | `D-381` |
| `IRIS-V1-GRAMMAR-V178` | 正向 | 需要解释器；需要 JIT；本机不适用 | 原始三重来源：`r"""\n  ${x}\\n\n  """`。 | 结果是`"${x}\\n"`；当插值和转义仍然是原始文本时，会发生严格的缩进剥离。 | `D-382` |
| `IRIS-V1-GRAMMAR-V179` | 正向 | 需要解释器；需要 JIT；本机不适用 | Iris 来源：`"a" 'b' r"c" """d"""`。|表达式产生 `String("abcd")`；字面量段之间没有出现语句终止符。 | `D-386` |
| `IRIS-V1-GRAMMAR-V189` | 诊断 | 解释器不适用； JIT 不适用；本机不适用 | 源值：`"\xFF"`；来源案例：`"\u{D800}"`。 | 第一个值是`String("U+00FF")`，而不是`Bytes`；代理逃逸发出 `LEX_BAD_ESCAPE`。 | `D-370` |
| `IRIS-V1-GRAMMAR-V190` | 诊断 | 解释器不适用； JIT 不适用；本机不适用 | 源值：`"${1 + 2}"`；来源案例：`"${value:04d}"`。 | 第一个插值接受完整表达式并产生 `String("3")`；格式迷你语言形式在解析阶段被拒绝。 | `D-383`|
|`IRIS-V1-GRAMMAR-V181` | 诊断 | 解释器不适用； JIT 不适用；本机不适用 | 源案例：`/a+/i`、`r/a\\/b/u`、`/a/ii` 和 `/(a)\\1/`。 | 前两个标记为 Regex 字面量及其列出的标志；重复标志和不受支持的反向引用语法会发出 Regex 子句所需的 Regex 词法或验证诊断。 | `D-505`, `D-506` |
| `IRIS-V1-GRAMMAR-V183` | 诊断 | 解释器不适用； JIT 不适用；本机不适用 | 源声明标识符 `alias`、`switch`、`when`、`and`、`or`、`not`，并尝试 `let class = 1`。 | 历史词是普通的标识符； `class` 是保留关键字标记，绑定在解析阶段被拒绝。 | `D-509` |
| `IRIS-V1-GRAMMAR-V184` | 诊断 | 解释器不适用； JIT 不适用；本机不适用|源案例：`a ..= b`、`a ..< b`、`view..member`、`a != b`、`property fun ready?=(v: Bool) {}`、`Box<Array<String>>`、`%{k: v}`、`${x}`、 `m"x"`、`br"x"`、`mbr"x"` 和 `a / b`。 | 每个案例都会收到 C017-C023 规定的上下文特定标记化； `${x}` 在字面量之外发出 `LEX_INTERPOLATION_OUTSIDE_LITERAL`，并且 `a / b` 使用除法而不是 Regex 开头。 | `D-510` |
| `IRIS-V1-GRAMMAR-V187` | 诊断 | 解释器不适用； JIT 不适用；本机不适用 | 源声明 `fun ready?() { true }`、`fun save!() { nil }`，然后尝试 `fun ready?!() { nil }` 和 `let done? = true`。 | 前两个选择器使用来自 `ready` 和 `save` 的不同身份进行解析；畸形的选择器和带后缀的本地名称在解析阶段会被拒绝。 | `D-342` |
| `IRIS-V1-GRAMMAR-V188` | 诊断 | 解释器不适用； JIT 不适用；本机不适用 | 源分配使用 `+=`、`-=`、`*=`、`/=`、`**=`、`&=`、`&#124;=`、`^=`、 `<<=`、`>>=` 和无效的 `%=`，具有单独的 `&&=` 和 `&#124;&#124;=` 控制夹具。 | 每个列出的复合运算符都会解析； `%= ` 被拒绝，因为 `%` 不是 Iris v1 运算符，而逻辑赋值仍保留保留的核心语法。|`D-348` |
| `IRIS-V1-GRAMMAR-V191` | 诊断 | 需要解释器；需要 JIT；本机不适用 | 源文本装置使用 `<BS><LF>` 作为反斜杠字节，后跟 LF：`let sum = 1 + <BS><LF>2`、`let bad = 1 + <BS><SP><LF>2` 和 `let also_bad = 1 + <BS><SP>// note<LF>2`。 | 在词法分析器删除 `sum` 后，第一个源将 `Integer(3)` 评估为 `<BS><LF>`。后两者在解析或执行之前都会发出 `LEX_BAD_CONTINUATION`。 | `D-388` |
| `IRIS-V1-GRAMMAR-V192` | 正向 | 解释器不适用； JIT 不适用；本机不适用 | 来源：`contract Child extends ParentA, ParentB {}`。 | 解析器接受带有 `extends` 子句的 Contract 声明，该子句包含有序父 Type 列表 `ParentA`、`ParentB`；它不会将此形式解析为 Module 组合。 | `D-277` |
| `IRIS-V1-GRAMMAR-V193`| 正向 |解释器不适用； JIT 不适用；本机不适用 | 源对：`class ImplicitRoot {}` 和 `class ExplicitRoot extends Object {}`。 | 两个声明都解析为 Class 标头；第一个缺少 `extends` 子句，第二个具有显式 `Object`，保留类型层同根规则所需的两种不同源形式。 | `D-283`|
