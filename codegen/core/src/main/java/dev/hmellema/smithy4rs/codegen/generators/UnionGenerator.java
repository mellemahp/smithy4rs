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
import java.util.function.Consumer;
import software.amazon.smithy.codegen.core.SymbolProvider;
import software.amazon.smithy.codegen.core.directed.GenerateUnionDirective;
import software.amazon.smithy.model.shapes.MemberShape;
import software.amazon.smithy.utils.CaseUtils;
import software.amazon.smithy.utils.StringUtils;

public class UnionGenerator implements
        Consumer<GenerateUnionDirective<CodeGenerationContext, RustCodegenSettings>> {
    public static final String SCHEMA_TEMPLATE = """
            ${smithy:T}!(${id:S}: {
                union ${shape:I} {${#memberSchemas}
                    ${value:C|}${/memberSchemas}
                }
            });
            """;
    public static final String STRUCT_TEMPLATE = """
            #[${union:T}]
            #[derive(${derive:T})]
            #[smithy_schema(${shape:I})]
            pub enum ${shape:T} {${#memberVariants}
                ${value:C|}${/memberVariants}
            }
            """;
    @Override
    public void accept(GenerateUnionDirective<CodeGenerationContext, RustCodegenSettings> directive) {
        directive.context()
                .writerDelegator()
                .useShapeWriter(directive.shape(), writer -> {
                    var members = directive.shape().getAllMembers();
                    var memberSchemas = members.entrySet()
                            .stream()
                            .map(entry -> (Runnable) new MemberSchemaGenerator(
                                    writer,
                                    directive.symbolProvider(),
                                    entry.getKey(),
                                    entry.getValue()))
                            .toList();
                    var memberVariants = members.entrySet()
                            .stream()
                            .map(entry -> (Runnable) new MemberGenerator(
                                    writer,
                                    directive.symbolProvider(),
                                    entry.getKey(),
                                    entry.getValue()))
                            .toList();
                    // == Write Template ==
                    writer.pushState();
                    // Common values
                    writer.putContext("shape", directive.symbolProvider().toSymbol(directive.shape()));
                    // Write schema definition
                    writer.pushState(new SchemaSection(directive.shape()));
                    writer.putContext("id", directive.shape().getId());
                    writer.putContext("memberSchemas", memberSchemas);
                    writer.putContext("smithy", Smithy4Rs.SMITHY_MACRO);
                    writer.write(SCHEMA_TEMPLATE);
                    writer.popState();
                    writer.pushState(new ShapeSection(directive.shape()));
                    // Write struct template
                    writer.putContext("memberVariants", memberVariants);
                    writer.putContext("derive", Smithy4Rs.SHAPE_DERIVE);
                    writer.putContext("union", Smithy4Rs.UNION_MACRO);
                    writer.write(STRUCT_TEMPLATE);
                    writer.popState();
                    writer.popState();
                });
    }

    private record MemberSchemaGenerator(
            RustWriter writer,
            SymbolProvider provider,
            String membername,
            MemberShape shape) implements Runnable {
        private static final String TEMPLATE = "${memberSchema:L}: ${member:I} = ${memberName:S}";

        @Override
        public void run() {
            writer.pushState();
            writer.putContext("memberSchema", getMemberIdent(membername));
            writer.putContext("member", provider.toSymbol(shape));
            writer.putContext("memberName", membername);
            writer.write(TEMPLATE);
            writer.popState();
        }
    }

    private record MemberGenerator(
            RustWriter writer,
            SymbolProvider provider,
            String membername,
            MemberShape shape) implements Runnable {
        private static final String TEMPLATE = """
                #[smithy_schema(${memberSchema:L})]
                ${memberName:L}(${member:T}),""";

        @Override
        public void run() {
            writer.pushState();
            writer.putContext("memberSchema", getMemberIdent(membername));
            writer.putContext("member", provider.toSymbol(shape));
            writer.putContext("memberName", toMemberName(membername));
            writer.write(TEMPLATE);
            writer.popState();
        }
    }

    private static String getMemberIdent(String memberName) {
        return CaseUtils.toSnakeCase(memberName).toUpperCase(Locale.ENGLISH);
    }

    private static String toMemberName(String memberName) {
        return StringUtils.capitalize(CaseUtils.toCamelCase(memberName));
    }
}
