/// Wrapper that implements async traits using blocking implementations.
///
/// This allows driver writers to depend on the async traits while still supporting embedded-hal peripheral implementations.
///
/// BlockingAsync will implement any async trait that maps to embedded-hal traits implemented for the wrapped driver.
///
/// Driver users are then free to choose which implementation that is available to them.
pub struct BlockingAsync<T> {
    wrapped: T,
}

impl<T> BlockingAsync<T> {
    /// Create a new instance of a wrapper for a given peripheral.
    pub fn new(wrapped: T) -> Self {
        Self { wrapped }
    }
}

//
// I2C implementations
//
impl<T, E> embedded_hal_1::i2c::ErrorType for BlockingAsync<T>
where
    E: embedded_hal_1::i2c::Error + 'static,
    T: embedded_hal_1::i2c::I2c<Error = E>,
{
    type Error = E;
}

impl<T, E> embedded_hal_async::i2c::I2c for BlockingAsync<T>
where
    E: embedded_hal_1::i2c::Error + 'static,
    T: embedded_hal_1::i2c::I2c<Error = E>,
{
    async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        self.wrapped.read(address, read)
    }

    async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        self.wrapped.write(address, write)
    }

    async fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Self::Error> {
        self.wrapped.write_read(address, write, read)
    }

    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal_1::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        self.wrapped.transaction(address, operations)
    }
}

//
// SPI implementatinos
//

impl<T, E> embedded_hal_async::spi::ErrorType for BlockingAsync<T>
where
    E: embedded_hal_async::spi::Error,
    T: embedded_hal_1::spi::SpiBus<Error = E>,
{
    type Error = E;
}

impl<T, E> embedded_hal_async::spi::SpiBus<u8> for BlockingAsync<T>
where
    E: embedded_hal_async::spi::Error,
    T: embedded_hal_1::spi::SpiBus<Error = E>,
{
    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn write(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.wrapped.write(data)?;
        Ok(())
    }

    async fn read(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
        self.wrapped.read(data)?;
        Ok(())
    }

    async fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        self.wrapped.transfer(read, write)?;
        Ok(())
    }

    async fn transfer_in_place(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
        self.wrapped.transfer_in_place(data)?;
        Ok(())
    }
}

/// NOR flash wrapper
use embedded_storage::nor_flash::{ErrorType, MultiwriteNorFlash, NorFlash, ReadNorFlash};
use embedded_storage_async::nor_flash::{
    MultiwriteNorFlash as AsyncMultiwriteNorFlash, NorFlash as AsyncNorFlash, ReadNorFlash as AsyncReadNorFlash,
};

impl<T> ErrorType for BlockingAsync<T>
where
    T: ErrorType,
{
    type Error = T::Error;
}

impl<T> AsyncNorFlash for BlockingAsync<T>
where
    T: NorFlash,
{
    const WRITE_SIZE: usize = <T as NorFlash>::WRITE_SIZE;
    const ERASE_SIZE: usize = <T as NorFlash>::ERASE_SIZE;

    async fn write(&mut self, offset: u32, data: &[u8]) -> Result<(), Self::Error> {
        self.wrapped.write(offset, data)
    }

    async fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        self.wrapped.erase(from, to)
    }
}

impl<T> AsyncReadNorFlash for BlockingAsync<T>
where
    T: ReadNorFlash,
{
    const READ_SIZE: usize = <T as ReadNorFlash>::READ_SIZE;
    async fn read(&mut self, address: u32, data: &mut [u8]) -> Result<(), Self::Error> {
        self.wrapped.read(address, data)
    }

    fn capacity(&self) -> usize {
        self.wrapped.capacity()
    }
}

impl<T> AsyncMultiwriteNorFlash for BlockingAsync<T> where T: MultiwriteNorFlash {}
