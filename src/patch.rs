use crate::color::Color;
use crate::events::EventTypeId;
use crate::layer::Layer;
use crate::rect::Rect;
use crate::tree::{EventHandlers, HandlerId, ViewId};
use cgmath::Matrix3;
use core::fmt;

/// Patches for native views.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Patch {
    ty: PatchType,
    view: ViewId,
    data: PatchData,
}

impl Patch {
    /// Update (or add) a view.
    pub fn update(view: ViewId, update: LayerPatch) -> Patch {
        Patch {
            ty: PatchType::Update,
            view,
            data: PatchData { update },
        }
    }

    /// Set up a parent-child relationship.
    ///
    /// This patch will only be called once per view.
    ///
    /// The parent view may be nonexistent in case of the first root view descendant.
    pub fn subview(view: ViewId, subview: ViewId) -> Patch {
        Patch {
            ty: PatchType::Subview,
            view,
            data: PatchData { subview },
        }
    }

    /// Remove a view.
    pub fn remove(view: ViewId) -> Patch {
        Patch {
            ty: PatchType::Remove,
            view,
            data: PatchData { remove: () },
        }
    }
}

impl fmt::Debug for Patch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            match self.ty {
                PatchType::Update => write!(f, "Update({:?}, {:?})", self.view, self.data.update),
                PatchType::Subview => {
                    write!(f, "Subview({:?}, {:?})", self.view, self.data.subview)
                }
                PatchType::Remove => write!(f, "Remove({:?})", self.view),
            }
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum PatchType {
    Update = 0,
    Subview = 1,
    Remove = 2,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union PatchData {
    update: LayerPatch,
    subview: ViewId,
    remove: (),
    order: ViewIdList,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NodePatch {
    ty: NodePatchType,
    data: NodePatchData,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum NodePatchType {
    Layer = 0,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union NodePatchData {
    layer: LayerPatch,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ViewIdList {
    len: usize,
    cap: usize,
    ptr: *const ViewId,
}

/// A serialized layer patch.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayerPatch {
    pub bounds: Rect,
    pub background: Color,
    pub corner_radius: f64,
    pub border_width: f64,
    pub border_color: Color,
    pub clip_contents: bool,
    pub transform: Matrix3<f64>,
    pub opacity: f64,
    pub hover_action: HandlerId,
    pub pointer_action: HandlerId,
    pub key_action: HandlerId,
    pub scroll_action: HandlerId,
}

impl LayerPatch {
    pub(crate) fn new(layer: &Layer, id: ViewId, handlers: &mut EventHandlers) -> Self {
        macro_rules! register_action {
            ($e:expr, $t:tt) => {{
                if let Some(action) = $e {
                    handlers.add_handler(id, action.clone());
                } else {
                    // remove existing
                    handlers.remove_handler(id, EventTypeId::$t)
                }
                (id, EventTypeId::$t)
            }};
        }

        LayerPatch {
            bounds: layer.bounds,
            background: layer.background,
            corner_radius: layer.corner_radius,
            border_width: layer.border.map(|(width, _)| width).unwrap_or(0.),
            border_color: layer.border.map(|(_, color)| color).unwrap_or_default(),
            clip_contents: layer.clip_contents,
            transform: layer.transform,
            opacity: layer.opacity,
            hover_action: register_action!(&layer.hover_action, Hover),
            pointer_action: register_action!(&layer.pointer_action, Pointer),
            key_action: register_action!(&layer.key_action, Key),
            scroll_action: register_action!(&layer.scroll_action, Scroll),
        }
    }
}
