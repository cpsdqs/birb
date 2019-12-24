//! Traits for backends.

use crate::nv_tree::NativeView;
use crate::raw_events::RawEvent;

/// A backend implementation.
pub trait Backend {
    /// A reference to a view in the backend.
    type ViewRef;

    /// Error type.
    type Error;

    /// Creates a new view.
    fn new_view(&mut self, view: NativeView) -> Result<Self::ViewRef, Self::Error>;

    /// Removes a view from the view hierarchy.
    fn remove_view(&mut self, view: Self::ViewRef) -> Result<(), Self::Error>;

    /// Updates a view.
    fn update_view(
        &mut self,
        view: &mut Self::ViewRef,
        view: NativeView,
    ) -> Result<(), Self::Error>;

    /// Replaces a view in the view hierarchy with another.
    ///
    /// Similar to update, but the view types will be different.
    fn replace_view(
        &mut self,
        view: &mut Self::ViewRef,
        view: NativeView,
    ) -> Result<(), Self::Error>;

    /// Sets a region of the view’s subviews.
    fn set_subviews<'a>(
        &mut self,
        view: &mut Self::ViewRef,
        region_start: usize,
        region_len: usize,
        subviews: Vec<&'a Self::ViewRef>,
    ) -> Result<(), Self::Error>;

    /// Sets the root view.
    fn set_root_view(&mut self, view: &mut Self::ViewRef) -> Result<(), Self::Error>;

    /// Returns the next event from the queue.
    ///
    /// This method may be called frequently in quick succession.
    fn poll(&mut self) -> Result<Option<RawEvent>, Self::Error>;
}
