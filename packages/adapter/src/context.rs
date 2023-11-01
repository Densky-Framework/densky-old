use std::{ffi::c_void, fmt::Debug};

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct CloudContextRaw(*mut c_void);

pub trait CloudContext: Default + Debug {
    fn to_raw(&self) -> CloudContextRaw {
        let ptr = Box::leak(Box::new(self));
        let ptr = ptr as *mut &Self;
        CloudContextRaw(ptr as _)
    }
}

impl CloudContextRaw {
    #[inline(always)]
    #[must_use]
    pub const fn null() -> CloudContextRaw {
        CloudContextRaw(std::ptr::null_mut())
    }

    #[inline]
    #[must_use]
    pub fn to_rusty<T>(&self) -> &'static T
    where
        T: CloudContext,
    {
        let ptr = self.0 as *mut T;
        unsafe { ptr.as_ref().unwrap() }
    }

    #[inline]
    #[must_use]
    pub fn into_rusty<T>(&self) -> Box<T>
    where
        T: CloudContext,
    {
        let ptr = self.0 as *mut T;
        unsafe { Box::from_raw(ptr) }
    }
}
