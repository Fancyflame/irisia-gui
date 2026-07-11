use std::{
    alloc::{self, Layout},
    any::{Any, TypeId},
    marker::PhantomData,
    mem,
    ptr::{self, drop_in_place, NonNull},
};

pub struct AnyQueue {
    ptr: NonNull<u8>,
    capacity: usize,
    align: usize,
    head: usize,
    tail: usize,
    wrap_at: Option<usize>,
    len: usize,
}

#[derive(Clone, Copy)]
struct RecordHeader {
    info: &'static dyn AnyInfo,
}

#[derive(Clone, Copy)]
struct RecordPlacement {
    header_offset: usize,
    payload_offset: usize,
    next_offset: usize,
}

#[derive(Clone, Copy)]
struct ParsedRecord {
    header: RecordHeader,
    payload_offset: usize,
    next_offset: usize,
}

impl AnyQueue {
    /// 创建一个空队列，初始时不分配缓冲区。
    pub fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            capacity: 0,
            align: Self::header_layout().align(),
            head: 0,
            tail: 0,
            wrap_at: None,
            len: 0,
        }
    }

    /// 返回当前队列中的元素个数。
    pub fn len(&self) -> usize {
        self.len
    }

    /// 当队列为空时返回 `true`。
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// 向队尾追加一个值。
    ///
    /// 值会以内联形式写入原始环形缓冲区；如果当前缓冲区无法提供一段
    /// “完整且满足对齐”的连续空间，则会先扩容或重排。
    pub fn push_back<T: Any>(&mut self, value: T) {
        let info = get_any_info::<T>();
        // 先预留空间，保证后面的写入路径只需要一次确定位置并原地构造。
        self.reserve_for(info);

        unsafe {
            let placement = self.claim_write_slot(info);
            // 先写入类型元数据，再在 payload 位置原地写入具体对象。
            self.ptr
                .as_ptr()
                .add(placement.header_offset)
                .cast::<RecordHeader>()
                .write(RecordHeader { info });
            ptr::write(
                self.ptr.as_ptr().add(placement.payload_offset).cast::<T>(),
                value,
            );
        }

        self.len += 1;
    }

    /// 当队首元素的具体类型为 `T` 时，返回它的共享引用。
    ///
    /// 空队列和类型不匹配都会返回 `None`。
    pub fn peek_front<T: Any>(&self) -> Option<&T> {
        if self.len == 0 {
            return None;
        }

        unsafe {
            // 先从原始内存解析记录，再在类型匹配后做引用转换。
            let record = self.parse_record(self.head);
            (record.header.info.type_id() == TypeId::of::<T>())
                .then(|| &*self.ptr.as_ptr().add(record.payload_offset).cast::<T>())
        }
    }

    /// 销毁队首元素并推进队列。
    ///
    /// 当队列为空时返回 `false`。
    pub fn drop_front(&mut self) -> bool {
        if self.len == 0 {
            return false;
        }

        unsafe {
            let record = self.parse_record(self.head);
            // 先原地析构 payload，再修改环形缓冲区状态。
            record
                .header
                .info
                .drop_in_place(self.ptr.as_ptr().add(record.payload_offset).cast());

            self.len -= 1;

            if self.len == 0 {
                // 删除最后一个元素后，恢复到标准空队列状态。
                self.head = 0;
                self.tail = 0;
                self.wrap_at = None;
            } else if self.wrap_at == Some(record.next_offset) {
                // 跨过回绕标记后，逻辑上的下一个元素就在 0 位置。
                self.head = 0;
                self.wrap_at = None;
            } else {
                self.head = record.next_offset;
            }
        }

        true
    }

    /// 确保队列还能容纳一个由 `info` 描述的新元素。
    ///
    /// 这里可能发生首次分配、提升整体对齐后的重建，或者扩容后再重建。
    /// 当当前环形布局放不下新记录时，不尝试用同容量压实解决，直接换更大的缓冲区。
    fn reserve_for(&mut self, info: &'static dyn AnyInfo) {
        let required_align = self
            .align
            .max(info.align())
            .max(Self::header_layout().align());

        if self.capacity == 0 {
            // 还没有缓冲区时，直接分配能放下首个元素的最小容量。
            let capacity = self.next_capacity(0, required_align, info);
            unsafe {
                self.replace_empty_allocation(capacity, required_align);
            }
            return;
        }

        if self.align < required_align {
            // 底层 allocation 的对齐不够，只能整体重建到更高对齐的缓冲区。
            let capacity = self.next_capacity(self.capacity, required_align, info);
            self.rebuild_allocation(capacity, required_align);
            return;
        }

        if unsafe { self.find_write_slot(info).is_some() } {
            return;
        }

        // 当前环形布局放不下时，不判断同容量压实是否可行，直接扩容并重建。
        let capacity = self.next_capacity(
            self.capacity
                .checked_add(1)
                .expect("any queue capacity overflow"),
            self.align,
            info,
        );
        self.rebuild_allocation(capacity, self.align);
    }

    /// 找到一个足以容纳“现有所有元素 + 一个新元素”的下一档容量。
    ///
    /// 容量从 `min_capacity` 或 64 开始按 2 倍增长，直到能容纳压紧后的全部记录。
    fn next_capacity(
        &self,
        min_capacity: usize,
        target_align: usize,
        new_info: &'static dyn AnyInfo,
    ) -> usize {
        let required = self.required_capacity_after_rebuild(target_align, new_info);
        let mut capacity = min_capacity.max(64);

        while capacity < required {
            capacity = capacity
                .checked_mul(2)
                .expect("any queue capacity overflow");
        }

        capacity
    }

    /// 计算顺序重建后容纳“现有元素 + 新元素”至少需要多少字节。
    fn required_capacity_after_rebuild(
        &self,
        target_align: usize,
        new_info: &'static dyn AnyInfo,
    ) -> usize {
        let mut cursor = 0usize;
        let mut offset = self.head;

        // 新 allocation 的基址至少满足 target_align，因此这里可以用纯 offset 对齐估算。
        for _ in 0..self.len {
            let record = unsafe { self.parse_record(offset) };
            cursor = Self::skip_record(cursor, record.header.info, target_align);
            offset = self.advance_offset(record.next_offset);
        }

        Self::skip_record(cursor, new_info, target_align)
    }

    /// 用纯布局计算跳过一条记录后，下一条记录应从哪里开始。
    ///
    /// 这个函数不读写内存，只用于新缓冲区的容量估算。
    fn skip_record(cursor: usize, info: &'static dyn AnyInfo, queue_align: usize) -> usize {
        let header_layout = Self::header_layout();
        debug_assert!(queue_align >= header_layout.align());
        debug_assert!(queue_align >= info.align());
        let header_start = Self::align_up(cursor, header_layout.align());
        let header_end = header_start
            .checked_add(header_layout.size())
            .expect("any queue capacity overflow");
        let payload_start = Self::align_up(header_end, info.align());
        let payload_end = payload_start
            .checked_add(info.size())
            .expect("any queue capacity overflow");

        Self::align_up(payload_end, header_layout.align())
    }

    /// 把队列重建到一个新的缓冲区中。
    ///
    /// 旧队列中的记录会按逻辑 FIFO 顺序重新解析并紧凑写入新缓冲区，
    /// 整个过程只搬运字节，不重新构造也不重复析构对象。
    fn rebuild_allocation(&mut self, new_capacity: usize, new_align: usize) {
        unsafe {
            if self.len == 0 {
                self.replace_empty_allocation(new_capacity, new_align);
                return;
            }

            let new_ptr = Self::alloc_buffer(new_capacity, new_align);
            let mut old_offset = self.head;
            let mut new_cursor = 0usize;

            for _ in 0..self.len {
                let record = self.parse_record(old_offset);
                // 基于新 allocation 重新计算布局，确保 payload 在新基址下仍然对齐。
                let placement = Self::record_placement(
                    new_ptr.as_ptr(),
                    new_cursor,
                    record.header.info,
                    new_capacity,
                )
                .expect("record should fit in rebuilt any queue");

                new_ptr
                    .as_ptr()
                    .add(placement.header_offset)
                    .cast::<RecordHeader>()
                    .write(record.header);

                if record.header.info.size() != 0 {
                    ptr::copy_nonoverlapping(
                        self.ptr.as_ptr().add(record.payload_offset),
                        new_ptr.as_ptr().add(placement.payload_offset),
                        record.header.info.size(),
                    );
                }

                new_cursor = placement.next_offset;
                old_offset = self.advance_offset(record.next_offset);
            }

            // 所有记录完成搬运后，再整体替换旧缓冲区。
            self.free_buffer();
            self.ptr = new_ptr;
            self.capacity = new_capacity;
            self.align = new_align;
            self.head = 0;
            self.tail = new_cursor;
            self.wrap_at = None;
        }
    }

    /// 为新元素提交最终写入位置，并同步更新环形缓冲区游标。
    ///
    /// 当写入位置回到 `0` 时，这里也会建立回绕标记。
    unsafe fn claim_write_slot(&mut self, info: &'static dyn AnyInfo) -> RecordPlacement {
        let slot = unsafe {
            self.find_write_slot(info)
                .expect("buffer should be reserved before write")
        };

        if self.len == 0 {
            // 第一个元素同时决定 head 和 tail。
            self.head = slot.header_offset;
            self.tail = slot.next_offset;
            self.wrap_at = None;
        } else if self.wrap_at.is_none() && slot.header_offset == 0 && self.tail != 0 {
            // 记下第一段的结束位置，读取时到这里就要跳回 0。
            self.wrap_at = Some(self.tail);
            self.tail = slot.next_offset;
        } else {
            self.tail = slot.next_offset;
        }

        slot
    }

    /// 在当前环形布局中，尝试为一个新元素寻找可写入位置。
    ///
    /// 优先尝试尾部连续段；如果当前还没有回绕，再尝试前缀段 `[0, head)`。
    unsafe fn find_write_slot(&self, info: &'static dyn AnyInfo) -> Option<RecordPlacement> {
        if self.capacity == 0 {
            return None;
        }

        if self.len == 0 {
            // 空队列总是从缓冲区起点开始写。
            return unsafe { Self::record_placement(self.ptr.as_ptr(), 0, info, self.capacity) };
        }

        if let Some(slot) = unsafe {
            Self::record_placement(self.ptr.as_ptr(), self.tail, info, self.write_segment_end())
        } {
            return Some(slot);
        }

        if self.wrap_at.is_none() {
            unsafe { Self::record_placement(self.ptr.as_ptr(), 0, info, self.head) }
        } else {
            None
        }
    }

    /// 从 `offset` 开始解析一条完整记录。
    ///
    /// 先读出 `RecordHeader`，再通过其中的 `AnyInfo` 恢复 payload 的布局，
    /// 最后算出下一条记录的起始位置。
    unsafe fn parse_record(&self, offset: usize) -> ParsedRecord {
        let segment_end = self.read_segment_end(offset);
        let header_layout = Self::header_layout();
        let header = unsafe { self.ptr.as_ptr().add(offset).cast::<RecordHeader>().read() };
        let payload_layout =
            unsafe { Layout::from_size_align_unchecked(header.info.size(), header.info.align()) };
        let (_, payload_offset, _payload_end, next_offset) = unsafe {
            Self::layout_record(
                self.ptr.as_ptr(),
                offset,
                payload_layout,
                segment_end,
                header_layout,
            )
        }
        .expect("corrupted any queue record layout");

        ParsedRecord {
            header,
            payload_offset,
            next_offset,
        }
    }

    /// 计算一条记录在某段连续空间中的写入位置。
    ///
    /// 返回值包含 header 起点、payload 起点以及下一条记录的对齐起点。
    unsafe fn record_placement(
        base_ptr: *const u8,
        cursor: usize,
        info: &'static dyn AnyInfo,
        segment_end: usize,
    ) -> Option<RecordPlacement> {
        let payload_layout =
            unsafe { Layout::from_size_align_unchecked(info.size(), info.align()) };
        let header_layout = Self::header_layout();
        let (header_offset, payload_offset, _, next_offset) = unsafe {
            Self::layout_record(base_ptr, cursor, payload_layout, segment_end, header_layout)
        }?;

        Some(RecordPlacement {
            header_offset,
            payload_offset,
            next_offset,
        })
    }

    /// 在 `[cursor, segment_end)` 中布置 `RecordHeader + payload`。
    ///
    /// 真实写入和记录解析都会共用这个布局原语，保证两边的计算方式一致。
    unsafe fn layout_record(
        base_ptr: *const u8,
        cursor: usize,
        payload_layout: Layout,
        segment_end: usize,
        header_layout: Layout,
    ) -> Option<(usize, usize, usize, usize)> {
        let (header_offset, after_header) =
            unsafe { Self::place_layout(base_ptr, cursor, header_layout, segment_end) }?;
        let (payload_offset, payload_end) =
            unsafe { Self::place_layout(base_ptr, after_header, payload_layout, segment_end) }?;
        let next_offset = unsafe {
            Self::next_record_offset(base_ptr, payload_end, header_layout.align(), segment_end)
        };
        Some((header_offset, payload_offset, payload_end, next_offset))
    }

    /// 在一段连续空间里，从 `cursor` 起放置一个满足 `layout` 的对象。
    ///
    /// 成功时返回该对象的对齐起点和结束位置。
    unsafe fn place_layout(
        base_ptr: *const u8,
        cursor: usize,
        layout: Layout,
        segment_end: usize,
    ) -> Option<(usize, usize)> {
        if cursor > segment_end {
            return None;
        }

        // 对齐必须相对于真实分配基址计算，不能只看纯 offset。
        let start = unsafe { Self::align_cursor(base_ptr, cursor, layout.align()) }?;
        let end = start.checked_add(layout.size())?;
        (end <= segment_end).then_some((start, end))
    }

    /// 从 payload 末尾推进到下一条 header 可能出现的位置。
    ///
    /// 如果当前连续段里已经找不到可对齐的 header 起点，就直接返回段尾，
    /// 让上层走回绕逻辑。
    unsafe fn next_record_offset(
        base_ptr: *const u8,
        payload_end: usize,
        header_align: usize,
        segment_end: usize,
    ) -> usize {
        match unsafe { Self::align_cursor(base_ptr, payload_end, header_align) } {
            Some(next) if next <= segment_end => next,
            _ => segment_end,
        }
    }

    /// 基于 `base_ptr` 这个真实分配基址，把 `cursor` 对齐到 `align`。
    ///
    /// 这里使用 `align_offset`，避免错误地假设 offset `0` 对所有类型都天然对齐。
    unsafe fn align_cursor(base_ptr: *const u8, cursor: usize, align: usize) -> Option<usize> {
        let offset = unsafe { base_ptr.add(cursor) }.align_offset(align);
        (offset != usize::MAX)
            .then(|| cursor.checked_add(offset))
            .flatten()
    }

    /// 返回 `offset` 所在那一段连续记录区域的结束边界。
    fn read_segment_end(&self, offset: usize) -> usize {
        match self.wrap_at {
            Some(_) if offset < self.head => self.head,
            Some(wrap_at) => wrap_at,
            None => self.capacity,
        }
    }

    /// 返回当前 tail 写入时可用连续空间的结束边界。
    fn write_segment_end(&self) -> usize {
        if self.wrap_at.is_some() {
            self.head
        } else {
            self.capacity
        }
    }

    /// 把解析出的物理偏移转换成逻辑上的下一个读取位置。
    ///
    /// 当命中回绕标记时，说明下一条记录需要从 `0` 继续读。
    fn advance_offset(&self, next_offset: usize) -> usize {
        if self.wrap_at == Some(next_offset) {
            0
        } else {
            next_offset
        }
    }

    /// 为一个空队列替换底层存储。
    ///
    /// 这用于首次分配，或在没有存活元素时切换容量/对齐。
    unsafe fn replace_empty_allocation(&mut self, new_capacity: usize, new_align: usize) {
        unsafe {
            self.free_buffer();
        }
        self.ptr = if new_capacity == 0 {
            NonNull::dangling()
        } else {
            Self::alloc_buffer(new_capacity, new_align)
        };
        self.capacity = new_capacity;
        self.align = new_align;
        self.head = 0;
        self.tail = 0;
        self.wrap_at = None;
    }

    /// 释放队列当前持有的原始缓冲区。
    unsafe fn free_buffer(&mut self) {
        if self.capacity != 0 {
            unsafe {
                alloc::dealloc(
                    self.ptr.as_ptr(),
                    Layout::from_size_align_unchecked(self.capacity, self.align),
                );
            }
        }
    }

    /// 按给定容量和对齐分配一块新的原始缓冲区。
    fn alloc_buffer(capacity: usize, align: usize) -> NonNull<u8> {
        debug_assert!(capacity != 0);
        let layout = Layout::from_size_align(capacity, align).expect("invalid any queue layout");
        unsafe {
            let ptr = alloc::alloc(layout);
            NonNull::new(ptr).unwrap_or_else(|| alloc::handle_alloc_error(layout))
        }
    }

    /// 将 `value` 向上取整到 `align` 的整数倍。
    fn align_up(value: usize, align: usize) -> usize {
        debug_assert!(align.is_power_of_two());
        value
            .checked_add(align - 1)
            .expect("any queue capacity overflow")
            & !(align - 1)
    }

    /// 返回记录头 `RecordHeader` 的固定布局。
    fn header_layout() -> Layout {
        Layout::new::<RecordHeader>()
    }
}

impl Default for AnyQueue {
    /// 创建一个空队列。
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for AnyQueue {
    /// 析构队列中的所有元素，然后释放底层缓冲区。
    fn drop(&mut self) {
        // 复用正常的出队析构路径，保证每个元素只会被析构一次。
        while self.drop_front() {}

        unsafe {
            self.free_buffer();
        }
    }
}

/// 返回 `T` 对应的类型元数据单例。
fn get_any_info<T: Any>() -> &'static dyn AnyInfo {
    struct Metadata<T>(std::marker::PhantomData<T>);

    impl<T: Any> AnyInfo for Metadata<T> {
        /// 返回 `T` 的 `TypeId`，用于运行时类型匹配。
        fn type_id(&self) -> TypeId {
            TypeId::of::<T>()
        }

        /// 对已经存入队列的 `T` payload 执行原地析构。
        unsafe fn drop_in_place(&self, to_drop: *mut ()) {
            unsafe {
                drop_in_place(to_drop.cast::<T>());
            }
        }

        /// 返回 `T` 的对齐要求。
        fn align(&self) -> usize {
            mem::align_of::<T>()
        }

        /// 返回 `T` 的字节大小。
        fn size(&self) -> usize {
            mem::size_of::<T>()
        }
    }

    &Metadata::<T>(PhantomData)
}

/// 一个具体 `Any` 类型在运行时对应的元数据接口。
trait AnyInfo {
    /// 返回该具体类型的 `TypeId`。
    fn type_id(&self) -> TypeId;
    /// 原地析构 `ptr` 所指向的一个 payload。
    unsafe fn drop_in_place(&self, ptr: *mut ());
    /// 返回 payload 的字节大小。
    fn size(&self) -> usize;
    /// 返回 payload 的字节对齐要求。
    fn align(&self) -> usize;
}

#[cfg(test)]
mod tests {
    use super::AnyQueue;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    #[test]
    fn fifo_for_mixed_types() {
        let mut queue = AnyQueue::new();
        queue.push_back(7u32);
        queue.push_back(String::from("alpha"));
        queue.push_back(11usize);

        assert_eq!(queue.peek_front::<u32>(), Some(&7));
        assert!(queue.drop_front());
        assert_eq!(
            queue.peek_front::<String>().map(String::as_str),
            Some("alpha")
        );
        assert!(queue.drop_front());
        assert_eq!(queue.peek_front::<usize>(), Some(&11));
        assert!(queue.drop_front());
        assert!(queue.peek_front::<usize>().is_none());
        assert!(!queue.drop_front());
    }

    #[test]
    fn type_mismatch_returns_none_without_consuming() {
        let mut queue = AnyQueue::new();
        queue.push_back(123u32);

        assert!(queue.peek_front::<String>().is_none());
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.peek_front::<u32>(), Some(&123));
    }

    #[repr(align(8))]
    struct LowAlign(&'static str);

    #[repr(align(32))]
    struct HighAlign(&'static str);

    #[test]
    fn upgrades_alignment_and_preserves_order() {
        let mut queue = AnyQueue::new();
        queue.push_back(LowAlign("low"));
        queue.push_back(HighAlign("high"));

        assert!(queue.align >= 32);
        assert_eq!(queue.peek_front::<LowAlign>().map(|v| v.0), Some("low"));
        assert!(queue.drop_front());
        assert_eq!(queue.peek_front::<HighAlign>().map(|v| v.0), Some("high"));
    }

    #[test]
    fn keeps_payload_aligned_after_wrap_and_upgrade() {
        let mut queue = AnyQueue::new();
        queue.push_back(LowAlign("a"));
        queue.push_back(LowAlign("b"));
        assert!(queue.drop_front());
        queue.push_back(HighAlign("c"));

        assert_eq!(queue.peek_front::<LowAlign>().map(|v| v.0), Some("b"));
        assert!(queue.drop_front());

        unsafe {
            let record = queue.parse_record(queue.head);
            let payload_ptr = queue.ptr.as_ptr().add(record.payload_offset);
            assert_eq!(payload_ptr.addr() % record.header.info.align(), 0);
        }

        assert_eq!(queue.peek_front::<HighAlign>().map(|v| v.0), Some("c"));
    }

    #[test]
    fn supports_multiple_pushes_in_wrapped_tail_segment() {
        let mut queue = AnyQueue::new();
        queue.push_back(1u64);
        queue.push_back(2u64);
        queue.push_back(3u64);
        queue.push_back(4u64);

        assert!(queue.drop_front());
        assert!(queue.drop_front());

        queue.push_back(5u64);
        queue.push_back(6u64);
        assert!(queue.wrap_at.is_some());
        queue.push_back(7u64);

        for expected in 3u64..=7 {
            assert_eq!(queue.peek_front::<u64>(), Some(&expected));
            assert!(queue.drop_front());
        }

        assert!(queue.is_empty());
    }

    #[test]
    fn supports_zero_sized_types() {
        #[derive(Debug, PartialEq)]
        struct Marker;

        let mut queue = AnyQueue::new();
        queue.push_back(Marker);

        assert!(queue.peek_front::<Marker>().is_some());
        assert!(queue.drop_front());
        assert!(queue.is_empty());
    }

    struct DropCounter {
        id: usize,
        drops: Arc<AtomicUsize>,
    }

    impl Drop for DropCounter {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn drops_each_value_exactly_once() {
        let drops = Arc::new(AtomicUsize::new(0));
        let mut queue = AnyQueue::new();

        queue.push_back(DropCounter {
            id: 1,
            drops: Arc::clone(&drops),
        });
        queue.push_back(DropCounter {
            id: 2,
            drops: Arc::clone(&drops),
        });

        assert_eq!(queue.peek_front::<DropCounter>().map(|v| v.id), Some(1));
        assert!(queue.drop_front());
        assert_eq!(drops.load(Ordering::SeqCst), 1);

        drop(queue);
        assert_eq!(drops.load(Ordering::SeqCst), 2);
    }
}
