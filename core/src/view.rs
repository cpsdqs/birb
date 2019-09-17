use crate::rect::Rect;
use crate::view_tree::Context;
use cgmath::{Vector2, Zero};
use core::any::Any;
use core::fmt;
use std::sync::Arc;
use uuid::Uuid;

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
///     StructName; // or StructName : ContextType
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
        $(fn new_state(&$ns_self:ident, $ns_ctx:ident) $new_state:tt)*
        fn body(&$self:ident, $state_var:ident: &$state_type:ty) $body:tt
        $($extra:tt)*
    ) => {
        $(#[$attr])*
        impl<Ctx: 'static> $crate::View<Ctx> for $struct {
            $crate::impl_view!(__internal1);
            $($crate::impl_view!(__internal2, Ctx, $ns_self, $ns_ctx, $new_state);)*
            $crate::impl_view!(__internal3, Ctx, $self, $state_var, $state_type, $body, $struct);
            $crate::impl_view!(__internal4, Ctx, $struct);
            $($extra)*
        }
    };
    (
        $(#[$attr:meta])*
        $struct:ty : $ctx:ty;
        $(fn new_state(&$ns_self:ident, $ns_ctx:ident) $new_state:tt)*
        fn body(&$self:ident, $state_var:ident: &$state_type:ty) $body:tt
        $($extra:tt)*
    ) => {
        $(#[$attr])*
        impl $crate::View<$ctx> for $struct {
            $crate::impl_view!(__internal1);
            $($crate::impl_view!(__internal2, $ctx, $ns_self, $ns_ctx, $new_state);)*
            $crate::impl_view!(__internal3, $ctx, $self, $state_var, $state_type, $struct);
            $crate::impl_view!(__internal4, $ctx, $struct);
            $($extra)*
        }
    };
    (__internal1) => {
        fn as_any(&self) -> &dyn ::core::any::Any {
            self
        }
    };
    (__internal2, $ctx:ty, $ns_self:ident, $ns_ctx:ident, $new_state:tt) => {
        fn new_state(
            &$ns_self,
            $ns_ctx:ident: $crate::Context<$ctx>,
        ) -> Box<dyn $crate::State<$ctx>> {
            $new_state
        }
    };
    (__internal3, $ctx:ty, $self:ident, $state_var:ident, $state_type:ty, $body:tt, $struct:ty) => {
        fn body(&$self, state: &dyn ::core::any::Any) -> ::std::sync::Arc<dyn $crate::View<$ctx>> {
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
    };
    (__internal4, $ctx:ty, $struct:ty) => {
        fn eq(&self, other: &dyn $crate::View<$ctx>) -> bool {
            if let Some(other) = other.as_any().downcast_ref::<$struct>() {
                self == other
            } else {
                false
            }
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
pub trait View<Ctx>: Any + fmt::Debug + Send + Sync {
    /// Creates a new state object for this view.
    ///
    /// Will create [`()`] by default.
    fn new_state(&self, context: Context<Ctx>) -> Box<dyn State<Ctx>> {
        drop(context);
        Box::new(())
    }

    /// Renders the body of this view.
    fn body(&self, state: &dyn Any) -> Arc<dyn View<Ctx>>;

    /// Compares this view to another; used for diffing.
    fn eq(&self, other: &dyn View<Ctx>) -> bool;

    /// For downcasting.
    fn as_any(&self) -> &dyn Any;

    /// A key used to identify this view in an array of views.
    ///
    /// Should be derived from a `key` property.
    fn key(&self) -> Option<u64> {
        None
    }

    /// Returns a subview context.
    fn subview_context(&self, state: &dyn Any, context: &Ctx) -> Option<Ctx> {
        drop(state);
        drop(context);
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
    fn is_same_type(&self, other: &dyn View<Ctx>) -> bool {
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
    Surface,
    VisualEffectView,
}

/// View state associated with a view.
///
/// Will be dropped right after the view disappears.
pub trait State<Ctx>: Any + fmt::Debug + Send {
    /// For downcasting.
    fn as_any(&self) -> &dyn Any;

    /// Called before the component is updated from a new virtual view.
    fn will_update(&self, update: &dyn View<Ctx>) {
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
impl<Ctx> State<Ctx> for () {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub type Fragment<Ctx> = Vec<Arc<dyn View<Ctx>>>;

/// A fragment view that expands into its children.
impl<Ctx: 'static> View<Ctx> for Fragment<Ctx> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn body(&self, _: &dyn Any) -> Arc<dyn View<Ctx>> {
        Arc::new(self.clone())
    }
    fn eq(&self, other: &dyn View<Ctx>) -> bool {
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
    fn layout(&self, bounds: Rect, mut context: LayoutContext) -> LayoutResult {
        LayoutResult {
            bounds,
            subview_bounds: context.subviews().map(|_| bounds).collect(),
            min_size: Vector2::zero(),
            track_pointer: false,
            clip_pointer: false,
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
    /// Own view bounds.
    bounds: Rect,

    /// Bounds for all subviews, in order.
    subview_bounds: Vec<Rect>,

    /// Minimum size of this view.
    min_size: Vector2<f64>,

    /// If true, will consider the layout bounds a pointer tracking rectangle.
    track_pointer: bool,

    /// If true, will clip all pointer tracking rectangles of child views to this view.
    clip_pointer: bool,
}

/// Identity layout.
///
/// Use this to use the default layout handler, which copies the bounds given by the superview to
/// all subviews and itself.
impl Layout for () {}
