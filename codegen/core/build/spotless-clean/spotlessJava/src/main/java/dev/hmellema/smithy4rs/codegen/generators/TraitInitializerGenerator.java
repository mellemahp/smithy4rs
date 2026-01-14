/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.generators;

import dev.hmellema.smithy4rs.codegen.CodeGenerationContext;
import dev.hmellema.smithy4rs.codegen.writer.RustWriter;
import java.util.List;
import software.amazon.smithy.model.shapes.Shape;
import software.amazon.smithy.model.shapes.ShapeId;
import software.amazon.smithy.model.traits.DefaultTrait;
import software.amazon.smithy.model.traits.DeprecatedTrait;
import software.amazon.smithy.model.traits.DocumentationTrait;
import software.amazon.smithy.model.traits.ExternalDocumentationTrait;
import software.amazon.smithy.model.traits.SinceTrait;
import software.amazon.smithy.model.traits.UnstableTrait;

/**
 * Generates all trait initializers for a given shape.
 */
record TraitInitializerGenerator(RustWriter writer, Shape shape, CodeGenerationContext context) implements
        Runnable {
    // TODO: Make configurable
    private static final List<ShapeId> EXCLUDED_TRAITS = List.of(
            // Documentation Traits.
            DocumentationTrait.ID,
            ExternalDocumentationTrait.ID,
            UnstableTrait.ID,
            DeprecatedTrait.ID,
            SinceTrait.ID,
            // Defaults are handled by rust macros
            DefaultTrait.ID);

    public static boolean hasTraits(Shape shape) {
        return !shape.getAllTraits()
                .keySet()
                .stream()
                .filter(id -> !EXCLUDED_TRAITS.contains(id))
                .toList()
                .isEmpty();
    }

    @Override
    public void run() {
        var traitsToAdd = shape.getAllTraits()
                .keySet()
                .stream()
                .filter(id -> !EXCLUDED_TRAITS.contains(id))
                .toList();
        if (traitsToAdd.isEmpty()) {
            return;
        }
        for (ShapeId shapeId : traitsToAdd) {
            var trait = shape.getAllTraits().get(shapeId);
            writer.pushState();
            writer.writeInline("@");
            context.getInitializer(trait).write(writer, context, trait);
            writer.write(';');
            writer.popState();
        }
    }
}
