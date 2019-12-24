//
//  SBNode.swift
//  SwiftBirb
//
//  Created by cpsdqs on 2019-08-13.
//  Copyright © 2019 cpsdqs. All rights reserved.
//

import Cocoa

/// A single birb tree node.
@objc public class SBNode : NSObject {
    unowned let host: SBHostingView
    var type: SBNodeType
    var view: SBRenderable
    var parent: SBNode?
    var subviews: [SBNode] = []

    init(host: SBHostingView, patch: SBNodePatch) {
        self.host = host
        type = patch.type
        view = PropertySelfDotViewNotInitializedAtSuperDotInitCallSigh()
        super.init()
        view = makeView(patch: patch)
    }

    /// Updates this node with a patch.
    @objc public func update(patch: SBNodePatch) {
        view.update(patch)
    }

    @objc public func replace(patch: SBNodePatch) {
        type = patch.type
        view = makeView(patch: patch)
    }

    @objc public func setSubviews(offset: UInt64, length: UInt64, subviews: SBNodeList) {
        let subviews = subviews.intoList()
        let a = Int(offset)
        let b = Int(length)

        self.subviews[a...(a + b)] = subviews[...]

        // TODO: need to call addSubview/removeSubview on SBRenderable?
    }

    /// Removes this node.
    @objc func remove() {
        view.removeSelf()
    }

    /// Creates the actual renderable given a patch. Used when creating/replacing.
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
    init(node: SBNode, patch: SBNodePatch)
    var node: SBNode { get }
    func update(_ patch: SBNodePatch)
    func addSubview(_ subview: SBNode)
    func removeSubview(_ subview: SBNode)
    func removeSelf()
}

/// sigh
/// why yes indeed SBNode.view could be an unconditional unwrapping type, but the only place we’d actually need this is during init and hence would be kind of a dumb thing to have
struct PropertySelfDotViewNotInitializedAtSuperDotInitCallSigh : SBRenderable {
    init() {}
    init(node: SBNode, patch: SBNodePatch) {}
    var node: SBNode {
        get {
            fatalError()
        }
    }
    func update(_ patch: SBNodePatch) {}
    func addSubview(_ subview: SBNode) {}
    func removeSubview(_ subview: SBNode) {}
    func removeSelf() {}
}
