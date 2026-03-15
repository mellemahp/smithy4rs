/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.integrations.core;

import dev.hmellema.smithy4rs.codegen.Utils;
import software.amazon.smithy.codegen.core.Symbol;
import software.amazon.smithy.model.traits.Trait;

final class TraitInitializerUtils {
    private static final String PRELUDE_MODULE = "smithy4rs_core::prelude";
    private static final String DELIM = "::";

    private TraitInitializerUtils() {}

    /**
     * Get the symbol corresponding to a prelude trait.
     *
     * @param trait trait to get symbol for
     * @return symbol
     */
    public static Symbol preludeTrait(Class<? extends Trait> trait) {
        return Symbol.builder()
                // We use the class name here rather than the trait ID name
                .name(trait.getSimpleName())
                .namespace(module(), DELIM)
                .build();
    }

    private static String module() {
        return Utils.inCore() ? "local" : PRELUDE_MODULE;
    }
}
