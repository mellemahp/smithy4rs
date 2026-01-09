/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen;

import software.amazon.smithy.model.node.ObjectNode;
import software.amazon.smithy.model.shapes.ShapeId;

/**
 * Settings for Rust codegen plugins
 */
public record RustCodegenSettings(ShapeId service) {
    public static RustCodegenSettings fromNode(ObjectNode node) {
        return new RustCodegenSettings(null);
    }
}
