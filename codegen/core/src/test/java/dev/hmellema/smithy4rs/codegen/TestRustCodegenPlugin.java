package dev.hmellema.smithy4rs.codegen;

import dev.hmellema.smithy4rs.codegen.transforms.SyntheticServiceTransform;
import dev.hmellema.smithy4rs.codegen.writer.RustWriter;
import software.amazon.smithy.build.PluginContext;
import software.amazon.smithy.build.SmithyBuildPlugin;
import software.amazon.smithy.codegen.core.directed.CodegenDirector;

/**
 * Test code generator that executes Rust directed codegen for Snapshot tests
 */
public final class TestRustCodegenPlugin implements SmithyBuildPlugin {
    public CodeGenerationContext capturedContext;

    @Override
    public String getName() {
        return "test-rust-codegen-core";
    }

    @Override
    public void execute(PluginContext context) {
        CodegenDirector<RustWriter, RustCodegenIntegration, CodeGenerationContext, RustCodegenSettings> runner =
                new CodegenDirector<>();
        var settings = RustCodegenSettings.fromNode(context.getSettings());
        runner.settings(settings);
        TestRustCodegen directedCodegen = new TestRustCodegen();
        runner.directedCodegen(directedCodegen);
        runner.fileManifest(context.getFileManifest());
        runner.service(SyntheticServiceTransform.SYNTHETIC_SERVICE_ID);
        runner.model(SyntheticServiceTransform.transform(context.getModel()));
        runner.integrationClass(RustCodegenIntegration.class);
        // TODO(transforms): Add default transforms
        runner.run();
        this.capturedContext = directedCodegen.context;
    }
}
