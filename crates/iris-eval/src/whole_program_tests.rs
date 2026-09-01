//! Differential tests over WHOLE PROGRAMS rather than single constructs.
//!
//! The conformance corpus is a microscope: its vectors have a median length
//! near a hundred characters, each isolating one rule. That is what a
//! specification needs, but it leaves a class of defect invisible - one that
//! appears only when features INTERACT. Two such defects reached a fully
//! covered backend: a stored property and its `@name` ivar addressed different
//! slots, and a property had no setter at all. Every corpus vector passed
//! throughout, because none both wrote a property and read it back from inside
//! a method.
//!
//! These programs are therefore written the way a user writes them - classes
//! with state and behaviour, modules that orchestrate them, loops and
//! recursion over real data - and are compared value-for-value against the
//! reference. A disagreement here is a defect the corpus cannot see.

use crate::backend::{Agreement, Bytecode, Interpreter, Observation, compare_backends};

/// Asserts both backends run `source` and answer `expected`.
///
/// A DECLINED program fails this too: a program the machine cannot run is not
/// a program a user can run, which is exactly what these tests measure.
#[track_caller]
fn agrees(source: &str, expected: &str) {
    let agreement = compare_backends(source, &[&Interpreter, &Bytecode]);
    let Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must run this program: {agreement:?}\n{source}")
    };
    assert_eq!(
        observation,
        &Observation::Value(expected.to_owned()),
        "{source}"
    );
}

/// Object state written, mutated, read back, and guarded by a raise.
#[test]
fn a_stateful_class_holds_its_own_values() {
    agrees(
        r#"
class Account {
  property owner: String = ""
  property balance: Integer = 0

  public fun deposit(amount: Integer) -> Integer {
    if amount <= 0 { raise :InvalidAmount }
    @balance = @balance + amount
    @balance
  }

  public fun withdraw(amount: Integer) -> Integer {
    if amount > @balance { raise :Insufficient }
    @balance = @balance - amount
    @balance
  }

  public fun to_string() -> String {
    @owner + ": " + @balance.to_string()
  }
}

module Bank {
  public fun run() -> Object {
    let a = Account.new()
    a.owner = "alice"
    a.deposit(100)
    a.withdraw(30)
    let refused = try { a.withdraw(1000) } catch e { e }
    let rejected = try { a.deposit(0) } catch e { e }
    [a.to_string(), a.balance, refused, rejected]
  }
}

Bank.run()
"#,
        "[\"alice: 70\", 70, :Insufficient, :InvalidAmount]",
    );
}

/// An object GRAPH built in a loop and walked in another.
#[test]
fn an_object_graph_is_built_and_traversed() {
    agrees(
        r#"
class Node {
  property value: Integer = 0
  property next: Object = nil
}

module ListOps {
  public fun build(xs: Array) -> Object {
    mut head = nil
    for x in xs {
      let n = Node.new()
      n.value = x
      n.next = head
      head = n
    }
    head
  }

  public fun sum(node: Object) -> Integer {
    mut total = 0
    mut cur = node
    while cur != nil {
      total = total + cur.value
      cur = cur.next
    }
    total
  }

  public fun length(node: Object) -> Integer {
    mut count = 0
    mut cur = node
    while cur != nil {
      count = count + 1
      cur = cur.next
    }
    count
  }
}

module Main {
  public fun run() -> Object {
    let list = ListOps.build([1, 2, 3, 4, 5])
    [ListOps.sum(list), ListOps.length(list), list.value]
  }
}

Main.run()
"#,
        "[15, 5, 5]",
    );
}

/// RECURSION, inheritance, an override, and dispatch through the base.
#[test]
fn recursion_and_inheritance_compose() {
    agrees(
        r#"
module Math {
  public fun fib(n: Integer) -> Integer {
    if n < 2 { return n }
    fib(n - 1) + fib(n - 2)
  }

  public fun fact(n: Integer) -> Integer {
    if n <= 1 { 1 } else { n * fact(n - 1) }
  }
}

class Shape {
  public fun area() -> Integer { 0 }
  public fun describe() -> String { "shape:" + area().to_string() }
}

class Square extends Shape {
  property side: Integer = 0
  public override fun area() -> Integer { @side * @side }
}

class Rect extends Shape {
  property w: Integer = 0
  property h: Integer = 0
  public override fun area() -> Integer { @w * @h }
}

module Main {
  public fun run() -> Object {
    let s = Square.new()
    s.side = 4
    let r = Rect.new()
    r.w = 3
    r.h = 5
    mut total = 0
    for shape in [s, r] { total = total + shape.area() }
    [Math.fib(10), Math.fact(5), s.describe(), r.describe(), total]
  }
}

Main.run()
"#,
        "[55, 120, \"shape:16\", \"shape:15\", 31]",
    );
}

/// A MODULE mixed into a class, reaching the composing object's own state.
#[test]
fn a_mixin_reaches_the_composing_object() {
    agrees(
        r#"
module Greet {
  public fun greeting() -> String { "hello, " + self.name() }
  public fun shout() -> String { self.greeting() + "!" }
}

class Person mixin Greet {
  property name_value: String = ""
  public fun name() -> String { @name_value }
}

module Main {
  public fun run() -> Object {
    let p = Person.new()
    p.name_value = "iris"
    [p.greeting(), p.shout()]
  }
}

Main.run()
"#,
        "[\"hello, iris\", \"hello, iris!\"]",
    );
}

/// Collections driven with CLOSURES, and a hash keyed by an authored value.
#[test]
fn collections_and_closures_work_together() {
    agrees(
        r#"
module Pipeline {
  public fun run() -> Object {
    let xs = [1, 2, 3, 4, 5, 6]
    let doubled = xs.map({ |v| v * 2 })
    let big = xs.select({ |v| v > 3 })
    let total = xs.reduce(0, { |acc, v| acc + v })
    mut counts = %{}
    for x in xs {
      counts[x] = x * x
    }
    [doubled, big, total, counts.length()]
  }
}

Pipeline.run()
"#,
        "[[2, 4, 6, 8, 10, 12], [4, 5, 6], 21, 6]",
    );
}

/// An exception crossing SEVERAL frames, with a `finally` that still runs.
#[test]
fn an_exception_crosses_frames() {
    agrees(
        r#"
mut trace = []

class Parser {
  public fun parse(text: String) -> Object {
    if text == "" { raise :EmptyInput }
    text.length()
  }
}

module Runner {
  public fun attempt(text: String) -> Object {
    let p = Parser.new()
    try {
      trace.append(:start)
      let n = p.parse(text)
      trace.append(:parsed)
      n
    } catch e {
      trace.append(:caught)
      e
    } finally {
      trace.append(:done)
    }
  }

  public fun run() -> Object {
    let good = attempt("abcd")
    let bad = attempt("")
    [good, bad, trace]
  }
}

Runner.run()
"#,
        "[4, :EmptyInput, [:start, :parsed, :done, :start, :caught, :done]]",
    );
}

/// A CONTRACT implemented by two classes, used through the contract's view.
#[test]
fn a_contract_is_satisfied_by_several_classes() {
    agrees(
        r#"
contract Describable { fun describe() -> String }

class Dog for Describable {
  public impl fun describe() -> String { "dog" }
}

class Cat for Describable {
  public impl fun describe() -> String { "cat" }
}

module Main {
  public fun run() -> Object {
    let d = Dog.new()
    let c = Cat.new()
    mut names = []
    for animal in [d, c] { names.append(animal.describe()) }
    [(d as Describable)..describe(), (c as Describable)..describe(), names]
  }
}

Main.run()
"#,
        "[\"dog\", \"cat\", [\"dog\", \"cat\"]]",
    );
}

/// A CLASS-level counter shared across instances, beside per-object state.
#[test]
fn class_state_and_instance_state_stay_separate() {
    agrees(
        r#"
class Counter {
  class property made: Integer = 0
  property id: Integer = 0

  public fun register() -> Integer {
    Counter.made = Counter.made + 1
    @id = Counter.made
    @id
  }
}

module Main {
  public fun run() -> Object {
    let a = Counter.new()
    let b = Counter.new()
    let c = Counter.new()
    a.register()
    b.register()
    c.register()
    [a.id, b.id, c.id, Counter.made]
  }
}

Main.run()
"#,
        "[1, 2, 3, 3]",
    );
}

/// A method REPLACED by a reopen, observed before and after the replacement.
#[test]
fn a_reopen_changes_behaviour_from_its_position() {
    agrees(
        r#"
class Formatter {
  public fun render(value: Integer) -> String { "v=" + value.to_string() }
}

module Main {
  public fun before() -> String { Formatter.new().render(7) }
}

let first = Main.before()

open class Formatter {
  override public fun render(value: Integer) -> String { "[" + value.to_string() + "]" }
}

module After {
  public fun run() -> Object { [first, Formatter.new().render(7)] }
}

After.run()
"#,
        "[\"v=7\", \"[7]\"]",
    );
}

/// Nested closures CAPTURING enclosing state, called later.
#[test]
fn closures_capture_and_outlive_their_frame() {
    agrees(
        r#"
module Factory {
  public fun adder(base: Integer) -> Object {
    { |x| base + x }
  }

  public fun run() -> Object {
    let add10 = adder(10)
    let add100 = adder(100)
    mut total = 0
    for x in [1, 2, 3] { total = total + add10.call(x) }
    [add10.call(5), add100.call(5), total]
  }
}

Factory.run()
"#,
        "[15, 105, 36]",
    );
}

/// A generic class CONSTRUCTED at two argument types, each keeping its own
/// class-level slot while instances stay independent.
#[test]
fn a_generic_class_separates_its_constructions() {
    agrees(
        r#"
class Box<T> {
  class property made: Integer = 0
  property held: Object = nil
}

module Main {
  public fun run() -> Object {
    Box<String>.made = 2
    Box<Integer>.made = 7
    let s = Box<String>.new()
    let i = Box<Integer>.new()
    s.held = "text"
    i.held = 42
    [Box<String>.made, Box<Integer>.made, s.held, i.held]
  }
}

Main.run()
"#,
        "[2, 7, \"text\", 42]",
    );
}

/// Reports how many corpus vectors the two backends AGREE on.
///
/// Compiling a program proves only that the machine accepted it; agreement
/// proves it answered what the language says. The two numbers are different
/// measurements and this one is the load-bearing one.
#[test]
#[ignore = "measurement, run explicitly"]
fn measure_corpus_agreement() {
    let raw = std::fs::read_to_string("/tmp/srcs.tsv").unwrap_or_default();
    let (mut agreed, mut disagreed, mut held, mut total) = (0_usize, 0_usize, 0_usize, 0_usize);
    let mut seen = 0_usize;
    let mut failures: Vec<String> = Vec::new();
    for line in raw.lines() {
        let Some((name, encoded)) = line.split_once('\t') else {
            continue;
        };
        let Ok(bytes) = base64_decode(encoded) else {
            continue;
        };
        let Ok(source) = String::from_utf8(bytes) else {
            continue;
        };
        // Bound the sweep so one non-terminating program cannot hide the
        // result for every other: the index is printed before the run, so a
        // hang names itself.
        let limit: usize = std::env::var("AGREE_LIMIT")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(usize::MAX);
        let skip: usize = std::env::var("AGREE_SKIP")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(0);
        seen += 1;
        if seen <= skip || seen > skip + limit {
            continue;
        }
        total += 1;
        eprintln!("RUN {seen} {name}");
        let guarded = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            compare_backends(&source, &[&Interpreter, &Bytecode])
        }));
        let Ok(outcome) = guarded else {
            disagreed += 1;
            if failures.len() < 400 {
                failures.push(format!("{name}: PANIC"));
            }
            continue;
        };
        match outcome {
            Agreement::Agreed { .. } => agreed += 1,
            Agreement::Disagreed { observations } => {
                disagreed += 1;
                if failures.len() < 400 {
                    failures.push(format!("{name}: {observations:?}"));
                }
            }
            // A HELD vector is named, not just counted: a rise in the held
            // count otherwise hides a failure that used to be visible.
            Agreement::Insufficient { ran, declined } => {
                held += 1;
                failures.push(format!("HOLD {name}: ran={ran:?} declined={declined:?}"));
            }
        }
    }
    println!("AGREE total={total} agreed={agreed} disagreed={disagreed} held={held}");
    for failure in &failures {
        println!("FAIL {failure}");
    }
}

fn base64_decode(text: &str) -> Result<Vec<u8>, ()> {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = Vec::new();
    let mut buffer = 0_u32;
    let mut bits = 0_u32;
    for byte in text.trim().bytes() {
        if byte == b'=' {
            break;
        }
        let Some(index) = TABLE.iter().position(|candidate| *candidate == byte) else {
            return Err(());
        };
        buffer = (buffer << 6) | index as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buffer >> bits) as u8);
        }
    }
    Ok(out)
}
