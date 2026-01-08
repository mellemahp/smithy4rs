/*
 * Copyright Scaffold Software LLC. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.test.utils;

import java.io.IOException;
import java.net.URI;
import java.net.URISyntaxException;
import java.net.URL;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;
import java.util.stream.Stream;
import software.amazon.smithy.build.SmithyBuildPlugin;

final class RustCodegenTestUtils {
    private RustCodegenTestUtils() {/* utility class */}

    /**
     * Convert URL to URI for resources
     */
    static URI getUriForResource(URL resource) {
        try {
            return resource.toURI();
        } catch (URISyntaxException e) {
            throw new RustCodegenTestException("Resource URI invalid", e);
        }
    }

    /**
     * Walk this level of the directory for a rust (.rs) file with the corresponding name.
     */
    static Path getRustFile(String name, Path parent) {
        try (var files = Files.walk(parent, 1)) {
            return files.filter(Files::isRegularFile)
                    .filter(p -> p.getFileName().toString().endsWith(".rs"))
                    .filter(p -> trimFileName(p.getFileName().toString()).equals(name))
                    .reduce((a, b) -> {
                        throw new RustCodegenTestException("Only rust file should match each smithy file. Found " +
                                "multiple for name: " + name);
                    })
                    .orElseThrow(() -> new RustCodegenTestException("No .rs file found corresponding to : " + name));
        } catch (IOException e) {
            throw new RustCodegenTestException("Failed to load snapshot tests", e);
        }
    }

    /**
     * Trims file extensions from file name.
     */
    static String trimFileName(String filename) {
        return filename.substring(0, filename.lastIndexOf("."));
    }


    static List<SnapshotTestCase> resolveTestCases(URL resource, SmithyBuildPlugin plugin) {
        try (Stream<Path> files = Files.walk(Paths.get(RustCodegenTestUtils.getUriForResource(resource)))) {
            return files.filter(Files::isRegularFile)
                    .filter(path -> path.getFileName().toString().endsWith(".smithy"))
                    .map(path -> SnapshotTestCase.fromModelPath(path, plugin))
                    .toList();
        } catch (IOException e) {
            throw new RustCodegenTestException("Failed to load snapshot tests", e);
        }
    }
}
