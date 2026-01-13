/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.integrations.core;

import dev.hmellema.smithy4rs.codegen.CodeGenerationContext;
import dev.hmellema.smithy4rs.codegen.TraitInitializer;
import dev.hmellema.smithy4rs.codegen.writer.RustWriter;
import software.amazon.smithy.model.traits.StringTrait;

final class StringTraitInitializer implements TraitInitializer<StringTrait> {
    @Override
    public Class<StringTrait> traitClass() {
        return StringTrait.class;
    }

    @Override
    public void write(RustWriter writer, CodeGenerationContext context, StringTrait trait) {
        var mapping = context.traitMapping(trait);
        writer.writeInline("$T::new($S)", mapping, trait.getValue());
    }
}
