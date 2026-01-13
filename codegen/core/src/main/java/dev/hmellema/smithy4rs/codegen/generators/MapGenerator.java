/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.generators;

import dev.hmellema.smithy4rs.codegen.CodeGenerationContext;
import dev.hmellema.smithy4rs.codegen.RustCodegenSettings;
import dev.hmellema.smithy4rs.codegen.sections.SchemaSection;
import dev.hmellema.smithy4rs.codegen.symbols.Smithy4Rs;
import java.util.function.Consumer;
import software.amazon.smithy.codegen.core.directed.GenerateMapDirective;

/**
 * Generates serializers and deserializers for Map shapes.
 */
public final class MapGenerator
        implements Consumer<GenerateMapDirective<CodeGenerationContext, RustCodegenSettings>> {
    private static final String BODY_TEMPLATE = """
            map ${shape:I} {${?hasKeyTraits}
                ${keyTraits:C}${/hasKeyTraits}
                key: ${key:I}${?hasValueTraits}
                ${valueTraits:C}${/hasValueTraits}
                value: ${value:I}
            }""";

    @Override
    public void accept(GenerateMapDirective<CodeGenerationContext, RustCodegenSettings> directive) {
        directive.context()
                .writerDelegator()
                .useShapeWriter(directive.shape(), writer -> {
                    writer.pushState();
                    writer.putContext("smithy", Smithy4Rs.SMITHY_MACRO);
                    writer.putContext("id", directive.shape().getId());
                    writer.openBlock("${smithy:T}!(${id:S}: {", "});", () -> {
                        writer.pushState(new SchemaSection(directive.shape()));
                        writer.putContext("key", directive.symbolProvider().toSymbol(directive.shape().getKey()));
                        writer.putContext("value", directive.symbolProvider().toSymbol(directive.shape().getValue()));
                        writer.putContext("shape", directive.symbolProvider().toSymbol(directive.shape()));
                        // Add top-level traits
                        if (TraitInitializerGenerator.hasTraits(directive.shape())) {
                            writer.write("$C",
                                    new TraitInitializerGenerator(writer,
                                            directive.shape(),
                                            directive.context()));
                        }
                        // Add key traits
                        writer.putContext("hasKeyTraits",
                                TraitInitializerGenerator.hasTraits(directive.shape().getKey()));
                        writer.putContext("keyTraits",
                                new TraitInitializerGenerator(writer,
                                        directive.shape().getKey(),
                                        directive.context()));
                        // Add value traits
                        writer.putContext("hasValueTraits",
                                TraitInitializerGenerator.hasTraits(directive.shape().getValue()));
                        writer.putContext("valueTraits",
                                new TraitInitializerGenerator(writer,
                                        directive.shape().getValue(),
                                        directive.context()));
                        // Write schema body
                        writer.write(BODY_TEMPLATE);
                        writer.popState();
                    });
                    writer.popState();
                });
    }
}
