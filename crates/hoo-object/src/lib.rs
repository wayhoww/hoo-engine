use std::{any::Any, cell::RefCell, hash::Hash, ops::Deref, os::raw::c_void, rc::Rc};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId {
    id: usize,
}

impl ObjectId {
    pub fn to_ptr(&self) -> *const c_void {
        self.id as *const c_void
    }

    pub fn from_ptr(ptr: *const c_void) -> Self {
        Self { id: ptr as usize }
    }
}

#[derive(Debug)]
pub struct RcObject<T: 'static + Sized + Any> {
    // id: ObjectId,
    inner: Rc<RefCell<T>>,
}

impl<T: 'static + Sized + Any> Clone for RcObject<T>{
    fn clone(&self) -> Self {
        Self {
            // id: self.id,
            inner: self.inner.clone(),
        }
    }
}

#[derive(Debug)]
pub struct RcTrait<T: 'static + ?Sized + Any> {
    // id: ObjectId,
    inner: Rc<RefCell<T>>,
    typed: Rc<dyn Any>,
}

// 不清楚为啥不能 derive
impl Clone for RcTrait<dyn Any> {
    fn clone(&self) -> Self {
        Self {
            // id: self.id,
            inner: self.inner.clone(),
            typed: self.typed.clone(),
        }
    }
}

#[macro_export]
macro_rules! into_trait {
    ($obj: expr) => {{
        let inner_clone = $obj.inner.clone();
        RcTrait::new_from_object($obj, inner_clone)
    }};
}

pub type RcAny = RcTrait<dyn Any>;

impl<T> RcObject<T> {
    pub fn new(inner: T) -> Self {
        Self {
            // id: ObjectId::new(),
            inner: Rc::new(RefCell::new(inner)),
        }
    }

    pub fn into_any(self) -> RcAny {
        into_trait!(self)
    }

    pub fn id(&self) -> ObjectId {
        ObjectId { id: Rc::as_ptr(&self.inner) as usize }
    }
}

impl<T: ?Sized> RcTrait<T> {
    pub fn new_from_object<U>(origin: RcObject<U>, origin_inner: Rc<RefCell<T>>) -> Self {
        let ptr1 = Rc::as_ptr(&origin.inner) as usize;
        let ptr2 = Rc::as_ptr(&origin_inner) as *const c_void as usize;
        assert_eq!(ptr1, ptr2);
        RcTrait {
            typed: origin.inner,
            inner: origin_inner,
        }
    }

    pub fn try_downcast<U: 'static>(self) -> Result<RcObject<U>, Self> {
        let down_casted: Result<Rc<RefCell<U>>, _> = self.typed.downcast();
        match down_casted {
            Ok(casted) => Ok(RcObject { inner: casted }),
            Err(typed) => Err(RcTrait {
                inner: self.inner,
                typed,
            }),
        }
    }

    pub fn id(&self) -> ObjectId {
        ObjectId { id: Rc::as_ptr(&self.inner) as *const c_void as usize }
    }
}

impl<T, U> TryFrom<RcTrait<U>> for RcObject<T> {
    type Error = RcTrait<U>;
    fn try_from(obj: RcTrait<U>) -> Result<Self, Self::Error> {
        obj.try_downcast()
    }
}

// for RcObject
impl<T> Hash for RcObject<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let ptr = Rc::as_ptr(&self.inner) as usize;
        state.write_usize(ptr);
    }
}

impl<T> PartialEq for RcObject<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<T> Eq for RcObject<T> {}

// for RcTrait
impl<T: ?Sized> Hash for RcTrait<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let ptr = Rc::as_ptr(&self.inner) as *const c_void as usize;
        state.write_usize(ptr);
    }
}

impl<T: ?Sized> PartialEq for RcTrait<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<T: ?Sized> Eq for RcTrait<T> {}

impl<T> Deref for RcObject<T> {
    type Target = RefCell<T>;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<T: ?Sized> Deref for RcTrait<T> {
    type Target = RefCell<T>;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}
