use crate::context::Context;
use crate::rect::Rect;
use cgmath::{Vector2, Zero};
use core::any::Any;
use core::fmt;
use std::sync::Arc;
use uuid::Uuid;

#[cfg(target_os = "macos")]
use swift_birb::protocol::SBViewId;

/// A unique identifier for a view.
///
/// (this is just a UUID)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ViewId(u32, u16, u16, [u8; 8]);

impl ViewId {
    pub(crate) fn new() -> ViewId {
        let uuid = Uuid::new_v4();
        let (a, b, c, d) = uuid.as_fields();
        ViewId(a, b, c, *d)
    }
}

impl Into<SBViewId> for ViewId {
    fn into(self) -> SBViewId {
        SBViewId {
            a: self.0,
            b: self.1,
            c: self.2,
            d: self.3,
        }
    }
}

// TODO: state might need to be Arc'd so callback closures can use it
// or i could also use message enums and a send_message function

/// Implements the `View` trait for a given struct.
///
/// Assumes that `PartialEq` is implemented. `Eq` would be preferred to avoid frequent updates.
///
/// Syntax:
///
/// ```text
/// impl_view! {
///     StructName;
///     fn new_state(&self) { // optional
///         ... -> Box<dyn State>
///     }
///     fn body(&self, state_variable: StateType) {
///         ... -> Box<dyn View>
///     }
///     (put extra items like key() here, using normal rust syntax)
/// }
/// ```
#[macro_export]
macro_rules! impl_view {
    (
        $(#[$attr:meta])*
        $struct:ty;
        $(fn new_state(&$ns_self:ident) $new_state:tt)*
        fn body(&$self:ident, $state_var:ident: &$state_type:ty) $body:tt
        $($extra:tt)*
    ) => {
        $(#[$attr])*
        impl $crate::View for $struct {
            fn as_any(&self) -> &dyn ::core::any::Any {
                self
            }

            $(fn new_state(&$ns_self) -> Box<dyn $crate::State> {
                $new_state
            })*

            fn body(&$self, state: &dyn ::core::any::Any) -> ::std::sync::Arc<dyn $crate::View> {
                if let Some($state_var) = state.downcast_ref::<$state_type>() {
                    fn _dont_complain_about_unused<T>(_: T) {}
                    _dont_complain_about_unused($state_var);
                    $body
                } else {
                    panic!(
                        "View::body: invalid state for {}; expected type {}",
                        stringify!($struct),
                        stringify!($state_type)
                    );
                }
            }

            fn eq(&self, other: &dyn $crate::View) -> bool {
                if let Some(other) = other.as_any().downcast_ref::<$struct>() {
                    self == other
                } else {
                    false
                }
            }

            $($extra)*
        }
    };
}

/// Views are the basic components of UI: they encapsulate properties and state to render a body
/// that’s composed of more views.
///
/// `View` implementors themselves should be cheap and fast to create, as they are not actual views
/// but their virtual representation à la virtual DOM. Similarly, `body` should be fast to compute,
/// preferably as a pure function dependent only on the view properties and the view state.
///
/// This trait should probably be implemented using the [`impl_view`] macro.
///
/// # Panics
/// `body` should always return a native view, eventually. Notably, care should be taken when
/// returning non-native views such that it doesn’t cause a cycle and end up causing an infinite
/// loop.
pub trait View: Any + fmt::Debug + Send + Sync {
    /// Creates a new state object for this view.
    ///
    /// Will create [`()`] by default.
    fn new_state(&self) -> Box<dyn State> {
        Box::new(())
    }

    /// Renders the body of this view.
    fn body(&self, state: &dyn Any) -> Arc<dyn View>;

    /// Compares this view to another; used for diffing.
    fn eq(&self, other: &dyn View) -> bool;

    /// For downcasting.
    fn as_any(&self) -> &dyn Any;

    /// A key used to identify this view in an array of views.
    ///
    /// Should be derived from a `key` property.
    fn key(&self) -> Option<u64> {
        None
    }

    /// Returns the native type if this is a native view.
    ///
    /// Should always be None for types outside of this crate.
    #[doc(hidden)]
    fn native_type(&self) -> Option<NativeType> {
        None
    }

    /// For proxy views; should not be overridden usually.
    ///
    /// Will be called if the views have the same TypeId, so the default implementation that always
    /// returns true should be fine for almost all views.
    fn is_same_type(&self, other: &dyn View) -> bool {
        drop(other);
        true
    }
}

/// Types of native views.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeType {
    Layer,
    Text,
    TextField,
    VkSurface,
    VisualEffectView,
}

/// View state associated with a view.
///
/// Will be dropped right after the view disappears.
pub trait State: Any + fmt::Debug {
    /// For downcasting.
    fn as_any(&self) -> &Any;

    /// Called before the associated view will appear.
    fn will_appear(&self, context: &Context) {
        drop(context);
    }

    /// Called after the associated view has appeared and been rendered.
    fn did_appear(&self) {}

    /// Called before the component is updated from a new virtual view.
    fn will_update(&self, update: &dyn View) {
        drop(update);
    }
}

impl_view! {
    /// An empty view type that does absolutely nothing.
    ();
    fn body(&self, _state: &()) {
        Arc::new(())
    }
}

/// For stateless views.
impl State for () {
    fn as_any(&self) -> &Any {
        self
    }
}

pub type Fragment = Vec<Arc<dyn View>>;

/// A fragment view that expands into its children.
impl View for Fragment {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn body(&self, _: &dyn Any) -> Arc<dyn View> {
        Arc::new(self.clone())
    }
    fn eq(&self, other: &dyn View) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            if self.len() != other.len() {
                return false;
            }
            for (i, j) in self.iter().zip(other.iter()) {
                if !i.eq(&**j) {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
}

/// A layout delegate for a native view.
pub trait Layout: Any + fmt::Debug + Send + Sync {
    /// Performs layout.
    ///
    /// - `bounds`: the (strongly) suggested bounds from the superview.
    /// - `context`: the layout context. Used to access subview layout.
    fn layout(&self, state: &dyn State, bounds: Rect, mut context: LayoutContext) -> LayoutResult {
        let _ = state;

        LayoutResult {
            bounds,
            subview_bounds: context.subviews().map(|_| bounds).collect(),
            min_size: Vector2::zero(),
        }
    }
}

pub struct LayoutContext<'a> {
    // tree: &'a mut ViewTree,
    tree: &'a mut (),
}

impl<'a> LayoutContext<'a> {
    pub fn subviews(&mut self) -> impl Iterator<Item = SubviewLayout<'_>> {
        // TODO
        Vec::new().into_iter()
    }
}

pub struct SubviewLayout<'a> {
    context: &'a mut LayoutContext<'a>,
}

impl<'a> SubviewLayout<'a> {
    /// Performs layout if it hasn’t been run already.
    pub fn force_layout(&mut self) {
        unimplemented!()
    }

    /// The subview’s minimum size.
    /// May be zero if it hasn’t been computed yet (e.g. on first render).
    /// If it’s important, use `force_layout` to try and get it a frame earlier.
    pub fn min_size(&self) -> Vector2<f64> {
        unimplemented!()
    }
}

pub struct LayoutResult {
    bounds: Rect,
    subview_bounds: Vec<Rect>,
    min_size: Vector2<f64>,
}

/// Identity layout.
///
/// Use this to use the default layout handler, which copies the bounds given by the superview to
/// all subviews and itself.
impl Layout for () {}
