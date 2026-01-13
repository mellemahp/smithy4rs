package dev.hmellema.smithy4rs.codegen.integrations.docs;

import dev.hmellema.smithy4rs.codegen.sections.DocstringSection;
import dev.hmellema.smithy4rs.codegen.writer.RustWriter;
import software.amazon.smithy.model.traits.ExternalDocumentationTrait;
import software.amazon.smithy.utils.CodeInterceptor;

/**
 * Adds Docstrings secton for the {@link ExternalDocumentationTrait}.
 */
final class ExternalDocumentationTraitInterceptor implements CodeInterceptor.Appender<DocstringSection, RustWriter> {
    private static final String TEMPLATE = """
            ## References
            ${#links}- [**${key:L}**](${value:S})
            ${/links}
            """;

    @Override
    public void append(RustWriter writer, DocstringSection section) {
        var trait = section.target().expectTrait(ExternalDocumentationTrait.class);
        writer.pushState();
        writer.putContext("links", trait.getUrls());
        writer.write(TEMPLATE);
        writer.popState();
    }

    @Override
    public Class<DocstringSection> sectionType() {
        return DocstringSection.class;
    }

    @Override
    public boolean isIntercepted(DocstringSection section) {
        return section.target() != null && section.target().hasTrait(ExternalDocumentationTrait.class);
    }
}
