/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.symbols;

import dev.hmellema.smithy4rs.codegen.Utils;
import software.amazon.smithy.codegen.core.Symbol;
import software.amazon.smithy.utils.SmithyInternalApi;

@SmithyInternalApi
public final class Smithy4Rs {
    private Smithy4Rs() {
        /* Utility class */
    }

    private static String derive() {
        return String.format("%s::derive", Utils.crateIdent());
    }

    public static final Symbol SMITHY_MACRO = Symbol.builder()
            .name("smithy")
            .namespace(Utils.crateIdent(), Utils.DELIM)
            .build();
    public static final Symbol DOC_MAP_MACRO = Symbol.builder()
            .name("doc_map")
            .namespace(Utils.crateIdent(), Utils.DELIM)
            .build();
    public static final Symbol SMITHY_ENUM = Symbol.builder()
            .name("smithy_enum")
            .namespace(derive(), Utils.DELIM)
            .build();
    public static final Symbol SHAPE_DERIVE = Symbol.builder()
            .name("SmithyShape")
            .namespace(derive(), Utils.DELIM)
            .build();
    public static final Symbol UNION_MACRO = Symbol.builder()
            .name("smithy_union")
            .namespace(derive(), Utils.DELIM)
            .build();
    public static final Symbol TRAIT_DERIVE = Symbol.builder()
            .name("SmithyTraitImpl")
            .namespace(derive(), Utils.DELIM)
            .build();
}
