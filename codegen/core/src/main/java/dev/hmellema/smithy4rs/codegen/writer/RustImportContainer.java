/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.writer;

import java.util.Map;
import java.util.Objects;
import java.util.TreeMap;
import software.amazon.smithy.codegen.core.ImportContainer;
import software.amazon.smithy.codegen.core.Symbol;

public class RustImportContainer implements ImportContainer {
    private final ImportNode root = new ImportNode("");

    // TODO: Should we support aliasing via symbol property?
    @Override
    public void importSymbol(Symbol symbol, String alias) {
        if (Objects.equals(symbol.getNamespace(), "")
                || symbol.getNamespace().startsWith("std")
                || symbol.getNamespace().equals("local")) {
            // Ignore local imports and std library imports
            return;
        }

        ImportNode current = root;
        var namespaceSegments = symbol.getNamespace().split(symbol.getNamespaceDelimiter());
        for (String segment : namespaceSegments) {
            current = current.children.computeIfAbsent(segment, ImportNode::new);
        }
        // Terminal node.
        current.children.put(symbol.getName(), new ImportNode(symbol.getName()));
    }

    @Override
    public String toString() {
        var builder = new StringBuilder();
        for (var node : root.children.values()) {
            builder.append("use ");
            node.write(builder, 1);
            // Remove newline and comma
            builder.deleteCharAt(builder.length() - 1);
            builder.deleteCharAt(builder.length() - 1);
            builder.append(";").append(System.lineSeparator());
        }
        return builder.toString();
    }

    private static final class ImportNode {
        private static final String SEPARATOR = "::";
        private static final String INDENT = "    ";

        private final Map<String, ImportNode> children = new TreeMap<>();
        private final String name;

        ImportNode(String name) {
            this.name = name;
        }

        void write(StringBuilder builder, int indent) {
            builder.append(name);
            switch (children.size()) {
                // If there are no children then this is a Symbol. Add a comma and
                // return
                case 0 -> builder.append(',').append(System.lineSeparator());
                // If there is a single child, continue writing, but don't indent or
                // add braces
                case 1 -> {
                    builder.append(SEPARATOR);
                    var next = children.values()
                            .stream()
                            .findFirst()
                            .orElseThrow(() -> new RuntimeException("Expected value"));
                    next.write(builder, indent);
                }
                default -> {
                    builder.append(SEPARATOR).append('{').append(System.lineSeparator());
                    for (var child : children.values()) {
                        builder.append(INDENT.repeat(indent));
                        child.write(builder, indent + 1);
                    }
                    builder.append(INDENT.repeat(indent - 1))
                            .append('}')
                            .append(',')
                            .append(System.lineSeparator());
                }
            }
        }
    }
}
