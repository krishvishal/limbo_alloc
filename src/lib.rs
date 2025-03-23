use std::{
    borrow::{Borrow, BorrowMut},
    fmt,
};
use std::{
    cell::Cell,
    mem::transmute,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use allocator_api2::alloc::{AllocError, Allocator, Layout};
use allocator_api2::boxed;
use bumpalo::Bump;

thread_local! {
    static CURRENT_ALLOCATOR: Cell<Option<&'static WrapAllocator>> = const { Cell::new(None) };
}

pub struct WrapAllocator {
    bump: Bump,
}

#[derive(Clone, Copy)]
pub struct LimboAllocator {
    allocator: Option<&'static WrapAllocator>,
}

pub struct AllocatorGuard {}

impl Drop for AllocatorGuard {
    fn drop(&mut self) {
        CURRENT_ALLOCATOR.set(None);
    }
}

impl WrapAllocator {
    pub fn new() -> Self {
        Self { bump: Bump::new() }
    }

    pub unsafe fn guard(&self) -> AllocatorGuard {
        let static_ref = unsafe { transmute::<&WrapAllocator, &'static WrapAllocator>(self) };
        CURRENT_ALLOCATOR.set(Some(static_ref));
        AllocatorGuard {}
    }
}

impl Default for WrapAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl LimboAllocator {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for LimboAllocator {
    fn default() -> Self {
        Self {
            allocator: CURRENT_ALLOCATOR.get(),
        }
    }
}

unsafe impl Allocator for LimboAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        if let Some(alloc) = self.allocator {
            let ptr = alloc.bump.alloc_layout(layout);

            let slice_ptr = NonNull::slice_from_raw_parts(
                NonNull::new(ptr.as_ptr()).ok_or(AllocError)?,
                layout.size(),
            );
            Ok(slice_ptr)
        } else {
            Err(AllocError)
        }
    }

    fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        match self.allocator {
            Some(allocator) => {
                let ptr = allocator.bump.alloc_layout(layout);
                unsafe {
                    std::ptr::write_bytes(ptr.as_ptr(), 0, layout.size());
                }
                let slice_ptr = NonNull::slice_from_raw_parts(
                    NonNull::new(ptr.as_ptr()).ok_or(AllocError)?,
                    layout.size(),
                );
                Ok(slice_ptr)
            }
            None => Err(AllocError),
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        // No need to deallocate in bump allocation
        // Memory is freed all at once when bump is dropped
    }

    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        match self.allocator {
            Some(allocator) => {
                let new_ptr = allocator.bump.alloc_layout(new_layout);
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        ptr.as_ptr(),
                        new_ptr.as_ptr(),
                        old_layout.size(),
                    );
                }

                let slice_ptr = NonNull::slice_from_raw_parts(
                    NonNull::new(new_ptr.as_ptr()).ok_or(AllocError)?,
                    new_layout.size(),
                );
                Ok(slice_ptr)
            }
            None => Err(AllocError),
        }
    }

    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        match self.allocator {
            Some(allocator) => {
                let new_ptr = allocator.bump.alloc_layout(new_layout);

                unsafe {
                    std::ptr::copy_nonoverlapping(
                        ptr.as_ptr(),
                        new_ptr.as_ptr(),
                        old_layout.size(),
                    );
                    std::ptr::write_bytes(
                        new_ptr.as_ptr().add(old_layout.size()),
                        0,
                        new_layout.size() - old_layout.size(),
                    );
                }

                let slice_ptr = NonNull::slice_from_raw_parts(
                    NonNull::new(new_ptr.as_ptr()).ok_or(AllocError)?,
                    new_layout.size(),
                );
                Ok(slice_ptr)
            }
            None => Err(AllocError),
        }
    }

    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        if let Some(alloc) = self.allocator {
            let new_ptr = alloc.bump.alloc_layout(new_layout);

            unsafe {
                std::ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_ptr(), new_layout.size());
            }

            let slice_ptr = NonNull::slice_from_raw_parts(
                NonNull::new(new_ptr.as_ptr()).ok_or(AllocError)?,
                new_layout.size(),
            );
            Ok(slice_ptr)
        } else {
            panic!()
        }
    }
}

impl Deref for WrapAllocator {
    type Target = Bump;

    fn deref(&self) -> &Bump {
        &self.bump
    }
}

impl DerefMut for WrapAllocator {
    fn deref_mut(&mut self) -> &mut Bump {
        &mut self.bump
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Box<T: ?Sized>(pub(crate) boxed::Box<T, LimboAllocator>);

impl<T> Box<T> {
    #[inline(always)]
    pub fn new(value: T) -> Self {
        Self::new_in(value, LimboAllocator::new())
    }

    #[inline(always)]
    pub fn new_in(value: T, alloc: LimboAllocator) -> Self {
        Self(boxed::Box::new_in(value, alloc))
    }

    pub fn unbox(self) -> T {
        boxed::Box::into_inner(self.0)
    }
}

impl<T: ?Sized> Box<T> {
    pub unsafe fn from_raw(raw: *mut T) -> Self {
        Self(boxed::Box::from_raw_in(raw, LimboAllocator::new()))
    }

    pub fn into_raw(b: Self) -> *mut T {
        boxed::Box::into_raw(b.0)
    }
}

impl<T: ?Sized> Deref for Box<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: ?Sized> DerefMut for Box<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> From<T> for Box<T> {
    #[inline(always)]
    fn from(v: T) -> Self {
        Box::new(v)
    }
}

impl<T: ?Sized> From<allocator_api2::boxed::Box<T, LimboAllocator>> for Box<T> {
    #[inline(always)]
    fn from(v: allocator_api2::boxed::Box<T, LimboAllocator>) -> Self {
        Box(v)
    }
}

impl<T: ?Sized> AsRef<T> for Box<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T: ?Sized> AsMut<T> for Box<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: ?Sized> Borrow<T> for Box<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

impl<T: ?Sized> BorrowMut<T> for Box<T> {
    fn borrow_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: ?Sized + Iterator> Iterator for Box<T> {
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        (**self).next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (**self).size_hint()
    }
}

impl<T: ?Sized + DoubleEndedIterator> DoubleEndedIterator for Box<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        (**self).next_back()
    }
}

impl<T: ?Sized + ExactSizeIterator> ExactSizeIterator for Box<T> {
    fn len(&self) -> usize {
        (**self).len()
    }
}

impl<T: ?Sized> fmt::Pointer for Box<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&(&**self as *const T), f)
    }
}

impl<T: ?Sized + Default> Default for Box<T> {
    fn default() -> Self {
        Box::new(T::default())
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Vec<T>(allocator_api2::vec::Vec<T, LimboAllocator>);

impl<T> Vec<T> {
    #[inline(always)]
    pub fn new() -> Self {
        Self::new_in(LimboAllocator::new())
    }

    #[inline(always)]
    pub fn new_in(alloc: LimboAllocator) -> Self {
        Self(allocator_api2::vec::Vec::new_in(alloc))
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_in(capacity, LimboAllocator::new())
    }

    pub fn with_capacity_in(capacity: usize, alloc: LimboAllocator) -> Self {
        Self(allocator_api2::vec::Vec::with_capacity_in(capacity, alloc))
    }

    pub fn into_boxed_slice(self) -> Box<[T]> {
        Box(self.0.into_boxed_slice())
    }

    pub fn leak<'a>(self) -> &'a mut [T]
    where
        T: 'a,
    {
        self.0.leak()
    }

    pub unsafe fn from_raw_parts(ptr: *mut T, length: usize, capacity: usize) -> Self {
        Self(allocator_api2::vec::Vec::from_raw_parts_in(
            ptr,
            length,
            capacity,
            LimboAllocator::new(),
        ))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[T] {
        self.0.as_slice()
    }

    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.0.as_mut_slice()
    }

    #[inline(always)]
    pub fn push(&mut self, value: T) {
        self.0.push(value)
    }

    #[inline(always)]
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional)
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    #[inline(always)]
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit()
    }

    #[inline(always)]
    pub fn as_ptr(&self) -> *const T {
        self.0.as_ptr()
    }

    #[inline(always)]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.0.as_mut_ptr()
    }
}

impl<T> Default for Vec<T> {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Deref for Vec<T> {
    type Target = allocator_api2::vec::Vec<T, LimboAllocator>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Vec<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> FromIterator<T> for Vec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let mut vec = Self::with_capacity(iter.size_hint().0);
        vec.extend(iter);
        vec
    }
}

impl<T> IntoIterator for Vec<T> {
    type Item = T;
    type IntoIter = allocator_api2::vec::IntoIter<T, LimboAllocator>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Vec<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Vec<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> Extend<T> for Vec<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.0.extend(iter)
    }
}

#[macro_export]
macro_rules! vec {
    () => {
        $crate::Vec::new()
    };
    ($elem:expr; $n:expr) => {
        $crate::Vec::from_iter(std::iter::repeat($elem).take($n))
    };
    ($($x:expr),+ $(,)?) => {
        $crate::Vec::from_iter([$($x),+])
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_vec_basic_operations() {
        let allocator = WrapAllocator::new();
        let _guard = unsafe { allocator.guard() };

        let mut vec = Vec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        assert_eq!(vec.len(), 3);
        assert_eq!(vec[0], 1);
        assert_eq!(vec[1], 2);
        assert_eq!(vec[2], 3);
    }
    #[test]
    fn test_box() {
        let allocator = WrapAllocator::new();
        let boxed = {
            let _guard = unsafe { allocator.guard() };
            Box::new(42)
        };
        assert_eq!(*boxed, 42);
    }

    #[test]
    fn test_vec_iteration() {
        let allocator = WrapAllocator::new();
        let _guard = unsafe { allocator.guard() };

        let mut vec = Vec::new();
        vec.extend(0..3);

        let sum: i32 = vec.clone().into_iter().sum();
        assert_eq!(sum, 3);

        let sum: i32 = vec.iter().sum();
        assert_eq!(sum, 3);

        let sum: i32 = vec.iter_mut().map(|x| *x).sum();
        assert_eq!(sum, 3);
    }

    struct TestParser<'a> {
        some_reference: &'a str,
        vector: Vec<String>,
        _allocator: &'a WrapAllocator,
        _guard: AllocatorGuard,
    }

    impl<'a> TestParser<'a> {
        fn new(reference: &'a str, allocator: &'a WrapAllocator, guard: AllocatorGuard) -> Self {
            let mut vector = Vec::new();

            vector.push(String::from("We are"));
            vector.push(String::from("So Back"));

            TestParser {
                some_reference: reference,
                vector,
                _allocator: &allocator,
                _guard: guard,
            }
        }

        fn add_item(&mut self, item: String) {
            self.vector.push(item);
        }
    }

    #[test]
    fn test_allocator_with_different_lifetimes() {
        {
            let text = String::from("test string");
            {
                let allocator: WrapAllocator = WrapAllocator::new();

                let guard = unsafe { allocator.guard() };

                let mut parser = TestParser::new(&text, &allocator, guard);
                parser.add_item(String::from("Additional item"));
            }
        }
    }
}
