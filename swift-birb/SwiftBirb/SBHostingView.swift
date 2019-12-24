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
    @objc public init() {
        super.init(frame: NSMakeRect(0, 0, 0, 0))
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    @objc public func createView(_ patch: SBNodePatch) -> SBNode {
        return SBNode(host: self, patch: patch)
    }

    @objc public func setRootView(_ view: SBNode) {
        // TODO: this
    }
}
