use embedded_hal::serial;

// note: CoreRead is only implemented for serial::Read if serial::Read::Error
// implements core::fmt::Debug. This is due to a limitation in serde

impl<'a, T> super::CoreRead<'a> for T
where
    T: serial::Read<u8>,
    <T as serial::Read<u8>>::Error: core::fmt::Debug,
{
    type Error = <T as serial::Read<u8>>::Error;

    fn fill(&mut self, buffer: &mut [u8]) -> Result<(), Self::Error> {
        for b in buffer {
            *b = nb::block!(self.read())?;
        }
        Ok(())
    }

    fn forward_str<V>(&mut self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'a>,
    {
        unimplemented!()
    }

    fn forward_bytes<V>(&mut self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'a>,
    {
        unimplemented!()
    }
}

impl<T> super::CoreWrite for T
where
    T: serial::Write<u8>,
    <T as serial::Write<u8>>::Error: core::fmt::Debug,
{
    type Error = <T as serial::Write<u8>>::Error;

    fn write(&mut self, val: u8) -> Result<(), Self::Error> {
        nb::block!(serial::Write::write(self, val))
    }
}
