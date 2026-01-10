/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.generators;

import dev.hmellema.smithy4rs.codegen.CodeGenerationContext;
import dev.hmellema.smithy4rs.codegen.RustCodegenSettings;
import dev.hmellema.smithy4rs.codegen.writer.RustWriter;
import java.util.Map;
import java.util.function.Consumer;
import java.util.stream.Collectors;
import software.amazon.smithy.codegen.core.directed.ShapeDirective;
import software.amazon.smithy.model.shapes.EnumShape;
import software.amazon.smithy.model.shapes.IntEnumShape;
import software.amazon.smithy.model.shapes.Shape;
import software.amazon.smithy.utils.CaseUtils;
import software.amazon.smithy.utils.StringUtils;

public final class EnumGenerator<T extends ShapeDirective<Shape, CodeGenerationContext, RustCodegenSettings>>
        implements Consumer<T> {

    private static final String TEMPLATE = """
            smithy!(${id:S}: {
                enum ${shape:I} {${#variants}
                    ${value:C|}${/variants}
                }
            });

            #[smithy_enum]
            #[derive(SmithyShape)]
            #[smithy_schema(${shape:I})]
            pub enum TestEnum {${#variants}
                ${value:C|},${/variants}
            }
            """;
    private static final String STRING_TEMPLATE = "${name:L} = ${value:S}";
    private static final String INT_TEMPLATE = "${name:L} = ${value:L}";

    @Override
    public void accept(T directive) {
        var shape = directive.shape();
        directive.context().writerDelegator().useShapeWriter(shape, writer -> {
            var values = getEnumValues(directive.shape());
            var isIntEnum = isIntEnum(directive.shape());
            var variants = values.entrySet()
                    .stream()
                    .map(entry -> new VariantGenerator(writer, entry.getKey(), entry.getValue(), isIntEnum))
                    .toList();
            writer.pushState();
            writer.putContext("shape", directive.symbolProvider().toSymbol(directive.shape()));
            writer.putContext("id", directive.shape().getId());
            writer.putContext("variants", variants);
            writer.write(TEMPLATE);
            writer.popState();
        });
    }

    private static boolean isIntEnum(Shape shape) {
        return shape instanceof IntEnumShape;
    }

    private static Map<String, String> getEnumValues(Shape shape) {
        if (shape instanceof EnumShape se) {
            return se.getEnumValues();
        } else if (shape instanceof IntEnumShape ie) {
            return ie.getEnumValues()
                    .entrySet()
                    .stream()
                    .collect(Collectors.toMap(Map.Entry::getKey, e -> e.getValue().toString()));
        }
        throw new IllegalArgumentException("Expected Int enum or enum");
    }

    private record VariantGenerator(
            RustWriter writer,
            String name,
            String value,
            boolean intEnum) implements Runnable {
        @Override
        public void run() {
            writer.pushState();
            writer.putContext("name", StringUtils.capitalize(CaseUtils.toPascalCase(name)));
            writer.putContext("value", value);
            if (intEnum) {
                writer.write(INT_TEMPLATE);
            } else {
                writer.write(STRING_TEMPLATE);
            }
            writer.popState();
        }
    }
}
