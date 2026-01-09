/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.test.utils;

import static org.junit.jupiter.api.Assertions.assertEquals;

import java.nio.file.Path;
import java.util.List;
import java.util.Objects;
import org.junit.jupiter.api.extension.Extension;
import org.junit.jupiter.api.extension.TestTemplateInvocationContext;
import software.amazon.smithy.build.MockManifest;
import software.amazon.smithy.build.PluginContext;
import software.amazon.smithy.build.SmithyBuildPlugin;
import software.amazon.smithy.model.Model;
import software.amazon.smithy.model.loader.ModelAssembler;
import software.amazon.smithy.model.node.ObjectNode;
import software.amazon.smithy.utils.IoUtils;

public record SnapshotTestCase(String name, Model model, Path expected, SmithyBuildPlugin buildPlugin)
        implements TestTemplateInvocationContext {

    static SnapshotTestCase fromModelPath(Path path, SmithyBuildPlugin buildPlugin) {
        var name = RustCodegenTestUtils.trimFileName(Objects.requireNonNull(path.getFileName()).toString());

        // Resolve the reference smithy model for processing
        var expected = Model.assembler()
                .putProperty(ModelAssembler.ALLOW_UNKNOWN_TRAITS, true)
                .addImport(path)
                .disableValidation()
                .assemble()
                .unwrap();

        var rustFile = RustCodegenTestUtils.getRustFile(name, path.getParent());

        return new SnapshotTestCase(name, expected, rustFile, buildPlugin);
    }

    @Override
    public String getDisplayName(int invocationIndex) {
        return this.name;
    }

    void execute(ObjectNode settings) {
        var mockManifest = new MockManifest();
        var context = PluginContext.builder()
                .model(model)
                .fileManifest(mockManifest)
                .pluginClassLoader(PluginContext.class.getClassLoader())
                .settings(settings)
                .build();

        // Run plugin
        buildPlugin.execute(context);

        // Compare results
        // TODO(customization): Should this be allowed to vary?

        var res = mockManifest.getFileString("smithy-generated.rs")
                .orElseThrow(() -> new RustCodegenTestException("Could not find expected output file"));
        var expected = IoUtils.readUtf8File(expected());

        assertEquals(expected, res);
    }

    @Override
    public List<Extension> getAdditionalExtensions() {
        return List.of((RunnableParameterResolver) this::execute);
    }
}
