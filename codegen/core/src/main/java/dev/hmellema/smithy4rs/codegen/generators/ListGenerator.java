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
import software.amazon.smithy.codegen.core.directed.GenerateListDirective;

/**
 * Generates a schema definition for a List shapes.
 *
 * <p><strong>NOTE</strong>: Lists ONLY generate a schema.
 */
public final class ListGenerator
        implements Consumer<GenerateListDirective<CodeGenerationContext, RustCodegenSettings>> {

    // TODO(traits): Add traits to schema
    private static final String TEMPLATE = """


            });
            """;

    @Override
    public void accept(GenerateListDirective<CodeGenerationContext, RustCodegenSettings> directive) {
        directive.context()
                .writerDelegator()
                .useShapeWriter(directive.shape(), writer -> {
                    writer.pushState();
                    writer.putContext("smithy", Smithy4Rs.SMITHY_MACRO);
                    writer.putContext("id", directive.shape().getId());
                    writer.openBlock("${smithy:T}!(${id:S}: {", "});", () -> {
                        writer.pushState(new SchemaSection(directive.shape()));
                        writer.putContext("shape", directive.symbolProvider().toSymbol(directive.shape()));
                        writer.putContext("member", directive.symbolProvider().toSymbol(directive.shape().getMember()));
                        writer.write("""
                                list ${shape:I} {
                                    member: ${member:I}
                                }""");
                        writer.popState();
                    });
                    writer.popState();
                });
    }
}
