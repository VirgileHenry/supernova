#[derive(Debug, Clone, Copy)]
pub struct BufferWriteError {
    buffer_capacity: usize,
    offset: usize,
    length: usize,
}

impl BufferWriteError {
    pub fn new(buffer_capacity: usize, offset: usize, length: usize) -> Self {
        Self {
            buffer_capacity,
            offset,
            length,
        }
    }
}

impl std::fmt::Display for BufferWriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to write to buffer: offset ({}) + length ({}) exceed capacity ({})",
            self.offset, self.length, self.buffer_capacity
        )
    }
}

impl std::error::Error for BufferWriteError {}
