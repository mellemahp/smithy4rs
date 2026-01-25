/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.integrations.core;

import dev.hmellema.smithy4rs.codegen.CodeGenerationContext;
import dev.hmellema.smithy4rs.codegen.TraitInitializer;
import dev.hmellema.smithy4rs.codegen.writer.RustWriter;
import software.amazon.smithy.codegen.core.Symbol;
import software.amazon.smithy.model.traits.LengthTrait;

final class LengthTraitInitializer implements TraitInitializer<LengthTrait> {
    private static final Symbol LENGTH_TRAIT = TraitInitializerUtils.preludeTrait(LengthTrait.class);

    @Override
    public Class<LengthTrait> traitClass() {
        return LengthTrait.class;
    }

    @Override
    public void write(RustWriter writer, CodeGenerationContext context, LengthTrait trait) {
        writer.putContext("min", trait.getMin());
        writer.putContext("max", trait.getMax());
        writer.putContext("length", LENGTH_TRAIT);
        writer.writeInline(
                "${length:T}::builder()${?min}.min(${min:L}L)${/min}${?max}.max(${max:L}L)${/max}.build()");
    }
}
