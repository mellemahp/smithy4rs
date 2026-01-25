/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.integrations.core;

import dev.hmellema.smithy4rs.codegen.CodeGenerationContext;
import dev.hmellema.smithy4rs.codegen.SymbolProperties;
import dev.hmellema.smithy4rs.codegen.TraitInitializer;
import dev.hmellema.smithy4rs.codegen.symbols.Smithy4Rs;
import dev.hmellema.smithy4rs.codegen.writer.RustWriter;
import java.util.Objects;
import software.amazon.smithy.codegen.core.Symbol;
import software.amazon.smithy.model.node.ArrayNode;
import software.amazon.smithy.model.node.BooleanNode;
import software.amazon.smithy.model.node.Node;
import software.amazon.smithy.model.node.NodeVisitor;
import software.amazon.smithy.model.node.NullNode;
import software.amazon.smithy.model.node.NumberNode;
import software.amazon.smithy.model.node.ObjectNode;
import software.amazon.smithy.model.node.StringNode;
import software.amazon.smithy.model.traits.Trait;

/**
 * Catch-all initializer for unknown traits.
 */
final class GenericTraitInitializer implements TraitInitializer<Trait> {
    private static final Symbol DYNAMIC_TRAIT = Symbol.builder()
            .name("DynamicTrait")
            .namespace("smithy4rs_core::schema", "::")
            .build();

    @Override
    public Class<Trait> traitClass() {
        return Trait.class;
    }

    @Override
    public void write(RustWriter writer, CodeGenerationContext context, Trait trait) {
        writer.pushState();
        writer.putContext("id", trait.toShapeId());
        writer.putContext("dynamicTrait", DYNAMIC_TRAIT);
        writer.putContext("node", new NodeWriter(writer, trait.toNode()));
        writer.writeInline("${dynamicTrait:T}::from(${id:S}, ${node:C})");
        writer.popState();
    }

    @Override
    public boolean isIntercepted(CodeGenerationContext context, Trait trait) {
        // Matches all shapes even if they have no mapping.
        return true;
    }

    private record NodeWriter(RustWriter writer, Node node) implements Runnable, NodeVisitor<Void> {
        @Override
        public void run() {
            node.accept(this);
        }

        @Override
        public Void arrayNode(ArrayNode arrayNode) {
            var consumers = arrayNode.getElements()
                    .stream()
                    .map(n -> (Runnable) () -> n.accept(this)).toList();
            writer.pushState();
            writer.putContext("nodes", consumers);
            writer.write("vec![${#nodes}${value:C}${^key.last}, ${/key.last}${/nodes}${^nodes}Vec::new()${/nodes}]");
            writer.popState();
            return null;
        }

        @Override
        public Void booleanNode(BooleanNode booleanNode) {
            writer.write("$L", booleanNode.getValue());
            return null;
        }

        @Override
        public Void nullNode(NullNode nullNode) {
            throw new UnsupportedOperationException("todo");
        }

        @Override
        public Void numberNode(NumberNode numberNode) {
            if (numberNode.isFloatingPointNumber()) {
                writer.write("$L", numberNode.getValue().doubleValue());
            } else {
                writer.write("$L", numberNode.getValue().intValue());
            }
            return null;
        }

        @Override
        public Void objectNode(ObjectNode objectNode) {
            // doc_map!["a" => 1, "b" => 2]
            writer.writeInline("$T![", Smithy4Rs.DOC_MAP_MACRO);
            var entries = objectNode.getMembers().entrySet().iterator();
            while (entries.hasNext()) {
                var entry = entries.next();
                writer.writeInline(
                        "$C => $C",
                        new NodeWriter(writer, entry.getKey()),
                        new NodeWriter(writer, entry.getValue())
                );
                if (entries.hasNext()) {
                    writer.writeInline(", ");
                }
            }
            writer.writeInline("]");
            return null;
        }

        @Override
        public Void stringNode(StringNode stringNode) {
            writer.write("$S", stringNode.getValue());
            return null;
        }
    }
}
