use bytemuck::{Pod, checked::pod_read_unaligned};
use core::mem::size_of;
use core::slice::SliceIndex;

#[derive(Debug)]
pub enum ReadError {
    OutOfBounds,
    InvalidData,
}

type Result<T> = core::result::Result<T, ReadError>;

pub(crate) trait Read<Idx: Copy> {
    fn read<T: Pod>(&mut self) -> Result<T>;
    fn seek(&mut self, pos: Idx) -> Result<()>;
}

pub struct MemoryReader<'a, Idx: Copy> {
    memory: &'a [u8],
    cursor: Idx,
}

impl<'r, Idx: Copy + Default> MemoryReader<'r, Idx> {
    pub fn new(memory: &'r [u8]) -> Self {
        MemoryReader {
            memory,
            cursor: Idx::default(),
        }
    }
}
    
impl<'a, Idx> Read<Idx> for MemoryReader<'a, Idx> 
    where 
    Idx: Copy + From<usize> + Into<usize> + core::ops::Add<Idx, Output = Idx> + core::ops::AddAssign<Idx> + core::cmp::PartialOrd,
    core::ops::Range<Idx>: SliceIndex<[u8]>
{
    fn read<T: Pod>(&mut self) -> Result<T> {
        let size = Idx::from(size_of::<T>());
        let start = self.cursor;
        let end = start + size;
        if end > Idx::from(self.memory.len()) {
            return Err(ReadError::OutOfBounds);
        }
        self.cursor += size;
        Ok(pod_read_unaligned::<T>(&self.memory[start.into()..end.into()]).clone())
    }

    fn seek(&mut self, pos: Idx) -> Result<()> {
        self.cursor = pos;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_u8() {
        let data = [0x12u8, 0x34, 0x56, 0x78];
        let mut reader = MemoryReader {
            memory: &data,
            cursor: 0,
        };
        let value1: u8 = reader.read().unwrap();
        let value2: u8 = reader.read().unwrap();
        assert_eq!(value1, 0x12);
        assert_eq!(value2, 0x34);
    }

    #[test]
    fn test_read_u16() {
        let data = [0x12u8, 0x34, 0x56, 0x78];
        let mut reader = MemoryReader {
            memory: &data,
            cursor: 0,
        };
        let value: u16 = reader.read().unwrap();
        assert_eq!(value, 0x3412);
    }

    #[test]
    fn test_read_multiple() {
        let data = [0x01u8, 0x02, 0x03, 0x04, 0x05, 0x06];
        let mut reader = MemoryReader {
            memory: &data,
            cursor: 0,
        };
        let value1: u8 = reader.read().unwrap();
        let value2: u16 = reader.read().unwrap();
        let value3: u8 = reader.read().unwrap();
        assert_eq!(value1, 0x01);
        assert_eq!(value2, 0x0302);
        assert_eq!(value3, 0x04);
    }
}
