//
//  SBNode.swift
//  SwiftBirb
//
//  Created by cpsdqs on 2019-08-13.
//  Copyright © 2019 cpsdqs. All rights reserved.
//

import Cocoa

/// A single birb tree node.
class SBNode {
    let id: ViewId
    unowned let host: SBHostingView
    var type: SBNodeType
    var firstRender = true
    var view: SBRenderable!
    var parent: SBNode?
    var subviews: Set<ViewId> = Set()

    init(host: SBHostingView, id: ViewId, patch: SBNodePatch) {
        self.host = host
        self.id = id
        type = patch.type
        update(patch)
    }

    /// Updates this node with a patch.
    func update(_ patch: SBNodePatch) {
        if patch.type != type || firstRender {
            if firstRender {
                firstRender = false
            } else {
                // TODO: view.destroy() or something
            }

            type = patch.type
            view = makeView(patch: patch)
        }
    }

    func addSubview(_ node: SBNode) {
        node.parent = self
        subviews.insert(node.id)
        view.addSubview(node)
    }

    func removeSubview(_ node: SBNode) {
        node.parent = nil
        subviews.remove(node.id)
        view.removeSubview(node)
    }

    /// Removes this node.
    func remove() {
        if let parent = self.parent {
            parent.removeSubview(self)
        }
        view.removeSelf()
    }

    func emitEvent(_ type: SBEventTypeId, _ timestamp: Double, _ data: SBEventData) {
        let handler = SBHandlerId(view: id.asRaw(), type: type);

        let event = SBEvent(type: type, handler: handler, timestamp: timestamp, data: data);
        host.dispatch(event: event);
    }

    private func makeView(patch: SBNodePatch) -> SBRenderable {
        switch (patch.type) {
        case SBNodeTypeLayer:
            return SBLayer(node: self, patch: patch)
        default:
            fatalError("Unknown patch node type \(type)")
        }
    }
}

protocol SBRenderable {
    init(node: SBNode, patch: SBNodePatch);
    func id() -> ViewId;
    func update(_ patch: SBNodePatch);
    func addSubview(_ subview: SBNode);
    func removeSubview(_ subview: SBNode);
    func removeSelf();
}
