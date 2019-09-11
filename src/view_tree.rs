use crate::view::{Fragment, State, View, ViewId};
use std::collections::HashMap;
use std::sync::Arc;

struct Subregion {
    pos: usize,
    len: usize,
}

/// A node in the view tree.
struct TreeNode {
    /// The current view object.
    view: Arc<dyn View>,
    /// If true, this view is a native view.
    is_native: bool,
    /// The immediate superview.
    superview: Option<ViewId>,
    /// The closest native ancestor view.
    nv_ancestor: Option<ViewId>,
    /// Subregion in the native ancestor’s subviews.
    nv_subregion: Subregion,
    /// The view state.
    state: Box<dyn State>,
    /// An ordered list of all subviews.
    subviews: Vec<ViewId>,
}

// TODO: context

/// A view tree; contains a hierarchy of virtual views and manages rendering and updating.
pub struct ViewTree {
    nodes: HashMap<ViewId, TreeNode>,
    root: ViewId,
}

impl ViewTree {
    /// Diffs a view with its current state in the tree.
    ///
    /// - `id`: the view id, for identifying the tree node
    /// - `view`: the new view
    /// - `nv_subregion_start`: the start index for the NV subregion for this view
    ///
    /// Returns native view IDs that belong to this view.
    fn diff(&mut self, id: ViewId, view: &Arc<dyn View>, nv_subregion_start: usize) -> Vec<ViewId> {
        if let Some(node) = self.nodes.get(&id) {
            let mut is_same_type = node.view.as_any().type_id() == view.as_any().type_id();
            if is_same_type {
                // allow proxy views to complain if they’re not actually the same type
                if !node.view.is_same_type(&**view) {
                    is_same_type = false;
                } else {
                    // same type; can be diffed
                    if !node.view.eq(&**view) {
                        self.update_view(id, view);
                    }
                }
            }

            if !is_same_type {
                // different type; needs to be replaced
                self.replace_view(id, view, nv_subregion_start);
            }
        } else {
            // does not exist; needs to be added
            self.add_view(id, view, nv_subregion_start);
        }

        // render the node’s body
        let node = self.nodes.get_mut(&id).unwrap();
        let body = node.view.body(&node.state);
        let subview_subregion_start = if node.is_native {
            0
        } else {
            nv_subregion_start
        };
        let subviews = self.diff_subviews(id, body, subview_subregion_start);

        let node = self.nodes.get_mut(&id).unwrap();
        node.nv_subregion.pos = nv_subregion_start;
        if node.is_native {
            // native views take up exactly one space
            node.nv_subregion.len = 1;
            vec![id]
        } else {
            // all other views are composite views and take up as much space as their contents
            node.nv_subregion.len = subviews.len();
            subviews
        }
    }

    /// Adds a new view to the tree.
    fn add_view(&mut self, id: ViewId, view: &Arc<dyn View>, nv_subregion_start: usize) {
        let is_native = view.native_type().is_some();
        let state = view.new_state();

        if is_native {
            // TODO: emit patch
        }

        self.nodes.insert(
            id,
            TreeNode {
                view: Arc::clone(view),
                is_native,
                superview: None,
                nv_ancestor: None,
                nv_subregion: Subregion {
                    pos: nv_subregion_start,
                    len: 0,
                },
                state,
                subviews: Vec::new(),
            },
        );
    }

    /// Removes a view and its subviews.
    ///
    /// Does *not* remove the view from the superview’s `subviews` list. The view must exist.
    fn remove_view(&mut self, id: ViewId) {
        let node = self.nodes.remove(&id).expect("removing nonexistent view");
        for subview in node.subviews {
            self.remove_view(subview);
        }
    }

    /// Replaces a view with another of a different type.
    ///
    /// The view must exist.
    fn replace_view(&mut self, id: ViewId, view: &Arc<dyn View>, nv_subregion_start: usize) {
        let current = self.nodes.get(&id).expect("replacing nonexistent view");
        let superview = current.superview;
        let nv_ancestor = current.nv_ancestor;

        self.remove_view(id);
        self.add_view(id, view, nv_subregion_start);

        let node = self.nodes.get_mut(&id).unwrap();
        node.superview = superview;
        node.nv_ancestor = nv_ancestor;
    }

    /// Updates an existing view with new properties, which must be of the same type.
    fn update_view(&mut self, id: ViewId, view: &Arc<dyn View>) {
        let node = self.nodes.get_mut(&id).expect("updating nonexistent view");
        debug_assert!(
            node.view.as_any().type_id() == view.as_any().type_id(),
            "update_view called with incorrect type"
        );
        node.state.will_update(&**view);
        node.view = Arc::clone(view);
    }

    /// Diffs the subview/the subviews of a node and returns the NV ids.
    fn diff_subviews(
        &mut self,
        superview: ViewId,
        subview: Arc<dyn View>,
        nv_subregion_start: usize,
    ) -> Vec<ViewId> {
        let superview_node = &self.nodes[&superview];
        // the closest native ancestor for the subview is either
        let nv_ancestor = if superview_node.is_native {
            // the superview itself
            Some(superview)
        } else {
            // or the superview’s native ancestor
            superview_node.nv_ancestor
        };

        let mut single_subview_storage = Vec::with_capacity(1);
        let subviews = match subview.as_any().downcast_ref::<Fragment>() {
            Some(subviews) => subviews, // list of subviews
            None => match subview.as_any().downcast_ref::<()>() {
                Some(()) => &single_subview_storage, // no subviews at all
                None => {
                    // single subview
                    single_subview_storage.push(subview);
                    &single_subview_storage
                }
            },
        };

        // This will expand an array of subviews as the superview’s (only) subviews.
        // To identify which existing subview and newly rendered subview are meant to be the same,
        // each subview has a key.

        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        enum Key {
            /// A user-specified key.
            Key(u64),
            /// An automatically assigned key.
            AutoKey(u64),
        }

        // If a subview doesn’t have a user-specified key, it’ll be auto-keyed sequentially by
        // index ignoring user-keyed items, e.g.
        //
        // array     [A, B, C(key=1), D(key=2), E]
        // auto-key   0  1                      2

        let mut auto_key_counter = 0;
        let mut current_subviews_by_id = HashMap::new();
        for id in &self.nodes[&superview].subviews {
            let key = self.nodes[&id].view.key().map(Key::Key).unwrap_or_else(|| {
                let k = auto_key_counter;
                auto_key_counter += 1;
                Key::AutoKey(k)
            });
            current_subviews_by_id.insert(key, *id);
        }

        let mut auto_key_counter = 0;
        let mut new_subviews = Vec::new();
        let mut nv_subviews = Vec::new();
        let mut nv_subregion_cursor = nv_subregion_start;

        // TODO: emit patches

        for view in subviews.iter().map(|view| Arc::clone(view)) {
            let key = view.key().map(Key::Key).unwrap_or_else(|| {
                let k = auto_key_counter;
                auto_key_counter += 1;
                Key::AutoKey(k)
            });

            if let Some(subview_id) = current_subviews_by_id.remove(&key) {
                // this new subview already has a corresponding old subview
                let mut nvs = self.diff(subview_id, &view, nv_subregion_cursor);
                nv_subregion_cursor += nvs.len();
                nv_subviews.append(&mut nvs);
                new_subviews.push(subview_id);
            } else {
                // no existing view with the same key, needs to be created
                let subview_id = ViewId::new();

                let mut nvs = self.diff(subview_id, &view, nv_subregion_cursor);
                nv_subregion_cursor += nvs.len();
                nv_subviews.append(&mut nvs);

                let subview_node = self.nodes.get_mut(&subview_id).unwrap();
                subview_node.superview = Some(superview);
                subview_node.nv_ancestor = nv_ancestor;
                new_subviews.push(subview_id);
            };
        }

        // unused subviews need to be removed
        for (_, id) in current_subviews_by_id {
            self.remove_view(id);
        }

        self.nodes.get_mut(&superview).unwrap().subviews = new_subviews;

        nv_subviews
    }
}