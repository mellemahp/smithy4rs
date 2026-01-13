package dev.hmellema.smithy4rs.codegen.sections;

import software.amazon.smithy.model.shapes.Shape;
import software.amazon.smithy.utils.CodeSection;

/**
 * TODO
 * @param target
 * @param parent
 */
public record DocstringSection(Shape target, CodeSection parent) implements CodeSection {
}