package dev.hmellema.smithy4rs.codegen.sections;

import software.amazon.smithy.model.shapes.Shape;
import software.amazon.smithy.utils.CodeSection;

/**
 * Contains a Schema definition for a shape.
 *
 * @param target shape that schema is defined for
 */
public record SchemaSection(Shape target) implements DocumentedSection {
}
