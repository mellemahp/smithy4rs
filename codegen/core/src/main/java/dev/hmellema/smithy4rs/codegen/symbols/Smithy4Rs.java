/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.symbols;

import software.amazon.smithy.codegen.core.Symbol;

public class Smithy4Rs {
    private Smithy4Rs() {
        /* Utility class */
    }
    private static final String SMITHY4RS_CORE = "smithy4rs_core";
    private static final String SMITHY4RS_DERIVE = "smithy4rs_core::derive";

    public static final Symbol SMITHY_MACRO = Symbol.builder()
            .name("smithy")
            .namespace(SMITHY4RS_CORE, "::")
            .build();
    public static final Symbol DOC_MAP_MACRO = Symbol.builder()
            .name("doc_map")
            .namespace(SMITHY4RS_CORE, "::")
            .build();
    public static final Symbol SMITHY_ENUM = Symbol.builder()
            .name("smithy_enum")
            .namespace(SMITHY4RS_DERIVE, "::")
            .build();
    public static final Symbol SHAPE_DERIVE = Symbol.builder()
            .name("SmithyShape")
            .namespace(SMITHY4RS_DERIVE, "::")
            .build();
    public static final Symbol UNION_MACRO = Symbol.builder()
            .name("smithy_union")
            .namespace(SMITHY4RS_DERIVE, "::")
            .build();
}
