/// A circular buffer.
pub struct CircularBuffer<T: Default, const SIZE: usize> {
    buffer: [T; SIZE],
    start: usize,
    length: usize,
}

impl<const SIZE: usize> CircularBuffer<u8, SIZE> {
    pub const fn new() -> Self {
        Self {
            buffer: [0; SIZE],
            start: 0,
            length: 0,
        }
    }
}

impl<T: Copy + Default, const SIZE: usize> Default for CircularBuffer<T, SIZE> {
    fn default() -> Self {
        Self {
            buffer: [T::default(); SIZE],
            start: 0,
            length: 0,
        }
    }
}

impl<T: Copy + Default, const SIZE: usize> CircularBuffer<T, SIZE> {
    /// Try to add the given element to the buffer. Returns true on success, or false if the buffer
    /// was already full.
    pub fn add(&mut self, element: T) -> bool {
        if self.length == self.buffer.len() {
            return false;
        }
        self.buffer[(self.start + self.length) % self.buffer.len()] = element;
        self.length += 1;
        true
    }

    /// Add as many elements as possible from the given slice to the buffer. Returns the number of
    /// elements added.
    pub fn add_all(&mut self, elements: &[T]) -> usize {
        let mut added = 0;
        for &element in elements {
            if self.add(element) {
                added += 1;
            } else {
                break;
            }
        }
        added
    }

    /// Take one element out of the buffer, if it has any.
    pub fn take(&mut self) -> Option<T> {
        if self.length == 0 {
            None
        } else {
            let element = self.buffer[self.start];
            self.start = (self.start + 1) % self.buffer.len();
            self.length -= 1;
            Some(element)
        }
    }

    /// Get a copy of the next element from the buffer, but don't remove it.
    pub fn peek(&self) -> Option<T> {
        if self.length == 0 {
            None
        } else {
            Some(self.buffer[self.start])
        }
    }

    /// Returns true if there are no elements in the buffer.
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Returns true if there is no space in the buffer for any more elements.
    pub fn is_full(&self) -> bool {
        self.length == self.buffer.len()
    }
}
