use crate::color::Color;
use crate::events::{EventHandler, Hover, Key, Pointer, Scroll};
use crate::impl_view;
use crate::rect::Rect;
use crate::view::{Fragment, Layout, NativeType, View};
use cgmath::{Matrix3, SquareMatrix};
use core::{fmt, mem};

#[cfg(target_os = "macos")]
use swift_birb::protocol::SBLayerPatch;

/// A native view that contains graphical content and may have subviews.
pub struct Layer<Ctx> {
    pub key: Option<u64>,

    /// Layer bounds.
    pub bounds: Rect,

    /// Background color, with which the layer bounds will be filled--respecting the corner radius.
    pub background: Color,

    /// Corner radius.
    pub corner_radius: f64,

    /// Border (width, color).
    pub border: Option<(f64, Color)>,

    /// Whether contents will be clipped to the layer’s bounds.
    pub clip_contents: bool,

    /// Layer affine transform.
    pub transform: Matrix3<f64>,

    /// Layer opacity.
    pub opacity: f64,

    /// Subviews of this layer.
    pub subviews: Fragment<Ctx>,

    /// Layout handler for this layer.
    pub layout: Box<dyn Layout>,

    // event handlers
    pub pointer_action: Option<EventHandler<Pointer>>,
    pub hover_action: Option<EventHandler<Hover>>,
    pub key_action: Option<EventHandler<Key>>,
    pub scroll_action: Option<EventHandler<Scroll>>,
}

struct DebugifyOption<'a, T>(&'a Option<T>);
impl<'a, T> fmt::Debug for DebugifyOption<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0.is_some() {
            write!(f, "Some(..)")
        } else {
            write!(f, "None")
        }
    }
}

impl<Ctx> fmt::Debug for Layer<Ctx> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Layer")
            .field("bounds", &self.bounds)
            .field("background", &self.background)
            .field("corner_radius", &self.corner_radius)
            .field("border", &self.border)
            .field("clip_contents", &self.clip_contents)
            .field("transform", &self.transform)
            .field("opacity", &self.opacity)
            .field("subviews", &self.subviews)
            .field("pointer_down_action", &DebugifyOption(&self.pointer_action))
            .field("pointer_hover_action", &DebugifyOption(&self.hover_action))
            .field("key_down_action", &DebugifyOption(&self.key_action))
            .field("scroll_action", &DebugifyOption(&self.scroll_action))
            .finish()
    }
}

// TODO: builder methods

impl<Ctx> Default for Layer<Ctx> {
    fn default() -> Self {
        Layer {
            key: None,
            bounds: Rect::zero(),
            background: Color::default(),
            corner_radius: 0.,
            border: None,
            clip_contents: false,
            transform: Matrix3::identity(),
            opacity: 1.,
            subviews: Vec::new(),
            pointer_action: None,
            hover_action: None,
            key_action: None,
            scroll_action: None,
            layout: Box::new(()),
        }
    }
}

impl<Ctx: 'static> PartialEq for Layer<Ctx> {
    fn eq(&self, other: &Layer<Ctx>) -> bool {
        self.bounds == other.bounds
            && self.background == other.background
            && self.corner_radius == other.corner_radius
            && self.border == other.border
            && self.clip_contents == other.clip_contents
            && self.transform == other.transform
            && self.opacity == other.opacity
            && self.subviews.eq(&other.subviews)
        // TODO: cmp event handlers?
    }
}

impl_view! {
    Layer<Ctx>;
    fn new_state(&self) {
        Box::new(())
    }
    fn body(&self, _state: &()) {
        std::sync::Arc::new(self.subviews.clone())
    }
    fn native_type(&self) -> Option<NativeType> {
        Some(NativeType::Layer)
    }
    fn key(&self) -> Option<u64> {
        self.key
    }
}

impl<Ctx> Layer<Ctx> {
    pub(crate) fn as_patch(&self) -> SBLayerPatch {
        SBLayerPatch {
            bounds: self.bounds.into(),
            background: self.background.into(),
            corner_radius: self.corner_radius,
            border_width: self.border.map_or(0., |(w, _)| w),
            border_color: self.border.map_or(Color::default(), |(_, c)| c).into(),
            clip_contents: self.clip_contents,
            transform: unsafe { mem::transmute(self.transform) }, // totally safe, trust me
            opacity: self.opacity,
        }
    }
}
