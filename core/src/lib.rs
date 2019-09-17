//! UI library.
//!
//! # Conceptual overview
//! Birb is a declarative view-based cross-platform UI framework.
//!
//! ## Views
//! There are two types of views: native views, and regular composite views (as well as some special
//! view types like Fragments). Native views like layers or text will be visible on-screen and can
//! be interacted with, while composite views are simply made up of other (simpler) views.
//!
//! Views have properties, state, and a body. When a view is created in code, it is not an actual
//! view but a virtual representation of a view, and should hence be very cheap to create, starting
//! with constructors: views should not do anything on their own when created other than storing
//! their properties. When the view is realized, it will be asked to create a state object which
//! will persist over the lifetime of the view—side effects and other things should be taken care
//! of here. Finally, the view body is derived from its properties and its state and declares the
//! view’s subviews.
//!
//! ## Events
//! When events arrive at a window, they will first target a specific view and then bubble up.
//! For keyboard events, the first target will be the view that has keyboard focus, and for pointer
//! events, the first target will be the topmost view with a pointer tracking rectangle under the
//! pointer.
//!
//! Pointer tracking rectangles are screen regions with a z-index that will occlude all tracking
//! rectangles below, and define the regions in which their owner views will receive pointer events.
//! Since not all views need pointer events, not all views have tracking rectangles.
//! Also note that views that clip their contents may clip all contained tracking rectangles to
//! their tracking rectangle as well.
//!
//! Events typically happen in multiple phases: pointers are pressed, moved, and released; keys are
//! pressed, possibly repeated, and released. Hence, this event architecture reflects that:
//! when an initial event (such as a pointer-down or keypress) arrives at a target view, *all* of
//! its ancestors will also receive the event. Each view then has an opportunity to indicate that
//! it wishes to continue receiving this event, along with a priority level.
//! The views that indicated the highest priority level will then continue receiving events—all
//! others will be notified that their event stream has been canceled.
//!
//! Over the course of an event views may increase their priority to capture events for themselves.
//! For example, when the user presses down on a list item, both the list item and the list scroll
//! view may register themselves with the same priority—but as soon as the pointer is moved, the
//! scroll view will take control to begin scrolling, and the list item can no longer be selected.
//!
//! ## Layout
//! Layout is performed top-down, meaning a superview will perform its layout first, define the
//! bounds of its subviews, and then the subviews will do the same. A view may output different
//! bounds than given by its superview: if a subview finds its size unsatisfactory, it should
//! request a layout frame, so that in the next frame, layout is performed again; this time with the
//! superview aware of its minimum size.
//!
//! ## Contexts
//! Contexts are used to propagate lateral parameters (e.g. a UI theme) down the view tree without
//! having to copy it into the view props every single time. They should be cheap to create and
//! clone (possibly making use of Arcs). Views may choose to modify the context to be different
//! for their subviews, too.
//!
//! ## Coordinate System
//! As the host is usually a window, this will be in terms of windows: the origin of the top-level
//! coordinate system is at the top left corner of the window’s content area. The y-axis is oriented
//! such that positive y points down. The z-axis points outwards from the screen.
//!
//! ## NVTree and Backends
//! To get the views in a ViewTree to show up on screen, an NVTree (native-view tree) and a backend
//! is required. The NVTree is like the ViewTree—except it only contains native views—and is the
//! structure where events and layout are handled. It’s also responsible for keeping the backend
//! in sync with the view tree.
//!
//! Backends are platform-specific UI frameworks like Cocoa; abstracted to a common interface. Some
//! backends may provide more features than others.
//!
//! All backends are guaranteed to support:
//!
//! - Layers
//! - Text
//! - Surfaces
//! - at least one type of pointer events

pub mod backend;
pub mod color;
pub mod events;
mod layer;
mod nv_tree;
pub mod raw_events;
mod rect;
#[macro_use]
mod view;
mod view_tree;

pub use nv_tree::{NVTree, NativeView, Patch};
pub use rect::Rect;
pub use view::{State, View};
pub use view_tree::{Context, ViewTree};
