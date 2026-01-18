/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.types;

import dev.hmellema.smithy4rs.codegen.CodeGenerationContext;
import dev.hmellema.smithy4rs.codegen.RustCodegenIntegration;
import dev.hmellema.smithy4rs.codegen.RustCodegenSettings;
import dev.hmellema.smithy4rs.codegen.transforms.SyntheticServiceTransform;
import dev.hmellema.smithy4rs.codegen.writer.RustWriter;
import software.amazon.smithy.build.PluginContext;
import software.amazon.smithy.build.SmithyBuildPlugin;
import software.amazon.smithy.codegen.core.directed.CodegenDirector;

/**
 * Test code generator that executes Rust directed codegen for Snapshot tests
 */
public final class RustTypeCodegenPlugin implements SmithyBuildPlugin {

    @Override
    public String getName() {
        return "rust-types";
    }

    @Override
    public void execute(PluginContext context) {
        CodegenDirector<RustWriter, RustCodegenIntegration, CodeGenerationContext, RustCodegenSettings> runner =
                new CodegenDirector<>();
        var settings = RustCodegenSettings.fromNode(context.getSettings());
        runner.settings(settings);
        runner.directedCodegen(new RustTypeCodegen());
        runner.fileManifest(context.getFileManifest());
        runner.service(SyntheticServiceTransform.SYNTHETIC_SERVICE_ID);
        runner.model(SyntheticServiceTransform.transform(context.getModel()));
        runner.integrationClass(RustCodegenIntegration.class);
        // TODO(transforms): Add default transforms
        runner.run();
    }
}
