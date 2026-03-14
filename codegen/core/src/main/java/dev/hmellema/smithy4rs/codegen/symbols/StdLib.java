/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.symbols;

import software.amazon.smithy.codegen.core.Symbol;
import software.amazon.smithy.utils.SmithyInternalApi;

@SmithyInternalApi
public final class StdLib {
    private static final String DELIM = "::";

    private StdLib() {
        /* Utility Class */
    }

    public static final Symbol PARTIAL_EQ_DERIVE = Symbol.builder()
            .name("PartialEq")
            .namespace("core::cmp", DELIM)
            .build();

    public static final Symbol CLONE_DERIVE = Symbol.builder()
            .name("Clone")
            .namespace("core::clone", DELIM)
            .build();
}
