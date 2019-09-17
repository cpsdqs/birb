//
//  SwiftBirb.m
//  SwiftBirb
//
//  Created by cpsdqs on 2019-08-13.
//  Copyright © 2019 cpsdqs. All rights reserved.
//

#import <objc/runtime.h>
#import "SwiftBirb-Swift.h"

static_assert(sizeof(float64_t) == 8, "float64_t has unexpected size");

// This method is required because Swift mangles symbol names
id SBHostingView_getClass(void) {
    return [SBHostingView class];
}
