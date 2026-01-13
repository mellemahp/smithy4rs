/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.integrations.docs;

import dev.hmellema.smithy4rs.codegen.CodeGenerationContext;
import dev.hmellema.smithy4rs.codegen.RustCodegenIntegration;
import dev.hmellema.smithy4rs.codegen.writer.RustWriter;
import java.util.List;
import software.amazon.smithy.utils.CodeInterceptor;
import software.amazon.smithy.utils.CodeSection;

/**
 * TODO
 */
public final class RustDocstringIntegration implements RustCodegenIntegration {
    @Override
    public String name() {
        return "rustdoc";
    }

    @Override
    public List<? extends CodeInterceptor<? extends CodeSection, RustWriter>> interceptors(
            CodeGenerationContext codegenContext
    ) {
        // TODO: Examples
        return List.of(
                new DocInjectorInterceptor(),
                new DocumentationTraitInterceptor(),
                new SinceTraitInterceptor(),
                new UnstableTraitInterceptor(),
                new ExternalDocumentationTraitInterceptor(),
                new DocFormatterInterceptor());
    }
}
