use crate::rect::Rect;
use crate::view::{LayoutResult, ViewId};
use std::collections::HashMap;

#[derive(Clone)]
pub enum NativeView {
    Layer,
}

/// Patches for the NV tree.
#[derive(Clone)]
pub enum Patch {
    /// Updates or creates a view.
    Update(ViewId, NativeView),
    /// Deletes and re-creates a view.
    Replace(ViewId, NativeView),
    /// Sets a view’s subviews.
    Subviews(ViewId, Vec<ViewId>),
    /// Removes a view.
    /// **Does not remove the view from the superview’s subview references.**
    Remove(ViewId),
}

/// Errors that may occur when running a patch.
pub enum PatchError {
    NoSuchView(ViewId),
}

struct NVTNode {
    view: NativeView,
    superview: Option<ViewId>,
    subviews: Vec<ViewId>,
    layout: Option<LayoutResult>,
}

/// The native-view tree; handles layout, events, and backends.
pub struct NVTree {
    nodes: HashMap<ViewId, NVTNode>,
    // TODO: spatial index
    tracking_rects: HashMap<ViewId, Rect>,
}

impl NVTree {
    pub fn new() -> NVTree {
        NVTree {
            nodes: HashMap::new(),
            tracking_rects: HashMap::new(),
        }
    }

    /// Patches the view tree.
    pub fn patch(&mut self, patch: Patch) -> Result<(), PatchError> {
        match patch {
            Patch::Update(id, view) => self.update_view(id, view),
            Patch::Replace(id, view) => self.replace_view(id, view),
            Patch::Subviews(id, subviews) => self.set_subviews(id, subviews),
            Patch::Remove(id) => self.remove_view(id),
        }
    }

    /// Updates or adds a view.
    fn update_view(&mut self, id: ViewId, view: NativeView) -> Result<(), PatchError> {
        if let Some(node) = self.nodes.get_mut(&id) {
            node.view = view;
        } else {
            self.nodes.insert(
                id,
                NVTNode {
                    view,
                    superview: None,
                    subviews: Vec::new(),
                    layout: None,
                },
            );
        }
        Ok(())
    }

    fn replace_view(&mut self, id: ViewId, view: NativeView) -> Result<(), PatchError> {
        self.remove_view(id)?;
        self.update_view(id, view)
    }

    /// Does not remove the view from the superview’s subviews list.
    fn remove_view(&mut self, id: ViewId) -> Result<(), PatchError> {
        if let Some(node) = self.nodes.remove(&id) {
            for id in node.subviews {
                self.remove_view(id)?;
            }
            Ok(())
        } else {
            Err(PatchError::NoSuchView(id))
        }
    }

    fn set_subviews(&mut self, id: ViewId, subviews: Vec<ViewId>) -> Result<(), PatchError> {
        for subview in &subviews {
            let node = match self.nodes.get_mut(subview) {
                Some(node) => node,
                None => return Err(PatchError::NoSuchView(*subview)),
            };
            node.superview = Some(id);
        }

        let node = match self.nodes.get_mut(&id) {
            Some(node) => node,
            None => return Err(PatchError::NoSuchView(id)),
        };
        node.subviews = subviews;
        Ok(())
    }
}
