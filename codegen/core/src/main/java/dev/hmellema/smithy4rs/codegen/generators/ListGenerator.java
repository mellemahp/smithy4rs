package dev.hmellema.smithy4rs.codegen.generators;

import dev.hmellema.smithy4rs.codegen.CodeGenerationContext;
import dev.hmellema.smithy4rs.codegen.RustCodegenSettings;
import java.util.List;
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
        smithy!(${id:S}: {
            list ${shape:I} {
                member: ${member:I}
            }
        });
        """;

    @Override
    public void accept(GenerateListDirective<CodeGenerationContext, RustCodegenSettings> directive) {
        directive.context()
                .writerDelegator().useShapeWriter(directive.shape(), writer -> {
                    var list = directive.symbolProvider().toSymbol(directive.shape());
                    var member = directive.symbolProvider().toSymbol(directive.shape().getMember());

                    // TODO(codegen): Add sections
                    writer.pushState();
                    writer.putContext("traits", List.of());
                    writer.putContext("id", directive.shape().getId());
                    writer.putContext("member", member);
                    writer.putContext("shape", list);
                    writer.write(TEMPLATE);
                    writer.popState();
                });
    }
}
