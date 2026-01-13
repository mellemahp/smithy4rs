package dev.hmellema.smithy4rs.codegen.sections;

import software.amazon.smithy.model.shapes.Shape;
import software.amazon.smithy.utils.CodeSection;

/**
 * Indicates that a section can have user-defined documentation
 */
public interface DocumentedSection extends CodeSection {
    /**
     * @return Shape targeted by this section, or null if target is not a shape
     */
    Shape target();
}