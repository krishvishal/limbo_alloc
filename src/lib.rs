use std::{
    cell::Cell,
    mem::transmute,
    ops::{Deref, DerefMut},
    pin::Pin,
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
        // CURRENT_ALLOCATOR.set();
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
