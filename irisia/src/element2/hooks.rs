use std::{
    any::Any,
    cell::RefCell,
    ops::{Deref, DerefMut},
};

type CreateVec<'a> = &'a RefCell<Vec<Option<Box<dyn Any>>>>;

pub struct HookStorage {
    hooks: Vec<Box<dyn Any>>,
}

impl HookStorage {
    pub fn new<F, R>(logic: F) -> (Self, R)
    where
        F: FnOnce(UseHook) -> R,
    {
        let hooks = RefCell::new(Vec::new());
        let r = logic(UseHook(UseHookInner::Create(&hooks)));
        (
            Self {
                hooks: hooks
                    .into_inner()
                    .into_iter()
                    .map(|opt| opt.unwrap())
                    .collect(),
            },
            r,
        )
    }

    pub fn call(&mut self) -> UseHook {
        UseHook(UseHookInner::Reuse(self.hooks.iter_mut()))
    }
}

pub struct UseHook<'a>(UseHookInner<'a>);

enum UseHookInner<'a> {
    Create(CreateVec<'a>),
    Reuse(std::slice::IterMut<'a, Box<dyn Any>>),
}

impl<'a> UseHook<'a> {
    pub fn define<T, F>(&mut self, init: F) -> Hook<'a, T>
    where
        T: 'static,
        F: FnOnce() -> T,
    {
        match &mut self.0 {
            UseHookInner::Create(vec) => {
                let mut vec_ref = vec.borrow_mut();
                let index = vec_ref.len();
                vec_ref.push(None);

                Hook(HookInner::Create {
                    value: Some(Box::new(init())),
                    vec,
                    index,
                })
            }
            UseHookInner::Reuse(iter) => {
                let this_storage = iter.next().expect("this hook not rendered when creating");
                Hook(HookInner::Reuse(
                    this_storage
                        .downcast_mut::<T>()
                        .expect("this hook incorrect"),
                ))
            }
        }
    }
}

pub struct Hook<'a, T: 'static>(HookInner<'a, T>);

enum HookInner<'a, T: 'static> {
    Create {
        value: Option<Box<T>>,
        vec: CreateVec<'a>,
        index: usize,
    },
    Reuse(&'a mut T),
}

impl<T: 'static> Drop for HookInner<'_, T> {
    fn drop(&mut self) {
        if let Self::Create { value, vec, index } = self {
            vec.borrow_mut()[*index] = value.take().map(|x| x as _);
        }
    }
}

impl<T: 'static> Deref for Hook<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match &self.0 {
            HookInner::Create { value, .. } => value.as_ref().unwrap(),
            HookInner::Reuse(r) => r,
        }
    }
}

impl<T: 'static> DerefMut for Hook<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.0 {
            HookInner::Create { value, .. } => value.as_mut().unwrap(),
            HookInner::Reuse(r) => r,
        }
    }
}
