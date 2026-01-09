/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.test.utils;

import java.util.Optional;
import java.util.function.Function;
import java.util.stream.Stream;
import org.junit.jupiter.api.extension.ExtensionContext;
import org.junit.jupiter.api.extension.TestTemplateInvocationContext;
import org.junit.jupiter.api.extension.TestTemplateInvocationContextProvider;
import software.amazon.smithy.build.SmithyBuildPlugin;

/**
 * Provider that generates Rust codegen test cases.
 */
public final class RustCodegenSnapShotProvider implements TestTemplateInvocationContextProvider {
    private static final String DEFAULT_SNAPSHOT_LOCATION = "generated";
    private static final Function<String, Optional<SmithyBuildPlugin>> SERVICE_FACTORY =
            SmithyBuildPlugin.createServiceFactory(RustCodegenSnapShotProvider.class.getClassLoader());

    @Override
    public boolean supportsTestTemplate(ExtensionContext context) {
        return context.getRequiredTestMethod().isAnnotationPresent(RustCodegenTest.class);
    }

    @Override
    public Stream<? extends TestTemplateInvocationContext> provideTestTemplateInvocationContexts(
            ExtensionContext context
    ) {

        // Test class should have the expected annotation
        var testMethod = context.getRequiredTestMethod();
        var pluginName = testMethod.getAnnotation(RustCodegenTest.class).value();
        var plugin = SERVICE_FACTORY.apply(pluginName)
                .orElseThrow(
                        () -> new RustCodegenTestException("Could not find specified build plugin"));

        // Ensure expected resource dir exists
        var testClass = context.getRequiredTestClass();
        var dir = testClass.getResource(DEFAULT_SNAPSHOT_LOCATION);
        if (dir == null) {
            throw new RustCodegenTestException("Expected `translations/` resource directory.");
        }

        return RustCodegenTestUtils.resolveTestCases(dir, plugin).stream();
    }
}
