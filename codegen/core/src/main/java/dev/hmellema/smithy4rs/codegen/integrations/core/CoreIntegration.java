/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.integrations.core;

import dev.hmellema.smithy4rs.codegen.RustCodegenIntegration;
import dev.hmellema.smithy4rs.codegen.TraitInitializer;
import java.util.List;
import java.util.Map;
import java.util.stream.Collectors;
import java.util.stream.Stream;
import software.amazon.smithy.codegen.core.Symbol;
import software.amazon.smithy.model.shapes.ShapeId;
import software.amazon.smithy.model.traits.*;
import software.amazon.smithy.utils.SmithyInternalApi;

/**
 * Core functionality for Rust code generation plugins.
 *
 * <p>This integration registers basic trait initializers and mappings for prelude traits.
 */
@SmithyInternalApi
public final class CoreIntegration implements RustCodegenIntegration {
    @Override
    public List<TraitInitializer<? extends Trait>> traitInitializers() {
        return List.of(
                new AnnotationTraitInitializer(),
                new StringTraitInitializer(),
                new GenericTraitInitializer());
    }

    @Override
    public Map<ShapeId, Symbol> traitMappings() {
        return Stream.of(
                // Validation Traits
                LengthTrait.ID,
                PatternTrait.ID,
                RangeTrait.ID,
                RequiredTrait.ID,
                SensitiveTrait.ID,
                SparseTrait.ID,
                UniqueItemsTrait.ID,
                RequiresLengthTrait.ID,
                ErrorTrait.ID,
                DefaultTrait.ID,
                // Base Prelude Protocol traits
                JsonNameTrait.ID,
                TimestampFormatTrait.ID,
                MediaTypeTrait.ID,
                XmlNameTrait.ID,
                XmlFlattenedTrait.ID,
                XmlAttributeTrait.ID,
                XmlNamespaceTrait.ID,
                EventHeaderTrait.ID,
                EventPayloadTrait.ID,
                HostLabelTrait.ID,
                EndpointTrait.ID,
                // Prelude behavior traits
                PaginatedTrait.ID,
                IdempotencyTokenTrait.ID,
                RetryableTrait.ID,
                RequestCompressionTrait.ID,
                StreamingTrait.ID
        ).collect(Collectors.toMap(i -> i, CoreIntegration::preludeTrait));
    }

    @Override
    public String name() {
        return "rust-core";
    }

    @Override
    public byte priority() {
        return -1;
    }

    private static Symbol preludeTrait(ShapeId traitId) {
        return Symbol.builder()
                .name(traitId.getName())
                .namespace("smithy4rs_core::prelude", "::")
                .build();
    }
}
