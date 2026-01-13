/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen;

import dev.hmellema.smithy4rs.codegen.writer.RustWriter;
import java.util.List;
import java.util.Map;
import software.amazon.smithy.codegen.core.SmithyIntegration;
import software.amazon.smithy.codegen.core.Symbol;
import software.amazon.smithy.model.shapes.ShapeId;
import software.amazon.smithy.model.traits.Trait;

/**
 * Java SPI for customizing Rust code generation.
 */
public interface RustCodegenIntegration
        extends SmithyIntegration<RustCodegenSettings, RustWriter, CodeGenerationContext> {
    /**
     * List of {@link TraitInitializer}'s to use when writing traits in Schema definitions.
     */
    default List<TraitInitializer<? extends Trait>> traitInitializers() {
        return List.of();
    }

    /**
     * ShapeId -> Symbol mappings for Traits.
     */
    default Map<ShapeId, Symbol> traitMappings() {
        return Map.of();
    }
}
