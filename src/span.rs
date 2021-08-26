use std::fmt;
use std::ops::{Bound, Deref, DerefMut, Index, Range, RangeBounds};

type PosWidth = u32;

#[derive(
	Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct BytePos(PosWidth);

impl BytePos {
	pub const MAX: Self = Self(PosWidth::MAX);
	pub const MIN: Self = Self(PosWidth::MIN);

	pub const fn from_u32(value: u32) -> Self {
		Self(value)
	}

	pub const fn from_usize(value: usize) -> Self {
		Self(value as u32)
	}

	pub const fn as_u32(&self) -> u32 {
		self.0
	}

	pub const fn as_usize(&self) -> usize {
		self.0 as usize
	}

	pub const fn offset(&self, offset: i32) -> Self {
		Self((self.0 as i32 + offset) as PosWidth)
	}
}

impl fmt::Display for BytePos {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Display::fmt(&self.0, f)
	}
}

impl From<u32> for BytePos {
	fn from(value: u32) -> Self {
		Self::from_u32(value)
	}
}

impl From<usize> for BytePos {
	fn from(value: usize) -> Self {
		Self::from_usize(value)
	}
}

impl From<BytePos> for u32 {
	fn from(value: BytePos) -> Self {
		value.as_u32()
	}
}

impl From<BytePos> for usize {
	fn from(value: BytePos) -> Self {
		value.as_usize()
	}
}

impl Deref for BytePos {
	type Target = PosWidth;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for BytePos {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl Index<BytePos> for &str {
	type Output = u8;

	fn index(&self, index: BytePos) -> &Self::Output {
		&self.as_bytes()[usize::from(index)]
	}
}

#[derive(
	Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct ByteSpan {
	low: BytePos,
	high: BytePos,
}

impl ByteSpan {
	pub fn new(mut low: BytePos, mut high: BytePos) -> Self {
		if low > high {
			std::mem::swap(&mut low, &mut high);
		}

		Self { low, high }
	}

	pub fn from_range<B, R>(range: R) -> Self
	where
		B: Clone + Into<BytePos>,
		R: RangeBounds<B>,
	{
		let low = match range.start_bound() {
			Bound::Included(included) => included.clone().into(),
			Bound::Excluded(excluded) => excluded.clone().into().offset(1),
			Bound::Unbounded => BytePos::MIN,
		};

		let high = match range.end_bound() {
			Bound::Included(included) => included.clone().into().offset(-1),
			Bound::Excluded(excluded) => excluded.clone().into(),
			Bound::Unbounded => BytePos::MAX,
		};

		Self::new(low, high)
	}

	pub const fn with_low(&self, low: BytePos) -> Self {
		Self { low, high: self.high }
	}

	pub const fn with_high(&self, high: BytePos) -> Self {
		Self { low: self.low, high }
	}

	pub const fn offset_low(&self, offset: i32) -> Self {
		Self { low: self.low.offset(offset), high: self.high }
	}

	pub const fn offset_high(&self, offset: i32) -> Self {
		Self { low: self.low, high: self.high.offset(offset) }
	}

	pub const fn offset(&self, offset: i32) -> Self {
		Self { low: self.low.offset(offset), high: self.high.offset(offset) }
	}

	pub fn union(&self, other: &Self) -> Self {
		Self {
			low: std::cmp::min(self.low, other.low),
			high: std::cmp::max(self.high, other.high),
		}
	}

	pub const fn len(&self) -> usize {
		self.high.as_usize() - self.low.as_usize()
	}

	pub fn to_range_usize(self) -> Range<usize> {
		self.low.into()..self.high.into()
	}
}

impl fmt::Display for ByteSpan {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}..{}", &self.low, &self.high)
	}
}

impl RangeBounds<BytePos> for ByteSpan {
	fn start_bound(&self) -> Bound<&BytePos> {
		Bound::Included(&self.high)
	}

	fn end_bound(&self) -> Bound<&BytePos> {
		Bound::Excluded(&self.high)
	}
}

impl Index<ByteSpan> for str {
	type Output = Self;

	fn index(&self, index: ByteSpan) -> &Self::Output {
		Self::index(self, index.to_range_usize())
	}
}
