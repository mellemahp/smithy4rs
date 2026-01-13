/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.generators;

import dev.hmellema.smithy4rs.codegen.CodeGenerationContext;
import dev.hmellema.smithy4rs.codegen.RustCodegenSettings;
import dev.hmellema.smithy4rs.codegen.sections.SchemaSection;
import dev.hmellema.smithy4rs.codegen.sections.ShapeSection;
import dev.hmellema.smithy4rs.codegen.symbols.Smithy4Rs;
import dev.hmellema.smithy4rs.codegen.writer.RustWriter;
import java.util.Locale;
import java.util.SimpleTimeZone;
import java.util.function.Consumer;
import software.amazon.smithy.codegen.core.SymbolProvider;
import software.amazon.smithy.codegen.core.directed.GenerateStructureDirective;
import software.amazon.smithy.model.shapes.MemberShape;
import software.amazon.smithy.utils.CaseUtils;

public final class StructureGenerator implements
        Consumer<GenerateStructureDirective<CodeGenerationContext, RustCodegenSettings>> {

    private static final String SCHEMA_TEMPLATE = """
            ${smithy:T}!(${id:S}: {
                structure ${shape:I} {${#memberSchemas}
                    ${value:C|}${/memberSchemas}
                }
            });
            """;
    private static final String STRUCT_TEMPLATE = """
            #[derive(${derive:T}, PartialEq, Clone)]
            #[smithy_schema(${shape:I})]
            pub struct ${shape:T} {${#memberFields}
                ${value:C|}${/memberFields}
            }
            """;
    @Override
    public void accept(GenerateStructureDirective<CodeGenerationContext, RustCodegenSettings> directive) {
        // Do not generate synthetic structs
        if (directive.shape().getId().getNamespace().startsWith("smithy.synthetic")
                || directive.shape().getId().getNamespace().startsWith("smithy.api")) {
            return;
        }
        directive.context()
                .writerDelegator()
                .useShapeWriter(directive.shape(), writer -> {
                    var members = directive.shape().getAllMembers();
                    var memberSchemas = members.entrySet()
                            .stream()
                            .map(entry -> (Runnable) new MemberSchema(
                                    writer,
                                    directive.symbolProvider(),
                                    entry.getKey(),
                                    entry.getValue()))
                            .toList();
                    var memberFields = members.entrySet()
                            .stream()
                            .map(entry -> (Runnable) new MemberField(
                                    writer,
                                    directive.symbolProvider(),
                                    entry.getKey(),
                                    entry.getValue()))
                            .toList();
                    writer.pushState();
                    // Common data
                    writer.putContext("shape", directive.symbolProvider().toSymbol(directive.shape()));
                    // Generate schema definition
                    writer.pushState(new SchemaSection(directive.shape()));
                    writer.putContext("id", directive.shape().getId());
                    writer.putContext("memberSchemas", memberSchemas);
                    writer.putContext("smithy", Smithy4Rs.SMITHY_MACRO);
                    writer.write(SCHEMA_TEMPLATE);
                    writer.popState();
                    // Generate `struct` impl
                    writer.pushState(new ShapeSection(directive.shape()));
                    writer.putContext("memberFields", memberFields);
                    writer.putContext("derive", Smithy4Rs.SHAPE_DERIVE);
                    writer.write(STRUCT_TEMPLATE);
                    writer.popState();
                    writer.popState();
                });
    }

    private record MemberSchema(
            RustWriter writer,
            SymbolProvider provider,
            String membername,
            MemberShape shape) implements Runnable {
        private static final String TEMPLATE = "${memberIdent:L}: ${shape:I} = ${memberName:S}";

        @Override
        public void run() {
            writer.pushState();
            writer.putContext("memberName", membername);
            writer.putContext("shape", provider.toSymbol(shape));
            writer.putContext("memberIdent", getMemberIdent(membername));
            writer.write(TEMPLATE);
            writer.popState();
        }
    }

    private record MemberField(
            RustWriter writer,
            SymbolProvider provider,
            String membername,
            MemberShape shape) implements Runnable {
        private static final String TEMPLATE = """
                #[smithy_schema(${memberIdent:L})]
                pub ${memberName:L}: ${member:T},""";

        @Override
        public void run() {
            writer.pushState();
            writer.putContext("memberName", membername);
            writer.putContext("member", provider.toSymbol(shape));
            writer.putContext("memberIdent", getMemberIdent(membername));
            writer.write(TEMPLATE);
            writer.popState();
        }
    }

    private static String getMemberIdent(String memberName) {
        return CaseUtils.toSnakeCase(memberName).toUpperCase(Locale.ENGLISH);
    }
}
