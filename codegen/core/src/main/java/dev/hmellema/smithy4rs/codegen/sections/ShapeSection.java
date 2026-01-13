/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.sections;

import software.amazon.smithy.model.shapes.Shape;

/**
 * Contains a shape (such as `enum` or `struct`) definition.
 *
 * @param target shape targeted by this section
 */
public record ShapeSection(Shape target) implements DocumentedSection {}
