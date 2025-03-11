use std::ops::{Deref, DerefMut};

use binrw::{BinRead, BinWrite};

use super::LengthType;

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct LengthVec<T, L: LengthType> {
    pub value: Vec<T>,
    marker: std::marker::PhantomData<L>,
}

impl<T, L: LengthType> BinRead for LengthVec<T, L>
where
    T: BinWrite + BinRead,
    for<'a> <T as BinRead>::Args<'a>: Clone,
{
    type Args<'a> = <T as BinRead>::Args<'a>;

    #[inline]
    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let count = <L as LengthType>::parse(reader, endian, ())?;
        let mut value = Vec::with_capacity(count);
        for _ in 0..count {
            value.push(T::read_options(reader, endian, args.clone())?);
        }
        Ok(LengthVec {
            value,
            marker: Default::default(),
        })
    }
}

impl<T: BinRead + BinWrite, L: LengthType> BinWrite for LengthVec<T, L>
where
    T: BinWrite + BinRead,
    for<'a> <T as BinWrite>::Args<'a>: Clone,
{
    type Args<'a> = <T as BinWrite>::Args<'a>;

    #[inline]
    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        <L as LengthType>::write(self.value.len(), writer, endian, ())?;
        for element in &self.value {
            element.write_options(writer, endian, args.clone())?;
        }
        Ok(())
    }
}

impl<T: BinRead + BinWrite, L: LengthType> LengthVec<T, L> {
    #[inline]
    pub fn size(&self) -> usize {
        std::mem::size_of::<L>() + std::mem::size_of::<T>() * self.value.len()
    }
}

impl<T: BinRead + BinWrite, L: LengthType> Deref for LengthVec<T, L> {
    type Target = Vec<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: BinRead + BinWrite, L: LengthType> DerefMut for LengthVec<T, L> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: BinRead + BinWrite, L: LengthType> AsRef<[T]> for LengthVec<T, L> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        &self.value
    }
}

impl<T: BinRead + BinWrite, L: LengthType> AsMut<[T]> for LengthVec<T, L> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.value
    }
}

impl<T: BinRead + BinWrite, L: LengthType> IntoIterator for LengthVec<T, L> {
    type Item = T;
    type IntoIter = <Vec<T> as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.value.into_iter()
    }
}

impl<T: BinRead + BinWrite, L: LengthType> From<LengthVec<T, L>> for Vec<T> {
    #[inline]
    fn from(value: LengthVec<T, L>) -> Self {
        value.value
    }
}

impl<T: BinRead + BinWrite, L: LengthType> From<Vec<T>> for LengthVec<T, L> {
    #[inline]
    fn from(value: Vec<T>) -> Self {
        Self {
            value,
            marker: Default::default(),
        }
    }
}

impl<T: BinRead + BinWrite + Clone, L: LengthType> From<&[T]> for LengthVec<T, L> {
    #[inline]
    fn from(value: &[T]) -> Self {
        Self {
            value: value.into(),
            marker: Default::default(),
        }
    }
}
