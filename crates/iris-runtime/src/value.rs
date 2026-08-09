use core::str::FromStr;

use num_bigint::{BigInt, ParseBigIntError, Sign};
use std::cell::RefCell;
use std::rc::Rc;

use crate::{BoundMethod, ClassId, ContractId, Method, ObjectId};

/// An arbitrary-precision Iris integer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegerValue(BigInt);

impl FromStr for IntegerValue {
    type Err = ParseBigIntError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        BigInt::from_str(source).map(Self)
    }
}

impl IntegerValue {
    /// Returns the canonical base-10 text used for external observations.
    pub fn decimal_text(&self) -> String {
        self.0.to_string()
    }

    pub(crate) fn from_bigint(value: BigInt) -> Self {
        Self(value)
    }

    pub(crate) const fn as_bigint(&self) -> &BigInt {
        &self.0
    }

    pub(crate) fn is_zero(&self) -> bool {
        self.0.sign() == Sign::NoSign
    }

    pub(crate) fn is_negative(&self) -> bool {
        self.0.sign() == Sign::Minus
    }

    pub(crate) fn abs(&self) -> Self {
        Self(if self.is_negative() {
            -&self.0
        } else {
            self.0.clone()
        })
    }

    pub(crate) fn to_u32(&self) -> Option<u32> {
        let (sign, digits) = self.0.to_u32_digits();
        match (sign, digits.as_slice()) {
            (Sign::NoSign, []) | (Sign::Plus, []) => Some(0),
            (Sign::Plus, [value]) => Some(*value),
            (Sign::Minus, _) | (Sign::Plus, [_, ..]) | (Sign::NoSign, [..]) => None,
        }
    }

    /// Returns the value as an unsigned 64-bit integer, or `None` when out of range.
    ///
    /// `D-241` composes a Contract-view hash from two `u64` component hashes,
    /// so a public hash must be readable back as one.
    pub fn to_u64(&self) -> Option<u64> {
        let (sign, digits) = self.0.to_u64_digits();
        match (sign, digits.as_slice()) {
            (Sign::NoSign, []) | (Sign::Plus, []) => Some(0),
            (Sign::Plus, [value]) => Some(*value),
            (Sign::Minus, _) | (Sign::Plus, [_, ..]) | (Sign::NoSign, [..]) => None,
        }
    }

    /// Returns the value as a signed 128-bit integer, or `None` when out of
    /// range.
    ///
    /// `IRIS-V1-COLLECTIONS-C038` gives a Range a step that may be NEGATIVE, so
    /// stepping cannot read its operands back as unsigned.
    pub fn to_i128(&self) -> Option<i128> {
        self.decimal_text().parse().ok()
    }

    /// Returns the value as a container index, or `None` when out of range.
    pub fn to_usize(&self) -> Option<usize> {
        self.to_u64().and_then(|value| value.try_into().ok())
    }

    pub(crate) fn to_f32(&self) -> f32 {
        self.0.to_string().parse::<f32>().unwrap_or_else(|_| {
            if self.is_negative() {
                f32::NEG_INFINITY
            } else {
                f32::INFINITY
            }
        })
    }

    pub(crate) fn to_f64(&self) -> f64 {
        self.0.to_string().parse::<f64>().unwrap_or_else(|_| {
            if self.is_negative() {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            }
        })
    }
}

impl From<u8> for IntegerValue {
    fn from(value: u8) -> Self {
        Self(BigInt::from(value))
    }
}

impl From<i8> for IntegerValue {
    fn from(value: i8) -> Self {
        Self(BigInt::from(value))
    }
}

impl From<u64> for IntegerValue {
    fn from(value: u64) -> Self {
        Self(BigInt::from(value))
    }
}

/// A runtime value independent of the heap's storage representation.
/// A shared, mutable Array body with the `IRIS-V1-COLLECTIONS-C026` version.
///
/// `C003` makes `Array<T>` identity-bearing, so this is a HANDLE: cloning it
/// shares one element sequence rather than copying it. `C026` additionally
/// requires that every length-changing or element-replacing operation increment
/// a content version so an active iterator can detect the change and raise
/// `ConcurrentModificationError` on its next advance.
#[derive(Clone)]
pub struct ArrayRef(Rc<RefCell<ArrayBody>>);

/// Renders as the element sequence.
///
/// The shared cell and the `C026` version are representation, not observable
/// content, and `Value` is compared through its `Debug` rendering in places, so
/// showing them would make an Array's rendering depend on its mutation history.
impl core::fmt::Debug for ArrayRef {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.0.borrow().elements, formatter)
    }
}

/// The elements and content version behind an [`ArrayRef`].
#[derive(Debug)]
pub struct ArrayBody {
    elements: Vec<Value>,
    version: u64,
}

impl ArrayRef {
    /// Creates a new Array holding `elements`, at content version zero.
    #[must_use]
    pub fn new(elements: Vec<Value>) -> Self {
        Self(Rc::new(RefCell::new(ArrayBody {
            elements,
            version: 0,
        })))
    }

    /// Reads the current elements.
    #[must_use]
    pub fn elements(&self) -> Vec<Value> {
        self.0.borrow().elements.clone()
    }

    /// Returns the current element count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.borrow().elements.len()
    }

    /// Returns whether the Array currently holds no elements.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Reads the element at `index`, if any.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<Value> {
        self.0.borrow().elements.get(index).cloned()
    }

    /// Returns the current `C026` content version.
    #[must_use]
    pub fn version(&self) -> u64 {
        self.0.borrow().version
    }

    /// Returns whether two handles denote the SAME Array.
    ///
    /// `C003` makes Array identity-bearing, so this is the `same?` question and
    /// is deliberately distinct from element-sequence equality.
    #[must_use]
    pub fn same(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }

    /// Mutates the elements, incrementing the `C026` content version.
    ///
    /// Every length-changing or element-replacing operation goes through here,
    /// so no such operation can forget to invalidate active iterators.
    pub fn mutate<T>(&self, change: impl FnOnce(&mut Vec<Value>) -> T) -> T {
        let mut body = self.0.borrow_mut();
        let outcome = change(&mut body.elements);
        body.version = body.version.saturating_add(1);
        outcome
    }
}

/// `C026` compares the current element SEQUENCE, not Array identity, so two
/// distinct Arrays holding equal elements in order are equal.
impl PartialEq for ArrayRef {
    fn eq(&self, other: &Self) -> bool {
        if self.same(other) {
            return true;
        }
        self.0.borrow().elements == other.0.borrow().elements
    }
}

impl FromIterator<Value> for ArrayRef {
    fn from_iter<I: IntoIterator<Item = Value>>(elements: I) -> Self {
        Self::new(elements.into_iter().collect())
    }
}

/// A shared, mutable Hash body with the `IRIS-V1-COLLECTIONS-C034` version.
///
/// `C003` makes `Hash<K,V>` identity-bearing, so this is a HANDLE: cloning it
/// shares one entry set rather than copying it.
///
/// `C034` counts only STRUCTURAL change: adding a key, removing a key, clearing
/// and rehashing bump the version, while updating the value for an existing key
/// does not. That distinction is the whole reason this carries its own version
/// type rather than reusing the Array rule, under which every element
/// replacement invalidates active cursors.
#[derive(Clone)]
pub struct HashRef(Rc<RefCell<HashBody>>);

/// The entries and structural version behind a [`HashRef`].
#[derive(Debug)]
pub struct HashBody {
    entries: Vec<(Value, Value)>,
    /// The bucket each entry was placed under at insertion.
    ///
    /// `IRIS-V1-COLLECTIONS-C028` looks an entry up by the key's CURRENT hash,
    /// and `C030` says a container does not track later hash changes, so the
    /// bucket is recorded when the entry is placed and only `rehash()` rebuilds
    /// it. Recomputing it from the key would hide exactly that staleness.
    buckets: Vec<Value>,
    version: u64,
}

impl HashRef {
    /// Creates a new Hash holding `entries`, at structural version zero.
    #[must_use]
    pub fn new(entries: Vec<(Value, Value)>) -> Self {
        let buckets = vec![Value::Nil; entries.len()];
        Self(Rc::new(RefCell::new(HashBody {
            entries,
            buckets,
            version: 0,
        })))
    }

    /// Reads the current entries.
    #[must_use]
    pub fn entries(&self) -> Vec<(Value, Value)> {
        self.0.borrow().entries.clone()
    }

    /// Returns the current entry count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.borrow().entries.len()
    }

    /// Returns whether the Hash currently holds no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Reads the value stored for `key`, if the Hash holds it.
    ///
    /// `C028` dispatches each key's current equality, which the derived value
    /// equality performs for the built-in cases.
    #[must_use]
    pub fn get(&self, key: &Value) -> Option<Value> {
        self.0
            .borrow()
            .entries
            .iter()
            .find(|(held, _)| held == key)
            .map(|(_, value)| value.clone())
    }

    /// Returns whether the Hash holds `key`.
    #[must_use]
    pub fn contains_key(&self, key: &Value) -> bool {
        self.get(key).is_some()
    }

    /// Returns the current `C034` structural version.
    #[must_use]
    pub fn version(&self) -> u64 {
        self.0.borrow().version
    }

    /// Returns whether two handles denote the SAME Hash.
    #[must_use]
    pub fn same(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }

    /// Inserts or updates `key`, answering nothing.
    ///
    /// `C034` makes an INSERT structural and an update of an existing key
    /// non-structural, so the version moves only when a key is actually added.
    pub fn insert(&self, key: Value, value: Value) {
        self.insert_at(None, key, value);
    }

    /// Inserts or updates, with the matching slot already resolved.
    ///
    /// `IRIS-V1-COLLECTIONS-C028` dispatches each key's CURRENT `==` Method,
    /// which only the evaluator can perform, so the caller resolves the slot
    /// and passes it here. `None` means no existing key compared equal, and the
    /// derived comparison is used only for the built-in value keys.
    /// The bucket recorded for a slot at insertion time.
    #[must_use]
    pub fn bucket_at(&self, slot: usize) -> Option<Value> {
        self.0.borrow().buckets.get(slot).cloned()
    }

    /// Inserts or updates, recording the bucket the key hashed to.
    pub fn insert_bucketed(&self, slot: Option<usize>, key: Value, value: Value, bucket: Value) {
        {
            let mut body = self.0.borrow_mut();
            if let Some(index) = slot
                && let Some(entry) = body.entries.get_mut(index)
            {
                entry.1 = value;
                return;
            }
            body.entries.push((key, value));
            body.buckets.push(bucket);
            body.version = body.version.saturating_add(1);
        }
    }

    pub fn insert_at(&self, slot: Option<usize>, key: Value, value: Value) {
        let mut body = self.0.borrow_mut();
        if let Some(index) = slot {
            if let Some(entry) = body.entries.get_mut(index) {
                entry.1 = value;
                return;
            }
        } else if let Some(existing) = body
            .entries
            .iter_mut()
            .find(|(held, _)| *held == key)
            .map(|(_, slot)| slot)
        {
            *existing = value;
            return;
        }
        body.entries.push((key, value));
        body.version = body.version.saturating_add(1);
    }

    /// Returns the value at a resolved slot.
    #[must_use]
    pub fn value_at(&self, slot: usize) -> Option<Value> {
        self.0
            .borrow()
            .entries
            .get(slot)
            .map(|(_, value)| value.clone())
    }

    /// Removes the entry at a resolved slot, answering its value.
    pub fn remove_at(&self, slot: usize) -> Option<Value> {
        let mut body = self.0.borrow_mut();
        if slot >= body.entries.len() {
            return None;
        }
        let (_, value) = body.entries.remove(slot);
        if slot < body.buckets.len() {
            body.buckets.remove(slot);
        }
        body.version = body.version.saturating_add(1);
        Some(value)
    }

    /// Removes `key`, answering the value it held.
    ///
    /// Removal is structural under `C034`, so the version moves whenever an
    /// entry actually leaves.
    pub fn remove(&self, key: &Value) -> Option<Value> {
        let mut body = self.0.borrow_mut();
        let position = body.entries.iter().position(|(held, _)| held == key)?;
        let (_, value) = body.entries.remove(position);
        body.version = body.version.saturating_add(1);
        Some(value)
    }

    /// Removes every entry.
    pub fn clear(&self) {
        let mut body = self.0.borrow_mut();
        if body.entries.is_empty() {
            return;
        }
        body.entries.clear();
        body.version = body.version.saturating_add(1);
    }

    /// Replaces the entries wholesale, counting the change as structural.
    ///
    /// `C031` builds and validates a rehash replacement BEFORE installing it,
    /// so the caller commits an already-checked entry set here.
    pub fn replace_entries(&self, entries: Vec<(Value, Value)>) {
        let mut body = self.0.borrow_mut();
        // C031 rebuilds from CURRENT hashes, so stale buckets are cleared and
        // the caller records the fresh ones.
        body.buckets = vec![Value::Nil; entries.len()];
        body.entries = entries;
        body.version = body.version.saturating_add(1);
    }
}

/// Renders as the entry list, so a Hash's rendering does not depend on its
/// mutation history.
impl core::fmt::Debug for HashRef {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.0.borrow().entries, formatter)
    }
}

/// Compares the current ENTRIES rather than Hash identity.
impl PartialEq for HashRef {
    fn eq(&self, other: &Self) -> bool {
        if self.same(other) {
            return true;
        }
        self.0.borrow().entries == other.0.borrow().entries
    }
}

/// A shared, mutable ByteArray body with the `IRIS-V1-COLLECTIONS-C075` version.
///
/// `C067` makes `ByteArray` identity-bearing, so this is a HANDLE, and `C075`
/// requires ANY content mutation to invalidate active iterators, which is
/// stricter than the Hash rule where only structural change counts.
#[derive(Clone)]
pub struct ByteArrayRef(Rc<RefCell<ByteArrayBody>>);

/// The bytes and content version behind a [`ByteArrayRef`].
#[derive(Debug)]
pub struct ByteArrayBody {
    bytes: Vec<u8>,
    version: u64,
}

impl ByteArrayRef {
    /// Creates a new ByteArray holding `bytes`, at content version zero.
    #[must_use]
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(Rc::new(RefCell::new(ByteArrayBody { bytes, version: 0 })))
    }

    /// Reads the current bytes.
    #[must_use]
    pub fn bytes(&self) -> Vec<u8> {
        self.0.borrow().bytes.clone()
    }

    /// Returns the current byte count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.borrow().bytes.len()
    }

    /// Returns whether the ByteArray currently holds no bytes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the current `C075` content version.
    #[must_use]
    pub fn version(&self) -> u64 {
        self.0.borrow().version
    }

    /// Returns whether two handles denote the SAME ByteArray.
    #[must_use]
    pub fn same(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }

    /// Mutates the bytes, incrementing the `C075` content version.
    pub fn mutate<T>(&self, change: impl FnOnce(&mut Vec<u8>) -> T) -> T {
        let mut body = self.0.borrow_mut();
        let outcome = change(&mut body.bytes);
        body.version = body.version.saturating_add(1);
        outcome
    }
}

/// Renders as the byte sequence, so the rendering does not depend on mutation
/// history.
impl core::fmt::Debug for ByteArrayRef {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.0.borrow().bytes, formatter)
    }
}

/// `C068` compares the exact current byte SEQUENCE rather than identity.
impl PartialEq for ByteArrayRef {
    fn eq(&self, other: &Self) -> bool {
        if self.same(other) {
            return true;
        }
        self.0.borrow().bytes == other.0.borrow().bytes
    }
}

/// The endpoints, endpoint openness and step of a Range.
///
/// `IRIS-V1-COLLECTIONS-C039` makes all four part of Range equality and of the
/// public hash. They are boxed so a Range does not widen every `Value`, and
/// through it every Result that carries one.
#[derive(Clone, Debug, PartialEq)]
pub struct RangeValue {
    /// The first value the Range would yield.
    pub start: IntegerValue,
    /// The endpoint, included only when `inclusive_end` is set.
    pub end: IntegerValue,
    /// Whether the endpoint is included, as `..=` rather than `..<`.
    pub inclusive_end: bool,
    /// The nonzero stride between yielded values.
    pub step: IntegerValue,
}

/// One loaded `FFI::Library`.
///
/// `IRIS-V1-FFI-C005` makes `FFI.open` the ONLY script-originated path into an
/// external binary, so a Library records exactly which symbols were bound with
/// a verified signature and nothing else is callable through it.
#[derive(Clone, Debug, PartialEq)]
pub struct LibraryValue {
    /// The requested library path.
    pub path: String,
    /// Symbols bound with a complete `C047` signature.
    pub bound: Vec<String>,
}

/// The canonical pattern and flags of a Regex.
///
/// `IRIS-V1-COLLECTIONS-C081` stores flags in the fixed order `imsx` with
/// absent flags omitted, so `/a/im` and `/a/mi` compare and hash equal.
#[derive(Clone, Debug, PartialEq)]
pub struct RegexValue {
    /// The canonical pattern text.
    pub pattern: String,
    /// The canonical flags, in `imsx` order.
    pub flags: String,
}

/// One immutable match result.
///
/// `IRIS-V1-COLLECTIONS-C083` requires capture ABSENCE to stay distinct from an
/// empty capture, which is why the captures are `Option`s rather than empty
/// strings, and requires scalar ranges alongside UTF-8 byte ranges.
#[derive(Clone, Debug, PartialEq)]
pub struct MatchValue {
    /// The full matched text.
    pub text: String,
    /// The byte offsets of the full match in the subject's UTF-8 encoding.
    pub byte_start: usize,
    /// The exclusive end byte offset of the full match.
    pub byte_end: usize,
    /// The scalar offsets of the full match in the subject.
    pub scalar_start: usize,
    /// The exclusive end scalar offset of the full match.
    pub scalar_end: usize,
    /// Numbered captures, where `None` marks a group that did not participate.
    pub captures: Vec<Option<String>>,
    /// Named captures, in pattern order.
    pub named: Vec<(String, Option<String>)>,
    /// The Regex that produced this Match.
    pub regex: RegexValue,
}

/// A shared, mutable text body.
///
/// `IRIS-V1-COLLECTIONS-C052` makes MutableString identity-bearing, so this is
/// a HANDLE, and `C053` makes `to_string` copy out of it so a snapshot does not
/// observe later mutation.
#[derive(Clone)]
pub struct MutableStringRef(Rc<RefCell<(String, u64)>>);

impl MutableStringRef {
    /// Creates a fresh MutableString identity holding `text`.
    #[must_use]
    pub fn new(text: String) -> Self {
        Self(Rc::new(RefCell::new((text, 0))))
    }

    /// Copies the current text out.
    #[must_use]
    pub fn text(&self) -> String {
        self.0.borrow().0.clone()
    }

    /// The `IRIS-V1-COLLECTIONS-C061` content version.
    ///
    /// Any content change increments it, so a scalar or grapheme iterator that
    /// captured an older value fails fast on its next advance.
    #[must_use]
    pub fn version(&self) -> u64 {
        self.0.borrow().1
    }

    /// Returns whether two handles denote the SAME MutableString.
    #[must_use]
    pub fn same(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }

    /// Replaces the content, which `C058` requires callers to have fully
    /// prepared first so the commit itself cannot fail partway.
    pub fn set(&self, text: String) {
        let mut body = self.0.borrow_mut();
        body.0 = text;
        body.1 = body.1.saturating_add(1);
    }
}

/// Renders as the current text.
impl core::fmt::Debug for MutableStringRef {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.0.borrow().0, formatter)
    }
}

/// `C055` compares current exact scalar CONTENT rather than identity.
impl PartialEq for MutableStringRef {
    fn eq(&self, other: &Self) -> bool {
        self.same(other) || self.0.borrow().0 == other.0.borrow().0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// The singleton nil value.
    Nil,
    /// A Boolean singleton value.
    Bool(bool),
    /// An arbitrary-precision integer.
    Integer(IntegerValue),
    /// An IEEE-754 binary32 value.
    Float32(f32),
    /// An IEEE-754 binary64 value.
    Float64(f64),
    /// A literal Iris Array.
    ///
    /// `IRIS-V1-COLLECTIONS-C003` classifies `Array<T>` as IDENTITY-BEARING
    /// with a mutable element sequence, so two bindings to one Array observe
    /// each other's mutations. The elements therefore live behind a shared
    /// handle rather than being copied on every bind, pass and read.
    Array(ArrayRef),
    /// A runtime-owned `ReadonlyArray` view.
    ///
    /// `IRIS-V1-CONTROL-D-142` lets user code iterate and copy a suppressed
    /// collection but never insert, delete, replace, or reorder it, so the
    /// read-only view is a DISTINCT value rather than an ordinary Array that
    /// happens not to be mutated.
    ReadonlyArray(Vec<Value>),
    /// A `SourceLocation` record: path, one-based line, one-based column.
    ///
    /// `IRIS-V1-CONTROL-C079` makes it an immutable identity-less value that
    /// compares and hashes STRUCTURALLY, unlike `ExceptionContext`, so two
    /// locations naming the same position are equal.
    SourceLocation(String, u32, u32),
    /// A `StackFrame` record: the callable's name and its location.
    StackFrame(String, Box<Value>),
    /// A `RaiseSite` record: the location a bare `raise` continued from.
    RaiseSite(Box<Value>),
    /// A Range value: start, end, and whether the end is inclusive.
    ///
    /// `IRIS-V1-COLLECTIONS-C006` gives `a ..= b` an inclusive end and
    /// `a ..< b` an exclusive one, and `C007` fixes both endpoints as
    /// Integers. `C003` makes it identity-less with a specification-stable
    /// hash, so two Ranges over the same interval are one value.
    /// A Range: start, end, inclusive end, and step.
    ///
    /// `IRIS-V1-COLLECTIONS-C039` makes the STEP part of Range equality and of
    /// the public hash, so it is carried in the value rather than inferred at
    /// each use.
    Range(Box<RangeValue>),
    /// A `Task<T>`, produced by invoking an async callable.
    ///
    /// `IRIS-V1-ASYNC-C006` makes it identity-bearing and gives it
    /// runtime-local equality and hash, so its identity does not depend on the
    /// eventual result, the exception, or its scheduler queue position.
    Task(ObjectId),
    /// A generator, produced by invoking a callable containing `yield`.
    ///
    /// `IRIS-V1-GRAMMAR-C072` makes it an `Iterator<T>` satisfying
    /// `IRIS-V1-COLLECTIONS-C011`, so `for` drives it through the same
    /// `iterator()`/`next()` protocol every other Iterator uses.
    Generator(ObjectId),
    /// A cursor over an Array, produced by `Array#iterator`.
    ///
    /// `IRIS-V1-COLLECTIONS-C011` makes Array iterable and `C012` drives `for`
    /// through `iterator()`/`next()`. The cursor carries its own position, so
    /// nested traversals of one Array use distinct Iterator objects as `C037`
    /// requires of the collection iterators generally.
    ArrayIterator(ObjectId),
    /// A live cursor over one Hash.
    ///
    /// `IRIS-V1-COLLECTIONS-C037` makes nested traversals use DISTINCT Iterator
    /// objects and denies any hidden current-iterator context, so each cursor
    /// is its own identity-bearing object rather than state on the Hash.
    HashIterator(ObjectId),
    /// A live cursor over a Bytes or ByteArray sequence.
    ByteIterator(ObjectId),
    /// A literal Iris `Hash<K,V>`.
    ///
    /// `IRIS-V1-COLLECTIONS-C033` leaves iteration order UNSPECIFIED, and
    /// `C028` dispatches each key's current `==` rather than a container-owned
    /// relation. Entries are therefore kept as an association list keyed by
    /// `Value` equality instead of a host `HashMap`, which would impose both a
    /// host hash and a host equality the clauses do not permit.
    /// A Gate: an Awaitable completed by an external post.
    ///
    /// `IRIS-V1-ASYNC-C014` lets Host completions enter the scheduler in the
    /// order they are POSTED, and a Gate is that post made observable to a
    /// fixture. It is identity-bearing because two awaits on the SAME Gate must
    /// both resume from one completion.
    Gate(ObjectId),
    /// An `FFI::Library`.
    ///
    /// `IRIS-V1-FFI-C043` makes the Library IDENTITY-BEARING, and `C045`
    /// requires every callable symbol to carry a verified signature, so the
    /// bound symbol names travel with it.
    Library(Box<LibraryValue>),
    /// An immutable Iris `Regex`.
    ///
    /// `IRIS-V1-COLLECTIONS-C077` makes Regex an immutable identity-LESS core
    /// value whose equality and public hash use canonical pattern text plus
    /// canonical flags, so those two strings are the value.
    Regex(Box<RegexValue>),
    /// An immutable Iris `Match`.
    ///
    /// `C083` makes a Match immutable and forbids exposing global variables or
    /// mutable engine state, so it carries its own resolved captures.
    Match(Box<MatchValue>),
    /// An identity-bearing mutable Iris `MutableString`.
    ///
    /// `IRIS-V1-COLLECTIONS-C052` makes each `m` literal evaluation create a
    /// FRESH identity, and `C053` makes `to_string` a snapshot, so the text
    /// lives behind a shared body and a snapshot copies out of it.
    MutableString(MutableStringRef),
    /// An immutable Iris `Bytes` value.
    ///
    /// `IRIS-V1-COLLECTIONS-C067` makes Bytes an IMMUTABLE identity-less byte
    /// sequence, so it is held by value, and `C068` makes its public hash
    /// stable while a ByteArray's raises.
    Bytes(Vec<u8>),
    /// An identity-bearing mutable Iris `ByteArray`.
    ///
    /// `C067` makes ByteArray identity-bearing and mutable, so it carries a
    /// shared body exactly as Array does, and `C075` requires its iterators to
    /// capture a content version.
    ByteArray(ByteArrayRef),
    /// An Iris Tuple.
    ///
    /// `IRIS-V1-COLLECTIONS-C021` makes a Tuple an IMMUTABLE identity-less
    /// heterogeneous product value, so unlike Array it is held by value and
    /// `C022` compares arity and elements in order.
    Tuple(Vec<Value>),
    /// `IRIS-V1-COLLECTIONS-C003` additionally classifies `Hash<K,V>` as
    /// IDENTITY-BEARING, so the entries live behind a shared handle and two
    /// bindings to one Hash observe each other's mutations.
    Hash(HashRef),
    /// An Iris String value.
    ///
    /// `IRIS-V1-COLLECTIONS-C041` makes a String contain only valid Unicode
    /// scalar values, and `C043` compares the exact scalar sequence and case
    /// with no normalization, case folding, or locale mapping, which is exactly
    /// what a host `String` comparison already does.
    Text(String),
    /// An interned Iris Symbol spelling.
    Symbol(String),
    /// A logical built-in Class object.
    Class(ClassId),
    /// An object owned by the runtime heap.
    Object(ObjectId),
    /// A Method bound to one receiver at member-read time.
    BoundMethod(BoundMethod),
    /// An unbound reflective Method object.
    Method(Method),
    /// An immutable Contract view over a receiver.
    ///
    /// `IRIS-V1-TYPES-C050` makes views identity-less capability values, so this
    /// carries the receiver and Contract identity rather than an allocation.
    ContractView(Box<Value>, ContractId),
    /// A declared Contract object.
    ///
    /// `IRIS-V1-TYPES-C041` makes Contract the obligation surface, and
    /// `IRIS-V1-TYPES-C076` requires Class, Module, Contract and Type objects to
    /// stay mutually distinct, so a Contract carries its own identity rather
    /// than reusing `ClassId` or `ModuleId`.
    Contract(ContractId),
    /// A decorator `Transformation`, the candidate transformation C125 fixes.
    ///
    /// `IRIS-V1-META-C125` gives it a MINIMAL surface: `empty`, `kind` and
    /// `add_method(selector, body)`. The staged Methods are carried as
    /// `(selector, closure)` pairs so the runtime phase applies them through
    /// the ordinary capability-checked publication path that
    /// `IRIS-V1-META-C090` requires of a handwritten declaration, rather than
    /// through a privileged back door of its own.
    Transformation {
        kind: &'static str,
        staged: Vec<(String, ObjectId)>,
    },
    /// An `Iteration.yield(value)` result carrying one yielded value.
    ///
    /// `IRIS-V1-COLLECTIONS-C013` makes it an immutable identity-less value that
    /// MAY carry any Iris value including `nil`, which is why a yielded `nil`
    /// must stay distinguishable from exhaustion.
    IterationYield(Box<Value>),
    /// The unique `Iteration.done` singleton.
    IterationDone,
    /// One evaluated `name: value` argument in flight to a call.
    ///
    /// `IRIS-V1-CONTROL-C026` fixes evaluation order across the positional and
    /// keyword channels, so a keyword argument is evaluated in place alongside
    /// the positionals and carries its name to the binding step rather than
    /// being split into a separate pre-evaluated list.
    KeywordArgument(String, Box<Value>),
    /// An identity-bearing `ExceptionContext` for one propagation event.
    ///
    /// `IRIS-V1-CONTROL-C056` gives every `raise` a fresh runtime-owned context
    /// carrying the raised value, and `IRIS-V1-CONTROL-C057` lets `raise value
    /// from cause` chain an explicit one, where `nil` suppresses chaining.
    ///
    /// The leading `ObjectId` is that identity. `C056` makes each propagation
    /// event DISTINCT, so re-raising the same value must produce a context that
    /// `same?` separates from the one being handled; comparing the payload
    /// structurally would wrongly make those two equal.
    /// The trailing `Vec` is `re_raise_sites`, which `IRIS-V1-CONTROL-D-155`
    /// makes an ORDERED sequence appended to by each bare `raise` without
    /// replacing the root stack, so multiple sites retain occurrence order.
    /// The trailing `Box<Value>` is `raise_location`, the `SourceLocation` of
    /// the INITIAL raise, which `IRIS-V1-CONTROL-C065` exposes get-only.
    ExceptionContext(
        ObjectId,
        Box<Value>,
        Box<Value>,
        Vec<Value>,
        Vec<Value>,
        Box<Value>,
    ),
    /// An identity-bearing Closure object.
    ///
    /// `IRIS-V1-RUNTIME-C042` requires each evaluation of a Closure expression to
    /// create a NEW identity-bearing object with its own captured environment,
    /// and makes default equality identity-only, so this carries an allocation
    /// identity rather than the code it runs.
    Closure(ObjectId),
    /// An interned Type object, distinct from the Class it reifies.
    ///
    /// `IRIS-V1-TYPES-C016` requires Type objects to be interned and
    /// identity-bearing, and `IRIS-V1-TYPES-C076` requires them to be distinct
    /// from the Class object, so a nominal Type carries the ClassId rather than
    /// being that ClassId.
    ///
    /// `IRIS-V1-TYPES-D-206` interns a CLOSED identity by definition AND
    /// normalized arguments, so `Box<String>` and `Box<Integer>` are different
    /// Types of one definition. The arguments therefore travel with the
    /// ClassId; dropping them made every construction of a definition one
    /// interned Type.
    Type(ClassId, Vec<ClassId>),
    /// An interned COMPOSED Type: a union or intersection reduced to its
    /// normal form.
    ///
    /// `IRIS-V1-TYPES-C016` interns Type objects by identity, and the chapter's
    /// normalization law table requires `String | Integer` and
    /// `Integer | String` to be ONE interned Type. A composed Type therefore
    /// carries a canonical member list rather than the written order, so
    /// commutativity, idempotence, and absorption hold by construction.
    ComposedType(ComposedType),
}

/// The normal form of a union or intersection Type.
///
/// Members are sorted and deduplicated when the form is built, so two
/// spellings of one Type compare equal without a separate normalization pass.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ComposedType {
    /// The uninhabited Type. `IRIS-V1-TYPES-C023` makes it absorbing in an
    /// intersection and an identity in a union.
    Never,
    Union(Vec<TypeAtom>),
    Intersection(Vec<TypeAtom>),
}

/// One irreducible constituent of a composed Type.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TypeAtom {
    /// A nominal Class, with the generic arguments D-206 interns it by.
    Nominal(ClassId, Vec<ClassId>),
    /// `NonNil`, which C011 makes a Type rather than a declared Class.
    NonNil,
    /// A named Contract used as a Type.
    ///
    /// `IRIS-V1-TYPES-V002` states intersection commutativity over two
    /// CONTRACTS, so a Contract is an irreducible constituent alongside a
    /// nominal Class.
    Contract(crate::ContractId),
    /// A nested UNION kept as ONE constituent of an intersection.
    ///
    /// `IRIS-V1-TYPES-V016` keeps `A & (B | C)` a COMPACT intersection
    /// CONTAINING the union member rather than distributing it, and `V214`
    /// reflects exactly those two members. Flattening the union into the
    /// enclosing intersection would lose that structure entirely.
    Union(Vec<TypeAtom>),
}
