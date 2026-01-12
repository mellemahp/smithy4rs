/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.generators;

import dev.hmellema.smithy4rs.codegen.CodeGenerationContext;
import dev.hmellema.smithy4rs.codegen.RustCodegenSettings;
import dev.hmellema.smithy4rs.codegen.symbols.Smithy4Rs;
import java.util.List;
import java.util.function.Consumer;
import software.amazon.smithy.codegen.core.directed.GenerateMapDirective;

/**
 * Generates serializers and deserializers for Map shapes.
 */
public final class MapGenerator
        implements Consumer<GenerateMapDirective<CodeGenerationContext, RustCodegenSettings>> {

    // TODO: Traits!
    private static final String TEMPLATE = """
            ${smithy:T}!(${id:S}: {
                map ${shape:I} {
                    key: ${key:I}
                    value: ${value:I}
                }
            });
            """;

    @Override
    public void accept(GenerateMapDirective<CodeGenerationContext, RustCodegenSettings> directive) {
        directive.context()
                .writerDelegator()
                .useShapeWriter(directive.shape(), writer -> {
                    var map = directive.symbolProvider().toSymbol(directive.shape());
                    var key = directive.symbolProvider().toSymbol(directive.shape().getKey());
                    var value = directive.symbolProvider().toSymbol(directive.shape().getValue());

                    // TODO(codegen): Add sections
                    writer.pushState();
                    writer.putContext("smithy", Smithy4Rs.SMITHY_MACRO);
                    writer.putContext("traits", List.of());
                    writer.putContext("id", directive.shape().getId());
                    writer.putContext("key", key);
                    writer.putContext("value", value);
                    writer.putContext("shape", map);
                    writer.write(TEMPLATE);
                    writer.popState();
                });
    }
}
