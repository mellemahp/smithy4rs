/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.symbols;

import software.amazon.smithy.codegen.core.Symbol;

/**
 * Rust standard library types.
 */
public class StdLib {
    private static final String DELIM = "::";

    private StdLib() {
        /* Utility class */
    }
    public static final Symbol VEC = Symbol.builder()
            .name("Vec")
            .namespace("std", "::")
            .build();
    public static final Symbol STRING = Symbol.builder()
            .name("String")
            .namespace("std", "::")
            .build();
    public static final Symbol BOX = Symbol.builder()
            .name("Box")
            .namespace("std", "::")
            .build();
    public static final Symbol PARTIAL_EQ_DERIVE = Symbol.builder()
            .name("PartialEq")
            .namespace("core::cmp", DELIM)
            .build();
    public static final Symbol CLONE_DERIVE = Symbol.builder()
            .name("Clone")
            .namespace("core::clone", DELIM)
            .build();
}
