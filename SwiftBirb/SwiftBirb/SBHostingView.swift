//
//  SBHostingView.swift
//  SwiftBirb
//
//  Created by cpsdqs on 2019-08-13.
//  Copyright © 2019 cpsdqs. All rights reserved.
//

import Cocoa

/// A view that hosts a birb view hierarchy.
@objc public class SBHostingView : NSView {
    let dispatcher: SBEventDispatcher
    var userData: size_t
    var views: [ViewId:SBNode] = [:]

    @objc public init(dispatcher: @escaping SBEventDispatcher, userData: size_t) {
        self.dispatcher = dispatcher
        self.userData = userData
        super.init(frame: NSMakeRect(0, 0, 0, 0))
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func dispatch(event: SBEvent) {
        dispatcher(event, userData)
    }

    @objc public func getUserData() -> size_t {
        return userData
    }

    /// Applies a patch.
    @objc public func patch(_ patch: SBPatch) {
        let view = ViewId(patch.view)
        switch (patch.type) {
        case SBPatchTypeUpdate:
            update(view, patch.data.update)
            break;
        case SBPatchTypeSubview:
            subview(view, patch.data.subview)
        case SBPatchTypeRemove:
            remove(view)
        default:
            fatalError("Unknown patch type \(patch.type)")
        }
    }

    func update(_ view: ViewId, _ patch: SBNodePatch) {
        if let node = views[view] {
            node.update(patch)
        } else {
            views[view] = SBNode(host: self, id: view, patch: patch)
        }
    }

    func subview(_ view: ViewId, _ cSubview: SBViewId) {
        let subview = ViewId(cSubview)

        if let node = views[view], let subviewNode = views[subview] {
            node.addSubview(subviewNode)
        }
    }

    func remove(_ view: ViewId) {
        if let node = views[view] {
            let subviews = node.subviews
            node.remove()
            for subview in subviews {
                remove(subview)
            }
            views.removeValue(forKey: view)
        }
    }
}
