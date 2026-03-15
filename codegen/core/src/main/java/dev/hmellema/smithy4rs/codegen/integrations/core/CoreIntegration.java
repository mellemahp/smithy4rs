/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.integrations.core;

import static dev.hmellema.smithy4rs.codegen.integrations.core.TraitInitializerUtils.preludeTrait;

import dev.hmellema.smithy4rs.codegen.RustCodegenIntegration;
import dev.hmellema.smithy4rs.codegen.RustCodegenSettings;
import dev.hmellema.smithy4rs.codegen.SymbolProperties;
import dev.hmellema.smithy4rs.codegen.TraitInitializer;
import dev.hmellema.smithy4rs.codegen.Utils;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import software.amazon.smithy.codegen.core.Symbol;
import software.amazon.smithy.codegen.core.SymbolProvider;
import software.amazon.smithy.model.Model;
import software.amazon.smithy.model.shapes.MemberShape;
import software.amazon.smithy.model.shapes.Shape;
import software.amazon.smithy.model.shapes.ShapeId;
import software.amazon.smithy.model.traits.*;
import software.amazon.smithy.utils.SmithyInternalApi;

/**
 * StdLib functionality for Rust code generation plugins.
 *
 * <p>This integration registers basic trait initializers and mappings for prelude traits.
 */
@SmithyInternalApi
public final class CoreIntegration implements RustCodegenIntegration {
    private static final Symbol REGEX_SYMBOL = Symbol.builder()
            .namespace(String.format("%s::schema", Utils.crateIdent()), Utils.DELIM)
            .name("RegexWrapper")
            .build();
    private static final ShapeId PATTERN_ID = ShapeId.from("smithy.api#pattern");

    @Override
    public List<TraitInitializer<? extends Trait>> traitInitializers() {
        return List.of(
                // Prelude initializers,
                new LengthTraitInitializer(),
                new RangeTraitInitializer(),
                // Service traits
                // TODO(service traits): Add initializers for service-level traits
                // Generic initializers (note: must come _after_ all others)
                new AnnotationTraitInitializer(),
                new StringTraitInitializer(),
                new GenericTraitInitializer());
    }

    @Override
    public Map<ShapeId, Symbol> traitMappings() {
        Map<ShapeId, Symbol> result = new HashMap<>();
        // Validation Traits
        result.put(LengthTrait.ID, preludeTrait(LengthTrait.class));
        result.put(PatternTrait.ID, preludeTrait(PatternTrait.class));
        result.put(RangeTrait.ID, preludeTrait(RangeTrait.class));
        result.put(RequiredTrait.ID, preludeTrait(RequiredTrait.class));
        result.put(SensitiveTrait.ID, preludeTrait(SensitiveTrait.class));
        result.put(SparseTrait.ID, preludeTrait(SparseTrait.class));
        result.put(UniqueItemsTrait.ID, preludeTrait(UniqueItemsTrait.class));
        result.put(RequiresLengthTrait.ID, preludeTrait(RequiresLengthTrait.class));
        result.put(ErrorTrait.ID, preludeTrait(ErrorTrait.class));
        result.put(DefaultTrait.ID, preludeTrait(DefaultTrait.class));
        // Base Prelude Protocol traits
        result.put(JsonNameTrait.ID, preludeTrait(JsonNameTrait.class));
        result.put(TimestampFormatTrait.ID, preludeTrait(TimestampFormatTrait.class));
        result.put(MediaTypeTrait.ID, preludeTrait(MediaTypeTrait.class));
        result.put(XmlNameTrait.ID, preludeTrait(XmlNameTrait.class));
        result.put(XmlFlattenedTrait.ID, preludeTrait(XmlFlattenedTrait.class));
        result.put(XmlAttributeTrait.ID, preludeTrait(XmlAttributeTrait.class));
        result.put(XmlNamespaceTrait.ID, preludeTrait(XmlNamespaceTrait.class));
        result.put(EventHeaderTrait.ID, preludeTrait(EventHeaderTrait.class));
        result.put(EventPayloadTrait.ID, preludeTrait(EventPayloadTrait.class));
        result.put(HostLabelTrait.ID, preludeTrait(HostLabelTrait.class));
        result.put(EndpointTrait.ID, preludeTrait(EndpointTrait.class));
        // Prelude behavior traits
        result.put(PaginatedTrait.ID, preludeTrait(PaginatedTrait.class));
        result.put(IdempotencyTokenTrait.ID, preludeTrait(IdempotencyTokenTrait.class));
        result.put(RetryableTrait.ID, preludeTrait(RetryableTrait.class));
        result.put(RequestCompressionTrait.ID, preludeTrait(RequestCompressionTrait.class));
        result.put(StreamingTrait.ID, preludeTrait(StreamingTrait.class));
        return result;
    }

    @Override
    public String name() {
        return "rust-core";
    }

    @Override
    public byte priority() {
        return -1;
    }

    @Override
    public SymbolProvider decorateSymbolProvider(
            Model model,
            RustCodegenSettings settings,
            SymbolProvider symbolProvider
    ) {
        return new SymbolProvider() {
            @Override
            public Symbol toSymbol(Shape shape) {
                var symbol = symbolProvider.toSymbol(shape);
                if (shape.getId().equals(PATTERN_ID)) {
                    // We want to use Regex for pattern inner type
                    return symbol.toBuilder()
                            .putProperty(SymbolProperties.INNER, REGEX_SYMBOL)
                            .build();
                }
                return symbol;
            }

            @Override
            public String toMemberName(MemberShape shape) {
                // Avoid squashing member name impl
                return symbolProvider.toMemberName(shape);
            }
        };
    }
}
