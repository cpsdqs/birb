use crate::protocol::*;
use birb::backend::Backend;
use birb::color::Color;
use birb::raw_events::RawEvent;
use birb::NativeView;
use birb::Rect;
use cgmath::{Matrix3, Point2, Vector2};
use core::convert::TryInto;
use core::ffi::c_void;
use core::marker::PhantomData;
use core::mem;
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

impl Into<SBVector2> for Point2<f64> {
    fn into(self) -> SBVector2 {
        SBVector2 {
            x: self.x,
            y: self.y,
        }
    }
}
impl Into<SBVector2> for Vector2<f64> {
    fn into(self) -> SBVector2 {
        SBVector2 {
            x: self.x,
            y: self.y,
        }
    }
}
impl Into<SBRect> for Rect {
    fn into(self) -> SBRect {
        SBRect {
            origin: self.origin.into(),
            size: self.size.into(),
        }
    }
}
impl Into<SBColor> for Color {
    fn into(self) -> SBColor {
        SBColor {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a,
        }
    }
}
impl Into<SBMatrix3> for Matrix3<f64> {
    fn into(self) -> SBMatrix3 {
        SBMatrix3 {
            m00: self.x.x,
            m01: self.x.y,
            m02: self.x.z,
            m10: self.y.x,
            m11: self.y.y,
            m12: self.y.z,
            m20: self.z.x,
            m21: self.z.y,
            m22: self.z.z,
        }
    }
}

type SomeUnsendType = *mut ();

/// SBHost (see SwiftBirb).
///
/// Must only be used on the “main” thread (i.e. whichever thread connects to Cocoa).
#[repr(C)]
struct Host(Id<Object>, PhantomData<SomeUnsendType>);

/// This must invariably have the same memory layout as an objective-c id.
#[repr(C)]
pub struct SBViewRef {
    obj: Id<Object>,
    _phantom: PhantomData<SomeUnsendType>,
}

impl SBViewRef {
    fn new(obj: Id<Object>) -> Self {
        SBViewRef {
            obj,
            _phantom: PhantomData,
        }
    }

    fn update(&mut self, patch: SBNodePatch) {
        unsafe {
            let _: () = msg_send![self.obj, updateWithPatch: patch];
        }
    }

    fn replace(&mut self, patch: SBNodePatch) {
        unsafe {
            let _: () = msg_send![self.obj, replaceWithPatch: patch];
        }
    }

    fn set_subviews(&mut self, offset: u64, length: u64, subviews: protocol::SBNodeList) {
        unsafe {
            let _: () =
                msg_send![self.obj, setSubviewsWithOffset:offset length:length subviews:subviews];
        }
    }

    fn remove(&mut self) {
        unsafe {
            let _: () = msg_send![self.obj, remove];
        }
    }
}

impl Host {
    pub fn new() -> Host {
        unsafe {
            let birb_host_class = SBHostingView_getClass();
            let i: *mut Object = msg_send![birb_host_class, alloc];
            let id = msg_send![i, init];
            Host(Id::from_retained_ptr(id), PhantomData)
        }
    }

    fn new_view(&mut self, patch: SBNodePatch) -> Result<SBViewRef, SBError> {
        unsafe {
            let node: Id<Object> = msg_send![self.0, createView: patch];
            Ok(SBViewRef::new(node))
        }
    }

    fn set_root_view(&mut self, view: &SBViewRef) {
        unsafe {
            let _: () = msg_send![self.0, setRootView:&view.obj];
        }
    }

    /// Returns a reference to the SBHostingView object.
    fn object(&mut self) -> &mut Id<Object> {
        &mut self.0
    }
}

fn nv_to_patch(nv: NativeView) -> SBNodePatch {
    match nv {
        NativeView::Layer {
            bounds,
            background,
            corner_radius,
            border_width,
            border_color,
            clip_contents,
            transform,
            opacity,
        } => SBNodePatch {
            type_: SBNodeTypeLayer,
            patch: SBNodePatchData {
                layer: SBLayerPatch {
                    bounds: bounds.into(),
                    background: background.into(),
                    border_color: border_color.into(),
                    border_width,
                    clip_contents,
                    corner_radius,
                    opacity,
                    transform: transform.into(),
                },
            },
        },
    }
}

pub enum SBError {}

/// SwiftBirb backend. Must only be used on the main thread.
pub struct SwiftBirb {
    host: Host,
}

impl SwiftBirb {
    pub fn new() -> SwiftBirb {
        SwiftBirb { host: Host::new() }
    }
}

impl Backend for SwiftBirb {
    type ViewRef = SBViewRef;
    type Error = SBError;

    fn new_view(&mut self, view: NativeView) -> Result<SBViewRef, SBError> {
        self.host.new_view(nv_to_patch(view))
    }

    fn update_view(&mut self, view: &mut SBViewRef, patch: NativeView) -> Result<(), SBError> {
        view.update(nv_to_patch(patch));
        Ok(())
    }

    fn remove_view(&mut self, mut view: SBViewRef) -> Result<(), SBError> {
        view.remove();
        Ok(())
    }

    fn replace_view(&mut self, view: &mut SBViewRef, patch: NativeView) -> Result<(), SBError> {
        view.replace(nv_to_patch(patch));
        Ok(())
    }

    fn set_subviews<'a>(
        &mut self,
        view: &mut SBViewRef,
        region_start: usize,
        region_len: usize,
        subviews: Vec<&'a SBViewRef>,
    ) -> Result<(), SBError> {
        let region_start = region_start.try_into().unwrap();
        let region_len = region_len.try_into().unwrap();

        let subviews_count = subviews.len().try_into().unwrap();
        // Safety: SBViewRef is memory-compatible with objc id...
        const _: [(); mem::size_of::<Id<Object>>()] = [(); mem::size_of::<SBViewRef>()];
        // ...hence this is a valid pointer to a list of ids.
        // Vec capacity does not matter in this case because the pointer can be deallocated without
        // knowing its size.
        let subviews_ptr =
            unsafe { mem::transmute::<*const &SBViewRef, *mut c_void>(subviews.as_ptr()) };

        let node_list = SBNodeList {
            nodes: subviews_ptr,
            count: subviews_count,
        };

        // this vec was converted into raw parts; must not drop it
        mem::forget(subviews);

        view.set_subviews(region_start, region_len, node_list);
        Ok(())
    }

    fn set_root_view(&mut self, view: &mut SBViewRef) -> Result<(), SBError> {
        self.host.set_root_view(view);
        Ok(())
    }

    fn poll(&mut self) -> Result<Option<RawEvent>, SBError> {
        todo!()
    }
}
