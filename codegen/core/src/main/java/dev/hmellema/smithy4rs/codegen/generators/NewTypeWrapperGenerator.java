/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.generators;

import dev.hmellema.smithy4rs.codegen.SymbolProperties;
import dev.hmellema.smithy4rs.codegen.sections.ShapeSection;
import dev.hmellema.smithy4rs.codegen.writer.RustWriter;
import software.amazon.smithy.codegen.core.SymbolProvider;
import software.amazon.smithy.model.shapes.Shape;

/**
 * Creates a wrapper class (sometimes called "new type" pattern)
 */
final class NewTypeWrapperGenerator {
    private static final String TEMPLATE = """
            ${derive:C|}
            #[smithy_schema(${shape:I})]
            #[repr(transparent)]
            pub struct ${shape:T}(${inner:T});
            """;

    public static void generate(RustWriter writer, Shape shape, SymbolProvider symbolProvider) {
        writer.pushState(new ShapeSection(shape));
        var symbol = symbolProvider.toSymbol(shape);
        writer.putContext("shape", symbol);
        writer.putContext("inner", symbol.expectProperty(SymbolProperties.INNER));
        writer.putContext("derive", new DeriveGenerator(writer, shape));
        writer.write(TEMPLATE);
        writer.popState();
    }
}
