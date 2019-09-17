use core::marker::PhantomData;
use objc::runtime::*;
use objc::{msg_send, sel, sel_impl};
use objc_id::Id;

#[link(name = "SwiftBirb")]
extern "C" {
    fn SBHostingView_getClass() -> *mut Object;
}

pub mod protocol {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]

    include!(concat!(env!("OUT_DIR"), "/protocol.rs"));
}

type SomeUnsendType = *mut ();

/// SBHost (see SwiftBirb).
///
/// Must only be used on the “main” thread (i.e. whichever thread connects to Cocoa).
#[repr(C)]
pub struct Host(Id<Object>, PhantomData<SomeUnsendType>);

impl Host {
    pub unsafe fn new(dispatcher: protocol::SBEventDispatcher, user_data: usize) -> Host {
        let birb_host_class = SBHostingView_getClass();
        let i: *mut Object = msg_send![birb_host_class, alloc];
        let id = msg_send![i, initWithDispatcher:dispatcher userData:user_data];
        Host(Id::from_retained_ptr(id), PhantomData)
    }

    pub unsafe fn user_data(&mut self) -> usize {
        msg_send![self.0, getUserData]
    }

    pub unsafe fn patch(&mut self, patch: protocol::SBEvent) {
        msg_send![self.0, patch: patch]
    }

    /// Returns a reference to the SBHostingView object.
    pub fn object(&mut self) -> &mut Id<Object> {
        &mut self.0
    }
}
