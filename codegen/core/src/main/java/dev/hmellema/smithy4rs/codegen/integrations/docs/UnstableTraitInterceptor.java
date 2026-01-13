package dev.hmellema.smithy4rs.codegen.integrations.docs;

import dev.hmellema.smithy4rs.codegen.sections.DocstringSection;
import dev.hmellema.smithy4rs.codegen.writer.RustWriter;
import software.amazon.smithy.model.traits.UnstableTrait;
import software.amazon.smithy.utils.CodeInterceptor;

/**
 * Add a warning for member and shapes marked with `@unstable`
 */
final class UnstableTraitInterceptor implements CodeInterceptor.Appender<DocstringSection, RustWriter>{
    private static final String TEMPLATE = """
            <div class="warning">
            
            **WARNING**: Unstable feature
            
            </div>
            """;

    @Override
    public void append(RustWriter writer, DocstringSection section) {
        writer.pushState();
        writer.write(TEMPLATE);
        writer.popState();
    }

    @Override
    public Class<DocstringSection> sectionType() {
        return DocstringSection.class;
    }

    @Override
    public boolean isIntercepted(DocstringSection section) {
        return section.target() != null && section.target().hasTrait(UnstableTrait.class);
    }
}
