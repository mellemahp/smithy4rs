/*
 * Copyright Scaffold Software LLC. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.test.utils;

import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;
import org.junit.jupiter.api.TestTemplate;
import org.junit.jupiter.api.Timeout;
import org.junit.jupiter.api.extension.ExtendWith;
import org.junit.platform.commons.annotation.Testable;
import software.amazon.smithy.build.SmithyBuildPlugin;

/**
 * Junit5 Test Extension for Rust Codegen Snapshot Tests.
 *
 * <p>This runner will search for pairs of pre- (smithy) and post-generation (rust) files in
 * a {@code generate/} resource directory. The pre- and post-generation file names should match and the
 * post-generation file extension should be `.rs`.
 *
 * <p>This extension is applied to a test method and provides a {@link Runnable} parameter. Simply run
 * the provided runnable to execute tests:
 * <pre>{@code
 * class SnapshotTests {
 *     @RustCodegenTest(MyCodeGeneratorPlugin.class)
 *     void execute(Consumer<ObjectNode> test) {
 *         test.run(MY_TEST_SETTINGS);
 *     }
 * }
 * }</pre>
 *
 * <p>The following shows an example of the expected file structure to discover translation
 * snapshots:
 * {@snippet lang=CommandLine:
 *  mypackage/
 *   └── src/
 *       └── test/
 *            └── resources/
 *                  └── generate/
 *                        ├── to-generate.rs
 *                        └── to-generated.smithy
 * }
 */
@Target(ElementType.METHOD)
@Retention(RetentionPolicy.RUNTIME)
@TestTemplate
@Testable
@Timeout(5)
@ExtendWith(RustCodegenSnapShotProvider.class)
public @interface RustCodegenTest {
    /**
     * Name of Smithy Build plugin to use to generate rust code from Smithy model.
     */
    String value();
}
