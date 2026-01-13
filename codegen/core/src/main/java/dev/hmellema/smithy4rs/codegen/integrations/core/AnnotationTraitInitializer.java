/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.integrations.core;

import dev.hmellema.smithy4rs.codegen.CodeGenerationContext;
import dev.hmellema.smithy4rs.codegen.TraitInitializer;
import dev.hmellema.smithy4rs.codegen.writer.RustWriter;
import software.amazon.smithy.model.traits.AnnotationTrait;

/**
 * Generates a simple, empty initializer for an annotation trait.
 */
final class AnnotationTraitInitializer implements TraitInitializer<AnnotationTrait> {

    @Override
    public Class<AnnotationTrait> traitClass() {
        return AnnotationTrait.class;
    }

    @Override
    public void write(RustWriter writer, CodeGenerationContext context, AnnotationTrait trait) {
        var mapping = context.traitMapping(trait);
        writer.writeInline("$T", mapping);
    }
}
