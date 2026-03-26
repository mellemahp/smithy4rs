/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.generators;

import dev.hmellema.smithy4rs.codegen.symbols.Smithy4Rs;
import dev.hmellema.smithy4rs.codegen.symbols.StdLib;
import dev.hmellema.smithy4rs.codegen.writer.RustWriter;
import java.util.EnumSet;
import java.util.List;
import software.amazon.smithy.codegen.core.Symbol;
import software.amazon.smithy.model.shapes.Shape;
import software.amazon.smithy.model.shapes.ShapeType;
import software.amazon.smithy.model.traits.TraitDefinition;

public record DeriveGenerator(RustWriter writer, Shape shape) implements Runnable {
    public static final List<Symbol> DERIVES_SHAPE = List.of(
            Smithy4Rs.SHAPE_DERIVE,
            StdLib.PARTIAL_EQ_DERIVE,
            StdLib.CLONE_DERIVE);

    public static final List<Symbol> DERIVES_TRAIT = List.of(
            Smithy4Rs.SHAPE_DERIVE,
            Smithy4Rs.TRAIT_DERIVE,
            StdLib.PARTIAL_EQ_DERIVE,
            StdLib.CLONE_DERIVE);

    public static final List<Symbol> DERIVES_TRAIT_WRAPPER = List.of(
            Smithy4Rs.SHAPE_DERIVE,
            Smithy4Rs.TRAIT_DERIVE,
            StdLib.CLONE_DERIVE);
    private static final String TEMPLATE = "#[derive(${#der}${value:T}${^key.last}, ${/key.last}${/der})]";
    private static final EnumSet<ShapeType> NON_WRAPPER = EnumSet.of(
            ShapeType.INT_ENUM,
            ShapeType.UNION,
            ShapeType.STRUCTURE);
    @Override
    public void run() {
        writer.pushState();
        var der = DERIVES_SHAPE;
        if (shape.hasTrait(TraitDefinition.class)) {
            if (NON_WRAPPER.contains(shape.getType())) {
                der = DERIVES_TRAIT;
            } else {
                der = DERIVES_TRAIT_WRAPPER;
            }
        }
        writer.putContext("der", der);
        writer.write(TEMPLATE);
        writer.popState();
    }
}
