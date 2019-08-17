//
//  ViewId.swift
//  SwiftBirb
//
//  Created by cpsdqs on 2019-08-13.
//  Copyright © 2019 cpsdqs. All rights reserved.
//

import Cocoa

struct ViewId : Hashable {
    let a: UInt32
    let b: UInt16
    let c: UInt16
    let d0: UInt8
    let d1: UInt8
    let d2: UInt8
    let d3: UInt8
    let d4: UInt8
    let d5: UInt8
    let d6: UInt8
    let d7: UInt8

    init(_ bvi: SBViewId) {
        a = bvi.a
        b = bvi.b
        c = bvi.c
        d0 = bvi.d.0
        d1 = bvi.d.1
        d2 = bvi.d.2
        d3 = bvi.d.3
        d4 = bvi.d.4
        d5 = bvi.d.5
        d6 = bvi.d.6
        d7 = bvi.d.7
    }

    func asRaw() -> SBViewId {
        let d = (d0, d1, d2, d3, d4, d5, d6, d7)
        return SBViewId(a: a, b: b, c: c, d: d)
    }
}
