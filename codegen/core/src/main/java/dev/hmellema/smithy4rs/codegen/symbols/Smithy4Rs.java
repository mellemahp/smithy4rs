/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.symbols;

import dev.hmellema.smithy4rs.codegen.SymbolProperties;
import software.amazon.smithy.codegen.core.Symbol;

public class Smithy4Rs {
    public static final Symbol SMITHY_MACRO = Symbol.builder()
            .name("smithy")
            .namespace("smithy4rs_core", "::")
            .putProperty(SymbolProperties.IS_MACRO, true)
            .build();

}
