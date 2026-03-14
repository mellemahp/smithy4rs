/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.generators;

import dev.hmellema.smithy4rs.codegen.CodeGenerationContext;
import dev.hmellema.smithy4rs.codegen.RustCodegenSettings;
import dev.hmellema.smithy4rs.codegen.RustSymbolProvider;
import dev.hmellema.smithy4rs.codegen.Utils;
import dev.hmellema.smithy4rs.codegen.sections.SchemaSection;
import dev.hmellema.smithy4rs.codegen.symbols.Smithy4Rs;
import java.util.function.Consumer;
import software.amazon.smithy.codegen.core.directed.CustomizeDirective;
import software.amazon.smithy.model.shapes.ShapeType;
import software.amazon.smithy.model.traits.TraitDefinition;

public class ScalarSchemaGenerator implements Consumer<CustomizeDirective<CodeGenerationContext, RustCodegenSettings>> {
    @Override
    public void accept(CustomizeDirective<CodeGenerationContext, RustCodegenSettings> directive) {
        directive.context()
                .writerDelegator()
                .useFileWriter(RustSymbolProvider.FILE, writer -> {
                    var shapes = directive.model()
                            .shapes()
                            .filter(Utils::shouldInclude)
                            .filter(s -> !s.getType().isShapeType(ShapeType.ENUM)
                                    && !s.getType().isShapeType(ShapeType.INT_ENUM))
                            .filter(s -> s.getType().getCategory().equals(ShapeType.Category.SIMPLE))
                            .toList();
                    writer.pushState();
                    writer.putContext("smithy", Smithy4Rs.SMITHY_MACRO);
                    for (var shape : shapes) {
                        // Generate trait definitions if applicable
                        if (shape.hasTrait(TraitDefinition.class)) {
                            NewTypeWrapperGenerator.generate(writer, shape, directive.symbolProvider());
                        }
                        writer.pushState();
                        writer.putContext("id", shape.getId());
                        writer.openBlock("${smithy:T}!(${id:S}: {", "});", () -> {
                            writer.pushState(new SchemaSection(shape));
                            if (TraitInitializerGenerator.hasTraits(shape)) {
                                writer.write("$C", new TraitInitializerGenerator(writer, shape, directive.context()));
                            }
                            writer.putContext("type", shape.getType().toString());
                            writer.putContext("shape", directive.symbolProvider().toSymbol(shape));
                            writer.write("${type:L} ${shape:I}");
                            writer.popState();
                        });
                        writer.popState();
                        // Add a newline for better spacing
                        writer.write("");
                    }
                    writer.popState();
                });
    }
}
