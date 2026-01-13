/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.integrations.docs;

import dev.hmellema.smithy4rs.codegen.sections.DocstringSection;
import dev.hmellema.smithy4rs.codegen.sections.DocumentedSection;
import dev.hmellema.smithy4rs.codegen.writer.RustWriter;
import software.amazon.smithy.model.traits.DeprecatedTrait;
import software.amazon.smithy.utils.CodeInterceptor;
import software.amazon.smithy.utils.CodeSection;

/**
 * Injects a {@link DocstringSection} that other integrations can act on to add docs.
 */
final class DocInjectorInterceptor implements CodeInterceptor.Prepender<CodeSection, RustWriter> {
    @Override
    public void prepend(RustWriter writer, CodeSection section) {
        if (section instanceof DocumentedSection ds) {
            var shape = ds.target();
            writer.injectSection(new DocstringSection(shape, section));
            if (shape == null) {
                return;
            }
            if (shape.hasTrait(DeprecatedTrait.class)) {
                var deprecated = shape.expectTrait(DeprecatedTrait.class);
                writer.pushState();
                writer.putContext("since", deprecated.getSince().orElse(""));
                writer.putContext("note", deprecated.getMessage().orElse(""));
                writer.write("#[deprecated${?since}(since = ${since:S}${?note}, ${/note}${?note}note = " +
                        "${note:S}${/note})" +
                        "${/since}]");
                writer.popState();
            }
        }
    }

    @Override
    public Class<CodeSection> sectionType() {
        return CodeSection.class;
    }

    @Override
    public boolean isIntercepted(CodeSection section) {
        return section instanceof DocumentedSection;
    }
}
