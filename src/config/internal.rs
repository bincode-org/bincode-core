use super::*;

pub trait InternalOptions {
    type Limit: SizeLimit + 'static;
    type Endian: BincodeByteOrder + 'static;
    type IntEncoding: IntEncoding + 'static;
    type Trailing: TrailingBytes + 'static;

    fn limit(&mut self) -> &mut Self::Limit;
}

impl<'a, O: InternalOptions> InternalOptions for &'a mut O {
    type Limit = O::Limit;
    type Endian = O::Endian;
    type IntEncoding = O::IntEncoding;
    type Trailing = O::Trailing;

    #[inline(always)]
    fn limit(&mut self) -> &mut Self::Limit {
        (*self).limit()
    }
}
