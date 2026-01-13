package dev.hmellema.smithy4rs.codegen.integrations.docs;

import dev.hmellema.smithy4rs.codegen.sections.DocstringSection;
import dev.hmellema.smithy4rs.codegen.writer.RustWriter;
import software.amazon.smithy.model.traits.ExternalDocumentationTrait;
import software.amazon.smithy.model.traits.SinceTrait;
import software.amazon.smithy.utils.CodeInterceptor;

/**
 * Adds note to Docstring based on `@since` trait.
 */
final class SinceTraitInterceptor implements CodeInterceptor.Appender<DocstringSection, RustWriter> {
    private static final String TEMPLATE = """
            <div class="note">
            
            **Since**: ${since:L}
            
            </div>
            """;

    @Override
    public void append(RustWriter writer, DocstringSection section) {
        var trait = section.target().expectTrait(SinceTrait.class);
        writer.pushState();
        writer.putContext("since", trait.getValue());
        writer.write(TEMPLATE);
        writer.popState();
    }

    @Override
    public Class<DocstringSection> sectionType() {
        return DocstringSection.class;
    }

    @Override
    public boolean isIntercepted(DocstringSection section) {
        return section.target() != null && section.target().hasTrait(SinceTrait.class);
    }
}
