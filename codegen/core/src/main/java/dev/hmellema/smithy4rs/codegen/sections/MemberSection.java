/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.sections;

import software.amazon.smithy.model.shapes.Shape;

/**
 * Contains a union, enum, or structure member
 *
 * @param target shape targeted by this section
 */
public record MemberSection(Shape target) implements DocumentedSection {}
