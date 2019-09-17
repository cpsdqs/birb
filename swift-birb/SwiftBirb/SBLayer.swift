//
//  SBLayer.swift
//  SwiftBirb
//
//  Created by cpsdqs on 2019-08-13.
//  Copyright © 2019 cpsdqs. All rights reserved.
//

import Cocoa

/// A layer view.
class SBLayer : NSView, SBRenderable {
    unowned let node: SBNode

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    required init(node: SBNode, patch: SBNodePatch) {
        self.node = node

        super.init(frame: NSMakeRect(0, 0, 1, 1))
        translatesAutoresizingMaskIntoConstraints = false
        autoresizesSubviews = false
        layerContentsRedrawPolicy = .onSetNeedsDisplay

        wantsLayer = true
        // add a hosted layer
        layer = CALayer()
        // disable default animations
        layer!.actions = ["content":NSNull()]

        update(patch)
    }

    @available(macOS 10.12.2, *)
    override func makeTouchBar() -> NSTouchBar? {
        // TODO: hook up to callback
        return nil
    }

    func updateContentsScale() {
        layer!.contentsScale = window?.deepestScreen?.backingScaleFactor ?? 1
    }

    override func viewDidMoveToWindow() {
        updateContentsScale()
    }

    // MARK: SBRenderable

    func id() -> ViewId {
        node.id
    }

    func update(_ patch: SBNodePatch) {
        assert(patch.type == SBNodeTypeLayer, "incorrect patch type")
        let data = patch.patch.layer

        let layer = self.layer!
        layer.bounds = data.bounds.cgRect
        layer.backgroundColor = data.background.cgColor
        layer.cornerRadius = CGFloat(data.corner_radius)
        if #available(macOS 10.15, *) {
            layer.cornerCurve = .continuous
        }
        layer.borderWidth = CGFloat(data.border_width)
        layer.borderColor = data.border_color.cgColor
        layer.masksToBounds = data.clip_contents
        layer.transform = data.transform.caTransform3D
        layer.opacity = Float(data.opacity)
    }

    func addSubview(_ subview: SBNode) {
        if let view = subview.view as? NSView {
            addSubview(view)
        } else {
            NSLog("Warning: ignoring non-view subview of layer")
        }
    }

    func removeSubview(_ subview: SBNode) {
        let subviewView = subview.view as? NSView
        if let index = subviews.firstIndex(where: { $0 == subviewView }) {
            subviews.remove(at: index)
        } else {
            NSLog("Warning: found no subview here with ID \(subview.id)")
        }
    }

    func removeSelf() {
        // nothing to do
    }
}
