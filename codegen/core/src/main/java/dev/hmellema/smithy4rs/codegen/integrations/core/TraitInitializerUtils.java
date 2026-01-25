package dev.hmellema.smithy4rs.codegen.integrations.core;

import software.amazon.smithy.codegen.core.Symbol;
import software.amazon.smithy.model.traits.Trait;

final class TraitInitializerUtils {
    private TraitInitializerUtils() {}

    /**
     * Get the symbol corresponding to a prelude trait.
     *
     * @param trait trait to get symbol for
     * @return symbol
     */
    static Symbol preludeTrait(Class<? extends Trait> trait) {
        return Symbol.builder()
                .name(trait.getSimpleName())
                .namespace("smithy4rs_core::prelude", "::")
                .build();
    }
}
