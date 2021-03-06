//! Execution contexts and sandboxing.
use v8_sys as v8;
use isolate;
use util;
use value;

/// A sandboxed execution context with its own set of built-in objects and functions.
#[derive(Debug)]
pub struct Context(isolate::Isolate, v8::ContextRef);

/// A guard that keeps a context bound while it is in scope.
#[must_use]
pub struct ContextGuard<'a>(&'a Context);

impl Context {
    /// Creates a new context and returns a handle to the newly allocated context.
    pub fn new(isolate: &isolate::Isolate) -> Context {
        unsafe {
            Context(isolate.clone(),
                    util::invoke(isolate, |c| v8::Context_New(c)).unwrap())
        }
    }

    /// Binds the context to the current scope.
    ///
    /// Within this scope, functionality that relies on implicit contexts will work.
    pub fn make_current(&self) -> ContextGuard {
        self.enter();
        ContextGuard(self)
    }

    fn enter(&self) {
        unsafe { util::invoke(&self.0, |c| v8::Context_Enter(c, self.1)).unwrap() }
    }

    fn exit(&self) {
        unsafe { util::invoke(&self.0, |c| v8::Context_Exit(c, self.1)).unwrap() }
    }

    /// Returns the global proxy object.
    ///
    /// Global proxy object is a thin wrapper whose prototype points to actual context's global
    /// object with the properties like Object, etc. This is done that way for security reasons (for
    /// more details see https://wiki.mozilla.org/Gecko:SplitWindow).
    ///
    /// Please note that changes to global proxy object prototype most probably would break VM---v8
    /// expects only global object as a prototype of global proxy object.
    ///
    pub fn global(&self) -> value::Object {
        unsafe {
            value::Object::from_raw(&self.0,
                                    util::invoke(&self.0, |c| v8::Context_Global(c, self.1))
                                        .unwrap())
        }
    }

    /// Creates a context from a set of raw pointers.
    pub unsafe fn from_raw(isolate: &isolate::Isolate, raw: v8::ContextRef) -> Context {
        Context(isolate.clone(), raw)
    }

    /// Returns the underlying raw pointer behind this context.
    pub fn as_raw(&self) -> v8::ContextRef {
        self.1
    }
}

reference!(Context, v8::Context_CloneRef, v8::Context_DestroyRef);

impl<'a> Drop for ContextGuard<'a> {
    fn drop(&mut self) {
        self.0.exit()
    }
}
