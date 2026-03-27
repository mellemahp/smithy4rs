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

    private static String schema() {
        return String.format("%s::schema", Utils.crateIdent());
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
    public static final Symbol INDEX_MAP = Symbol.builder()
            .name("IndexMap")
            .namespace(Utils.crateIdent(), "::")
            .build();
    public static final Symbol BYTE_BUFFER = Symbol.builder()
            .name("ByteBuffer")
            .namespace(Utils.crateIdent(), "::")
            .build();
    public static final Symbol BIG_INT = Symbol.builder()
            .name("BigInt")
            .namespace(Utils.crateIdent(), "::")
            .build();
    public static final Symbol BIG_DECIMAL = Symbol.builder()
            .name("BigDecimal")
            .namespace(Utils.crateIdent(), "::")
            .build();
    public static final Symbol DOCUMENT = Symbol.builder()
            .name("Document")
            .namespace(schema(), "::")
            .build();
}
