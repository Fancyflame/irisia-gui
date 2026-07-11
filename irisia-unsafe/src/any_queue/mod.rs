mod type_eliminate;

pub mod any_box;
pub use any_box::AnyBox;

use std::{
    alloc::{self, Layout},
    any::Any,
    mem,
    ptr::{self, NonNull},
};

use type_eliminate::{get_any_metadata, AnyMetadata};

#[repr(C)]
#[derive(Clone, Copy)]
struct RecordHeader {
    metadata: &'static dyn AnyMetadata,
}

#[derive(Clone, Copy)]
struct Record {
    metadata: &'static dyn AnyMetadata,
    payload_offset: usize,
    next_offset: usize,
    size: usize,
}

#[derive(Clone, Copy)]
struct WriteSlot {
    offset: usize,
    next_offset: usize,
    wraps: bool,
}

/// A byte-packed heterogeneous FIFO queue.
pub struct AnyQueue {
    ptr: NonNull<u8>,
    capacity: usize,
    head: usize,
    tail: usize,
    wrap_at: Option<usize>,
    len: usize,
}

impl AnyQueue {
    /// Creates an empty queue without allocating.
    pub fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            capacity: 0,
            head: 0,
            tail: 0,
            wrap_at: None,
            len: 0,
        }
    }

    /// Returns the number of elements currently in the queue.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns whether the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Appends a value to the back of the queue.
    pub fn push_back<T: Any>(&mut self, value: T) {
        let metadata = get_any_metadata::<T>();
        let record_size = Self::record_size(metadata);
        self.reserve_for(record_size);

        let slot = self
            .find_write_slot(record_size)
            .expect("AnyQueue capacity was not reserved");

        unsafe {
            ptr::write_unaligned(
                self.ptr.as_ptr().add(slot.offset).cast::<RecordHeader>(),
                RecordHeader { metadata },
            );

            let payload_size = metadata.layout().size();
            if payload_size != 0 {
                ptr::copy_nonoverlapping(
                    (&value as *const T).cast::<u8>(),
                    self.ptr.as_ptr().add(slot.offset + Self::header_size()),
                    payload_size,
                );
            }
        }
        mem::forget(value);

        if self.len == 0 {
            self.head = slot.offset;
        }
        if slot.wraps {
            self.wrap_at = Some(self.tail);
        }
        self.tail = slot.next_offset;
        self.len += 1;
    }

    /// Moves the front value into `out`, returning whether a value was present.
    pub fn pop_front(&mut self, out: &mut AnyBox) -> bool {
        if self.len == 0 {
            return false;
        }

        let record = unsafe { self.parse_record(self.head) };
        unsafe {
            out.replace_from_raw(
                record.metadata,
                self.ptr.as_ptr().add(record.payload_offset),
            );
        }

        self.len -= 1;
        if self.len == 0 {
            self.head = 0;
            self.tail = 0;
            self.wrap_at = None;
        } else if self.wrap_at == Some(record.next_offset) {
            self.head = 0;
            self.wrap_at = None;
        } else {
            self.head = record.next_offset;
        }

        true
    }

    fn reserve_for(&mut self, new_record_size: usize) {
        if self.capacity != 0 && self.find_write_slot(new_record_size).is_some() {
            return;
        }

        let required = self.required_capacity(new_record_size);
        let mut new_capacity = self.capacity.max(64).max(required);
        if self.capacity != 0 && new_capacity == self.capacity {
            new_capacity = self
                .capacity
                .checked_mul(2)
                .expect("AnyQueue capacity overflow");
        }
        while new_capacity < required {
            new_capacity = new_capacity
                .checked_mul(2)
                .expect("AnyQueue capacity overflow");
        }

        self.rebuild(new_capacity);
    }

    fn required_capacity(&self, new_record_size: usize) -> usize {
        let mut required = new_record_size;
        let mut offset = self.head;
        for _ in 0..self.len {
            let record = unsafe { self.parse_record(offset) };
            required = required
                .checked_add(record.size)
                .expect("AnyQueue capacity overflow");
            offset = self.advance_offset(record.next_offset);
        }
        required
    }

    fn rebuild(&mut self, new_capacity: usize) {
        let new_ptr = Self::alloc_buffer(new_capacity);
        let mut old_offset = self.head;
        let mut new_offset = 0;

        for _ in 0..self.len {
            let record = unsafe { self.parse_record(old_offset) };
            unsafe {
                ptr::copy_nonoverlapping(
                    self.ptr.as_ptr().add(old_offset),
                    new_ptr.as_ptr().add(new_offset),
                    record.size,
                );
            }
            new_offset += record.size;
            old_offset = self.advance_offset(record.next_offset);
        }

        self.free_buffer();
        self.ptr = new_ptr;
        self.capacity = new_capacity;
        self.head = 0;
        self.tail = new_offset;
        self.wrap_at = None;
    }

    fn find_write_slot(&self, size: usize) -> Option<WriteSlot> {
        if self.capacity == 0 {
            return None;
        }
        if self.len == 0 {
            return (size <= self.capacity).then_some(WriteSlot {
                offset: 0,
                next_offset: size,
                wraps: false,
            });
        }

        if self.wrap_at.is_some() {
            return (self.tail.checked_add(size)? <= self.head).then_some(WriteSlot {
                offset: self.tail,
                next_offset: self.tail + size,
                wraps: false,
            });
        }

        if self.tail.checked_add(size)? <= self.capacity {
            return Some(WriteSlot {
                offset: self.tail,
                next_offset: self.tail + size,
                wraps: false,
            });
        }

        (size <= self.head).then_some(WriteSlot {
            offset: 0,
            next_offset: size,
            wraps: true,
        })
    }

    unsafe fn parse_record(&self, offset: usize) -> Record {
        let header =
            unsafe { ptr::read_unaligned(self.ptr.as_ptr().add(offset).cast::<RecordHeader>()) };
        let payload_offset = offset + Self::header_size();
        let size = Self::record_size(header.metadata);
        Record {
            metadata: header.metadata,
            payload_offset,
            next_offset: offset + size,
            size,
        }
    }

    fn advance_offset(&self, next_offset: usize) -> usize {
        if self.wrap_at == Some(next_offset) {
            0
        } else {
            next_offset
        }
    }

    fn header_size() -> usize {
        mem::size_of::<RecordHeader>()
    }

    fn record_size(metadata: &'static dyn AnyMetadata) -> usize {
        Self::header_size()
            .checked_add(metadata.layout().size())
            .expect("AnyQueue record size overflow")
    }

    fn alloc_buffer(capacity: usize) -> NonNull<u8> {
        let layout = Layout::from_size_align(capacity, 1).expect("invalid AnyQueue layout");
        unsafe {
            NonNull::new(alloc::alloc(layout)).unwrap_or_else(|| alloc::handle_alloc_error(layout))
        }
    }

    fn free_buffer(&mut self) {
        if self.capacity != 0 {
            unsafe {
                alloc::dealloc(
                    self.ptr.as_ptr(),
                    Layout::from_size_align_unchecked(self.capacity, 1),
                );
            }
            self.ptr = NonNull::dangling();
            self.capacity = 0;
        }
    }
}

impl Default for AnyQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for AnyQueue {
    fn drop(&mut self) {
        let mut out = AnyBox::new();
        while self.pop_front(&mut out) {}
        self.free_buffer();
    }
}

#[cfg(test)]
mod tests {
    use super::{AnyBox, AnyQueue};
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    #[test]
    fn mixed_types_are_fifo_and_downcastable() {
        let mut queue = AnyQueue::new();
        let mut out = AnyBox::new();
        queue.push_back(7u32);
        queue.push_back(String::from("alpha"));
        queue.push_back(11usize);

        assert!(queue.pop_front(&mut out));
        assert_eq!(out.downcast_ref::<u32>(), Some(&7));
        assert!(queue.pop_front(&mut out));
        assert_eq!(
            out.downcast_ref::<String>().map(String::as_str),
            Some("alpha")
        );
        assert!(queue.pop_front(&mut out));
        assert_eq!(out.downcast_ref::<usize>(), Some(&11));
        assert!(!queue.pop_front(&mut out));
    }

    #[repr(align(32))]
    struct HighAlign(u8);

    #[test]
    fn packed_unaligned_records_are_restored_aligned() {
        let mut queue = AnyQueue::new();
        let mut out = AnyBox::new();
        queue.push_back(1u8);
        queue.push_back(HighAlign(9));

        assert!(queue.pop_front(&mut out));
        assert_eq!(out.downcast_ref::<u8>(), Some(&1));
        assert!(queue.pop_front(&mut out));
        assert_eq!(out.downcast_ref::<HighAlign>().map(|v| v.0), Some(9));
    }

    #[test]
    fn wraps_and_expands_while_preserving_order() {
        let mut queue = AnyQueue::new();
        let mut out = AnyBox::new();

        for value in 0usize..20 {
            queue.push_back(value);
        }
        for expected in 0usize..8 {
            assert!(queue.pop_front(&mut out));
            assert_eq!(out.downcast_ref::<usize>(), Some(&expected));
        }
        for value in 20usize..60 {
            queue.push_back(value);
        }

        for expected in 8usize..60 {
            assert!(queue.pop_front(&mut out));
            assert_eq!(out.downcast_ref::<usize>(), Some(&expected));
        }
        assert!(queue.is_empty());
    }

    #[repr(align(64))]
    struct AlignedMarker;

    #[test]
    fn supports_zero_sized_values_with_high_alignment() {
        let mut queue = AnyQueue::new();
        let mut out = AnyBox::new();
        queue.push_back(AlignedMarker);

        assert!(queue.pop_front(&mut out));
        assert!(out.downcast_ref::<AlignedMarker>().is_some());
        assert!(out.as_any_mut().is_some());
    }

    struct DropCounter {
        drops: Arc<AtomicUsize>,
    }

    impl Drop for DropCounter {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn replacing_and_dropping_values_drops_each_once() {
        let drops = Arc::new(AtomicUsize::new(0));
        let mut out = AnyBox::new();
        let mut queue = AnyQueue::new();
        queue.push_back(DropCounter {
            drops: drops.clone(),
        });
        queue.push_back(DropCounter {
            drops: drops.clone(),
        });

        assert!(queue.pop_front(&mut out));
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        assert!(queue.pop_front(&mut out));
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        drop(out);
        assert_eq!(drops.load(Ordering::SeqCst), 2);

        let mut queue = AnyQueue::new();
        queue.push_back(DropCounter {
            drops: drops.clone(),
        });
        queue.push_back(DropCounter {
            drops: drops.clone(),
        });
        drop(queue);
        assert_eq!(drops.load(Ordering::SeqCst), 4);
    }
}
