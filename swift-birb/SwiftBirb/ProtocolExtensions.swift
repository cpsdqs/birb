//
//  CoreExtensions.swift
//  SwiftBirb
//
//  Created by cpsdqs on 2019-08-14.
//  Copyright © 2019 cpsdqs. All rights reserved.
//

import Cocoa

let sRGB = CGColorSpace(name: CGColorSpace.sRGB)!

extension SBColor {
    var cgColor: CGColor {
        get {
            let components = UnsafeMutablePointer<CGFloat>.allocate(capacity: 4)
            components[0] = CGFloat(r)
            components[1] = CGFloat(g)
            components[2] = CGFloat(b)
            components[3] = CGFloat(a)
            return CGColor(colorSpace: sRGB, components: components)!
        }
    }
}

extension SBRect {
    var cgRect: CGRect {
        get {
            CGRect(x: CGFloat(origin.x), y: CGFloat(origin.y), width: CGFloat(size.x), height: CGFloat(size.y))
        }
    }
}

extension SBMatrix3 {
    var caTransform3D: CATransform3D {
        get {
            CATransform3D(m11: CGFloat(m00), m12: CGFloat(m01), m13: CGFloat(m02), m14: 0, m21: CGFloat(m10), m22: CGFloat(m11), m23: CGFloat(m12), m24: 0, m31: CGFloat(m20), m32: CGFloat(m21), m33: CGFloat(m22), m34: 0, m41: 0, m42: 0, m43: 0, m44: 1)
        }
    }
}

extension SBNodeList {
    /// Converts this into a Swift list and deallocates the memory.
    ///
    /// Safety: once this function has been called, the SBNodeList must not be used, because it would access deallocated memory.
    func intoList() -> [SBNode] {
        var list: [SBNode] = []
        for i in 0...count {
            let node = nodes.assumingMemoryBound(to: SBNode.self).advanced(by: Int(i)).pointee
            list.append(node)
        }
        nodes.deallocate()
        return list
    }
}
