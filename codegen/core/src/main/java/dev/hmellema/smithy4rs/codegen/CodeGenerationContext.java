/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen;

import dev.hmellema.smithy4rs.codegen.writer.RustWriter;
import java.util.List;
import software.amazon.smithy.build.FileManifest;
import software.amazon.smithy.codegen.core.CodegenContext;
import software.amazon.smithy.codegen.core.SymbolProvider;
import software.amazon.smithy.codegen.core.WriterDelegator;
import software.amazon.smithy.model.Model;

public record CodeGenerationContext(
        Model model,
        RustCodegenSettings settings,
        SymbolProvider symbolProvider,
        FileManifest fileManifest,
        WriterDelegator<RustWriter> writerDelegator,
        List<RustCodegenIntegration> integrations) implements CodegenContext<RustCodegenSettings, RustWriter,
                RustCodegenIntegration> {}
