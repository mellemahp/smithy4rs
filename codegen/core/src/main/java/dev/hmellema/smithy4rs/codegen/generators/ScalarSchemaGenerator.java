/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.generators;

import dev.hmellema.smithy4rs.codegen.CodeGenerationContext;
import dev.hmellema.smithy4rs.codegen.RustCodegenSettings;
import dev.hmellema.smithy4rs.codegen.RustSymbolProvider;
import dev.hmellema.smithy4rs.codegen.sections.SchemaSection;
import dev.hmellema.smithy4rs.codegen.symbols.Smithy4Rs;
import java.util.function.Consumer;
import software.amazon.smithy.codegen.core.directed.CustomizeDirective;
import software.amazon.smithy.model.loader.Prelude;
import software.amazon.smithy.model.shapes.ShapeType;
import software.amazon.smithy.utils.CaseUtils;
import software.amazon.smithy.utils.StringUtils;

public class ScalarSchemaGenerator implements Consumer<CustomizeDirective<CodeGenerationContext, RustCodegenSettings>> {
    @Override
    public void accept(CustomizeDirective<CodeGenerationContext, RustCodegenSettings> directive) {
        directive.context()
                .writerDelegator()
                .useFileWriter(RustSymbolProvider.FILE, writer -> {
                    var shapes = directive.model()
                            .shapes()
                            .filter(s -> !Prelude.isPreludeShape(s))
                            .filter(s -> !s.getType().isShapeType(ShapeType.ENUM)
                                    && !s.getType().isShapeType(ShapeType.INT_ENUM))
                            .filter(s -> s.getType().getCategory().equals(ShapeType.Category.SIMPLE))
                            .toList();
                    writer.pushState();
                    writer.putContext("smithy", Smithy4Rs.SMITHY_MACRO);
                    for (var shape : shapes) {
                        writer.pushState();
                        writer.putContext("id", shape.getId());
                        writer.openBlock("${smithy:T}!(${id:S}: {", "});", () -> {
                            writer.pushState(new SchemaSection(shape));
                            writer.putContext("type", getSchemaType(shape.getType()));
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

    private static String getSchemaType(ShapeType type) {
        return CaseUtils.toCamelCase(StringUtils.lowerCase(type.toString()));
    }
}
