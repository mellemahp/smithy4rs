/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.sections;

import software.amazon.smithy.model.shapes.Shape;

/**
 * Contains a Schema definition for a shape.
 *
 * @param target shape that schema is defined for
 */
public record SchemaSection(Shape target) implements DocumentedSection {}
