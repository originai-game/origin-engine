use super::*;

mod sealed {
    pub trait Sealed {}
    pub trait SealedField {}
}

/// A Rust type a column can be viewed as (`ArchetypeView::read::<T>`): plain
/// old data whose every bit pattern is a value. Sealed: `f32`, `f64`, `i32`,
/// `u32`, `i64`, `u64`, `Vec2`, `Vec3`, `Vec4`, `Quat`, `Mat4` and [`Bool8`]
/// (a bool column). Every `FieldValue` is a [`ReflectField`]; `bool` and
/// [`Str<N>`] are fields but not views (a string column is read through
/// [`unpack_str`]).
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a column element type",
    note = "a column is viewed as f32, f64, i32, u32, i64, u64, Vec2, Vec3, Vec4, Quat, Mat4 or Bool8 (for a bool column)"
)]
pub trait FieldValue: Copy + bytemuck::Pod + sealed::Sealed + 'static {
    const TYPE: FieldType;

    fn zero() -> Self {
        bytemuck::Zeroable::zeroed()
    }

    fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }

    fn as_bytes_mut(&mut self) -> &mut [u8] {
        bytemuck::bytes_of_mut(self)
    }
}

macro_rules! field_value {
    ($($t:ty => $ft:ident),* $(,)?) => {$(
        impl sealed::Sealed for $t {}
        impl FieldValue for $t {
            const TYPE: FieldType = FieldType::$ft;
        }
    )*};
}

field_value! {
    f32 => F32, f64 => F64, i32 => I32, u32 => U32, i64 => I64, u64 => U64,
    glam::Vec2 => Vec2, glam::Vec3 => Vec3, glam::Vec4 => Vec4, glam::Quat => Quat,
    glam::Mat4 => Mat4, Bool8 => Bool,
}

/// One element of a `bool` column as a view sees it: the byte itself. `0` is
/// false and anything else true; the engine writes 0 or 1.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(transparent)]
pub struct Bool8(pub u8);

impl Bool8 {
    pub const FALSE: Bool8 = Bool8(0);
    pub const TRUE: Bool8 = Bool8(1);

    pub const fn get(self) -> bool {
        self.0 != 0
    }

    pub fn set(&mut self, on: bool) {
        self.0 = u8::from(on);
    }
}

impl From<bool> for Bool8 {
    fn from(on: bool) -> Self {
        Bool8(u8::from(on))
    }
}

impl From<Bool8> for bool {
    fn from(b: Bool8) -> Self {
        b.get()
    }
}

impl std::fmt::Debug for Bool8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get())
    }
}

/// A Rust type a `#[derive(Reflect)]` field may have: every [`FieldValue`],
/// plus `bool` (a `bool` field) and [`Str<N>`] (a `strN` field). An enum field is a `u32` with `#[reflect(variants = [...])]`.
/// Sealed. The two methods are what the derive generates calls to.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a reflected field type",
    note = "fields of a #[derive(Reflect)] struct are f32, f64, i32, u32, i64, u64, Vec2, Vec3, Vec4, Quat, Mat4, bool, Bool8 or Str<N> (an enum is a u32 with #[reflect(variants = [..])])"
)]
pub trait ReflectField: Copy + Send + Sync + sealed::SealedField + 'static {
    const TYPE: FieldType;

    /// Calls `out` once with the packed bytes of the value.
    #[doc(hidden)]
    fn write_packed(&self, out: &mut dyn FnMut(&[u8]));

    /// A value from `input`, called once to fill the packed bytes.
    #[doc(hidden)]
    fn read_packed(input: &mut dyn FnMut(&mut [u8])) -> Self;
}

impl<T: FieldValue + Send + Sync> sealed::SealedField for T {}

impl<T: FieldValue + Send + Sync> ReflectField for T {
    const TYPE: FieldType = <T as FieldValue>::TYPE;

    fn write_packed(&self, out: &mut dyn FnMut(&[u8])) {
        out(self.as_bytes());
    }

    fn read_packed(input: &mut dyn FnMut(&mut [u8])) -> Self {
        let mut v = <T as FieldValue>::zero();
        input(v.as_bytes_mut());
        v
    }
}

impl sealed::SealedField for bool {}

impl sealed::SealedField for crate::ease::Ease {}

impl ReflectField for crate::ease::Ease {
    const TYPE: FieldType = FieldType::Ease;

    fn write_packed(&self, out: &mut dyn FnMut(&[u8])) {
        out(&self.to_bytes());
    }

    fn read_packed(input: &mut dyn FnMut(&mut [u8])) -> Self {
        let mut bytes = [0; 20];
        input(&mut bytes);
        Self::from_bytes(&bytes).expect("validated reflected ease")
    }
}

impl ReflectField for bool {
    const TYPE: FieldType = FieldType::Bool;

    fn write_packed(&self, out: &mut dyn FnMut(&[u8])) {
        out(&[u8::from(*self)]);
    }

    fn read_packed(input: &mut dyn FnMut(&mut [u8])) -> Self {
        let mut b = [0u8];
        input(&mut b);
        b[0] != 0
    }
}

/// A string field: UTF-8 text of at most `N` bytes stored inline,
/// zero-padded (`str32` is `Str<32>`). `N` is 1 to [`STR_MAX_CAPACITY`],
/// checked at compile time.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Str<const N: usize>([u8; N]);

/// A `str64` field.
pub type Str64 = Str<64>;

/// A `str256` field.
pub type Str256 = Str<256>;

impl<const N: usize> Str<N> {
    /// The capacity in bytes of UTF-8.
    pub const CAPACITY: usize = N;

    /// The empty string.
    pub const EMPTY: Self = Str([0; N]);

    /// `text` as an inline string. Refused when it is longer than the
    /// capacity in UTF-8 bytes or holds a NUL character ([`pack_str`]).
    pub fn new(text: &str) -> Result<Self> {
        let packed = pack_str(text, N).map_err(|reason| Error::new(ErrorKind::Invalid, reason))?;
        let mut out = [0u8; N];
        out.copy_from_slice(&packed);
        Ok(Str(out))
    }

    /// The text: the bytes before the first zero ([`unpack_str`]).
    pub fn as_str(&self) -> &str {
        unpack_str(&self.0)
    }

    /// The packed bytes.
    pub fn as_bytes(&self) -> &[u8; N] {
        &self.0
    }
}

impl<const N: usize> Default for Str<N> {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl<const N: usize> std::fmt::Debug for Str<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_str(), f)
    }
}

impl<const N: usize> std::fmt::Display for Str<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<const N: usize> sealed::SealedField for Str<N> {}

impl<const N: usize> ReflectField for Str<N> {
    const TYPE: FieldType = {
        assert!(
            N >= 1 && N <= STR_MAX_CAPACITY,
            "a Str<N> holds 1 to STR_MAX_CAPACITY (4096) bytes"
        );
        FieldType::Str(N as u16)
    };

    fn write_packed(&self, out: &mut dyn FnMut(&[u8])) {
        out(&self.0);
    }

    fn read_packed(input: &mut dyn FnMut(&mut [u8])) -> Self {
        let mut b = [0u8; N];
        input(&mut b);
        Str(b)
    }
}

/// A number type a [`List`] holds: `f32`, `f64`, `i32`, `u32`, `i64`,
/// `u64` (a handle is a `u64`). Sealed.
pub trait ListElement: FieldValue + Send + Sync {
    #[doc(hidden)]
    const SCALAR: Scalar;
}

macro_rules! list_element {
    ($($t:ty => $s:ident),* $(,)?) => {$(
        impl ListElement for $t {
            const SCALAR: Scalar = Scalar::$s;
        }
    )*};
}

list_element! { f32 => F32, f64 => F64, i32 => I32, u32 => U32, i64 => I64, u64 => U64 }

/// A list field: at most `N` values of `T` stored inline (`u64[8]` is
/// `List<u64, 8>`). `N` is 1 to [`LIST_MAX_CAPACITY`], checked at compile
/// time. The slots past the length are kept zero, so two lists with the same
/// elements are the same bytes.
#[derive(Clone, Copy, PartialEq)]
pub struct List<T: ListElement, const N: usize> {
    items: [T; N],
    len: usize,
}

impl<T: ListElement, const N: usize> List<T, N> {
    /// The capacity in elements.
    pub const CAPACITY: usize = N;

    /// The empty list.
    pub fn new() -> Self {
        List {
            items: [T::zero(); N],
            len: 0,
        }
    }

    /// A list of `items`, refused past the capacity ([`pack_list`]'s reason).
    pub fn from_slice(items: &[T]) -> Result<Self> {
        let mut out = Self::new();
        for &x in items {
            out.push(x)?;
        }
        Ok(out)
    }

    /// Appends `x`, refused when the list is full.
    pub fn push(&mut self, x: T) -> Result<()> {
        if self.len == N {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("the list is full: it holds {N}"),
            ));
        }
        self.items[self.len] = x;
        self.len += 1;
        Ok(())
    }

    /// Empties the list.
    pub fn clear(&mut self) {
        self.items = [T::zero(); N];
        self.len = 0;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_slice(&self) -> &[T] {
        &self.items[..self.len]
    }
}

impl<T: ListElement, const N: usize> Default for List<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ListElement + std::fmt::Debug, const N: usize> std::fmt::Debug for List<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.as_slice()).finish()
    }
}

impl<T: ListElement, const N: usize> sealed::SealedField for List<T, N> {}

impl<T: ListElement, const N: usize> ReflectField for List<T, N> {
    const TYPE: FieldType = {
        assert!(
            N >= 1 && N <= LIST_MAX_CAPACITY,
            "a List<T, N> holds 1 to LIST_MAX_CAPACITY (1024) elements"
        );
        FieldType::List(T::SCALAR, N as u16)
    };

    fn write_packed(&self, out: &mut dyn FnMut(&[u8])) {
        let mut bytes = vec![0u8; <Self as ReflectField>::TYPE.size()];
        let lane = T::SCALAR.size();
        T::SCALAR.count_lane(self.len, &mut bytes[..lane]);
        for (i, x) in self.items.iter().enumerate() {
            bytes[lane * (i + 1)..lane * (i + 2)].copy_from_slice(x.as_bytes());
        }
        out(&bytes);
    }

    fn read_packed(input: &mut dyn FnMut(&mut [u8])) -> Self {
        let ty = <Self as ReflectField>::TYPE;
        let mut bytes = vec![0u8; ty.size()];
        input(&mut bytes);
        let mut out = Self::new();
        for e in list_elements(ty, &bytes) {
            let mut x = T::zero();
            x.as_bytes_mut().copy_from_slice(e);
            out.items[out.len] = x;
            out.len += 1;
        }
        out
    }
}
