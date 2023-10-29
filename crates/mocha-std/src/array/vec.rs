use std::{fmt, mem::MaybeUninit, ops, ptr, slice};

pub struct ArrayVec<T, const CAPACITY: usize> {
    buf: [MaybeUninit<T>; CAPACITY],
    init: usize,
}

impl<T, const CAPACITY: usize> ArrayVec<T, CAPACITY> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            buf: unsafe { MaybeUninit::uninit().assume_init() },
            init: 0,
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self.buf.as_ptr().cast()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.buf.as_mut_ptr().cast()
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.init
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.init == 0
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        CAPACITY
    }

    #[inline]
    pub fn push(&mut self, value: T) -> Result<(), T> {
        if let Some(cell) = self.buf.get_mut(self.init) {
            cell.write(value);
            self.init += 1;

            Ok(())
        } else {
            Err(value)
        }
    }

    #[inline]
    pub fn extend_from_slice(&mut self, other: &[T]) -> Result<(), ()>
    where
        T: Clone,
    {
        if self.len() + other.len() <= self.capacity() {
            for value in other.iter() {
                let _ = self.push(value.clone());
            }

            Ok(())
        } else {
            Err(())
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        let elements: *mut [T] = self.as_mut_slice();

        unsafe {
            self.init = 0;
            ptr::drop_in_place(elements);
        }
    }
}

impl<T, const CAPACITY: usize> AsRef<ArrayVec<T, CAPACITY>> for ArrayVec<T, CAPACITY> {
    fn as_ref(&self) -> &ArrayVec<T, CAPACITY> {
        self
    }
}

impl<T, const CAPACITY: usize> AsMut<ArrayVec<T, CAPACITY>> for ArrayVec<T, CAPACITY> {
    fn as_mut(&mut self) -> &mut ArrayVec<T, CAPACITY> {
        self
    }
}

impl<T, const CAPACITY: usize> AsRef<[T]> for ArrayVec<T, CAPACITY> {
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T, const CAPACITY: usize> AsMut<[T]> for ArrayVec<T, CAPACITY> {
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T, const CAPACITY: usize> fmt::Debug for ArrayVec<T, CAPACITY>
where
    T: fmt::Debug,
{
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_slice(), fmt)
    }
}

impl<T, const CAPACITY: usize> ops::Deref for ArrayVec<T, CAPACITY> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.as_ptr(), self.init) }
    }
}

impl<T, const CAPACITY: usize> ops::DerefMut for ArrayVec<T, CAPACITY> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.init) }
    }
}
