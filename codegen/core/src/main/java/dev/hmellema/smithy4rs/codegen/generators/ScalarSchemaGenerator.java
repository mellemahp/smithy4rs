/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.generators;

import dev.hmellema.smithy4rs.codegen.CodeGenerationContext;
import dev.hmellema.smithy4rs.codegen.RustCodegenSettings;
import dev.hmellema.smithy4rs.codegen.RustSymbolProvider;
import java.util.function.Consumer;
import software.amazon.smithy.codegen.core.directed.CustomizeDirective;
import software.amazon.smithy.model.loader.Prelude;
import software.amazon.smithy.model.shapes.ShapeType;
import software.amazon.smithy.utils.CaseUtils;
import software.amazon.smithy.utils.StringUtils;

public class ScalarSchemaGenerator implements Consumer<CustomizeDirective<CodeGenerationContext, RustCodegenSettings>> {
    private static final String TEMPLATE = """
            smithy!(${id:S}: {
                ${type:L} ${shape:I}
            });
            """;

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
                    for (var shape : shapes) {
                        writer.pushState();
                        writer.putContext("shape", directive.symbolProvider().toSymbol(shape));
                        writer.putContext("id", shape.toShapeId());
                        writer.putContext("type", getSchemaType(shape.getType()));
                        // TODO: Add traits
                        writer.write(TEMPLATE);
                        writer.popState();
                    }
                });
    }

    private static String getSchemaType(ShapeType type) {
        return CaseUtils.toCamelCase(StringUtils.lowerCase(type.toString()));
    }
}
