/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.integrations.docs;

import dev.hmellema.smithy4rs.codegen.sections.DocstringSection;
import dev.hmellema.smithy4rs.codegen.writer.RustWriter;
import software.amazon.smithy.model.traits.DocumentationTrait;
import software.amazon.smithy.utils.CodeInterceptor;

final class DocumentationTraitInterceptor implements CodeInterceptor<DocstringSection, RustWriter> {
    @Override
    public void write(RustWriter writer, String previousText, DocstringSection section) {
        writer.writeWithNoFormatting(section.target().expectTrait(DocumentationTrait.class).getValue());

        if (!previousText.isEmpty()) {
            // Add spacing if additional headings have been added
            writer.writeInline(System.lineSeparator());
            writer.writeInlineWithNoFormatting(previousText);
        }
    }

    @Override
    public Class<DocstringSection> sectionType() {
        return DocstringSection.class;
    }

    @Override
    public boolean isIntercepted(DocstringSection section) {
        return section.target() != null && section.target().hasTrait(DocumentationTrait.class);
    }
}
