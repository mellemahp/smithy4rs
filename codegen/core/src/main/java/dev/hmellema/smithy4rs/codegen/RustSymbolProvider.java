/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen;

import java.util.EnumSet;
import java.util.Locale;
import java.util.logging.Logger;
import software.amazon.smithy.codegen.core.Symbol;
import software.amazon.smithy.codegen.core.SymbolProvider;
import software.amazon.smithy.model.Model;
import software.amazon.smithy.model.loader.Prelude;
import software.amazon.smithy.model.shapes.*;
import software.amazon.smithy.utils.CaseUtils;

/**
 * Maps Smithy types to Rust symbols
 */
public record RustSymbolProvider(Model model) implements ShapeVisitor<Symbol>, SymbolProvider {
    private static final Logger LOGGER = Logger.getLogger(RustSymbolProvider.class.getName());
    public static final String FILE = "smithy-generated.rs";
    private static final EnumSet<ShapeType> GENERATED_TYPES = EnumSet.of(
            ShapeType.UNION,
            ShapeType.ENUM,
            ShapeType.INT_ENUM,
            ShapeType.STRUCTURE);
    @Override
    public Symbol toSymbol(Shape shape) {
        return shape.accept(this);
    }

    @Override
    public String toMemberName(MemberShape shape) {
        return CaseUtils.toSnakeCase(shape.getMemberName()).toLowerCase();
    }

    @Override
    public Symbol blobShape(BlobShape blobShape) {
        return Symbol.builder()
                .name("ByteBuffer")
                .putProperty(SymbolProperties.SCHEMA_SYMBOL, getSchemaSymbol(blobShape))
                .namespace("smithy4rs_core", "::")
                .build();
    }

    @Override
    public Symbol booleanShape(BooleanShape booleanShape) {
        return Symbol.builder()
                .name("bool")
                .putProperty(SymbolProperties.SCHEMA_SYMBOL, getSchemaSymbol(booleanShape))
                .namespace("std", "::")
                .build();
    }

    @Override
    public Symbol listShape(ListShape listShape) {
        return Symbol.builder()
                .name("Vec")
                .namespace("std::vec", "::")
                .putProperty(SymbolProperties.SCHEMA_SYMBOL, getSchemaSymbol(listShape))
                .addReference(listShape.getMember().accept(this))
                .declarationFile(FILE)
                .build();
    }

    @Override
    public Symbol mapShape(MapShape mapShape) {
        return Symbol.builder()
                .name("IndexMap")
                .namespace("smithy4rs_core", "::")
                .putProperty(SymbolProperties.SCHEMA_SYMBOL, getSchemaSymbol(mapShape))
                .addReference(mapShape.getKey().accept(this))
                .addReference(mapShape.getValue().accept(this))
                .declarationFile(FILE)
                .build();
    }

    @Override
    public Symbol byteShape(ByteShape byteShape) {
        return Symbol.builder()
                .name("i8")
                .putProperty(SymbolProperties.SCHEMA_SYMBOL, getSchemaSymbol(byteShape))
                .namespace("std", "::")
                .build();
    }

    @Override
    public Symbol shortShape(ShortShape shortShape) {
        return Symbol.builder()
                .name("i16")
                .putProperty(SymbolProperties.SCHEMA_SYMBOL, getSchemaSymbol(shortShape))
                .namespace("std", "::")
                .build();
    }

    @Override
    public Symbol integerShape(IntegerShape integerShape) {
        return Symbol.builder()
                .name("i32")
                .putProperty(SymbolProperties.SCHEMA_SYMBOL, getSchemaSymbol(integerShape))
                .namespace("std", "::")
                .build();
    }

    @Override
    public Symbol intEnumShape(IntEnumShape shape) {
        return Symbol.builder()
                .name(shape.getId().getName())
                .putProperty(SymbolProperties.SCHEMA_SYMBOL, getSchemaSymbol(shape))
                .declarationFile(FILE)
                .build();
    }

    @Override
    public Symbol longShape(LongShape longShape) {
        return Symbol.builder()
                .name("i64")
                .putProperty(SymbolProperties.SCHEMA_SYMBOL, getSchemaSymbol(longShape))
                .namespace("std", "::")
                .build();
    }

    @Override
    public Symbol floatShape(FloatShape floatShape) {
        return Symbol.builder()
                .name("f32")
                .putProperty(SymbolProperties.SCHEMA_SYMBOL, getSchemaSymbol(floatShape))
                .namespace("std", "::")
                .build();
    }

    @Override
    public Symbol documentShape(DocumentShape documentShape) {
        return Symbol.builder()
                .name("Document")
                .putProperty(SymbolProperties.SCHEMA_SYMBOL, getSchemaSymbol(documentShape))
                .namespace("smithy4rs_core", "::")
                .build();
    }

    @Override
    public Symbol doubleShape(DoubleShape doubleShape) {
        return Symbol.builder()
                .name("f64")
                .putProperty(SymbolProperties.SCHEMA_SYMBOL, getSchemaSymbol(doubleShape))
                .namespace("std", "::")
                .build();
    }

    @Override
    public Symbol bigIntegerShape(BigIntegerShape bigIntegerShape) {
        return Symbol.builder()
                .name("BigInt")
                .putProperty(SymbolProperties.SCHEMA_SYMBOL, getSchemaSymbol(bigIntegerShape))
                .namespace("smithy4rs_core", "::")
                .build();
    }

    @Override
    public Symbol bigDecimalShape(BigDecimalShape bigDecimalShape) {
        return Symbol.builder()
                .name("BigDecimal")
                .putProperty(SymbolProperties.SCHEMA_SYMBOL, getSchemaSymbol(bigDecimalShape))
                .namespace("smithy4rs_core", "::")
                .build();
    }

    @Override
    public Symbol operationShape(OperationShape operationShape) {
        return null;
    }

    @Override
    public Symbol resourceShape(ResourceShape resourceShape) {
        return null;
    }

    @Override
    public Symbol serviceShape(ServiceShape serviceShape) {
        return null;
    }

    @Override
    public Symbol stringShape(StringShape stringShape) {
        return Symbol.builder()
                .name("String")
                .putProperty(SymbolProperties.SCHEMA_SYMBOL, getSchemaSymbol(stringShape))
                .namespace("std", "::")
                .build();
    }

    @Override
    public Symbol enumShape(EnumShape shape) {
        return Symbol.builder()
                .name(shape.getId().getName())
                .putProperty(SymbolProperties.SCHEMA_SYMBOL, getSchemaSymbol(shape))
                .declarationFile(FILE)
                .build();
    }

    @Override
    public Symbol structureShape(StructureShape structureShape) {
        // TODO: Add escaping
        return Symbol.builder()
                .name(structureShape.getId().getName())
                .putProperty(SymbolProperties.SCHEMA_SYMBOL, getSchemaSymbol(structureShape))
                .declarationFile(FILE)
                .build();
    }

    @Override
    public Symbol unionShape(UnionShape unionShape) {
        return Symbol.builder()
                .name(unionShape.getId().getName())
                .putProperty(SymbolProperties.SCHEMA_SYMBOL, getSchemaSymbol(unionShape))
                .declarationFile(FILE)
                .build();
    }

    @Override
    public Symbol memberShape(MemberShape memberShape) {
        // Dereference into the target of this member
        var target = model.getShape(memberShape.getTarget())
                .orElseThrow(
                        () -> new RuntimeException(
                                "Could not find shape " + memberShape.getTarget() + " targeted by "
                                        + memberShape));
        return toSymbol(target);
    }

    @Override
    public Symbol timestampShape(TimestampShape timestampShape) {
        return Symbol.builder()
                .name("Instant")
                .putProperty(SymbolProperties.SCHEMA_SYMBOL, getSchemaSymbol(timestampShape))
                .namespace("smithy4rs_core", "::")
                .build();
    }

    private static Symbol getSchemaSymbol(Shape shape) {
        return Symbol.builder()
                .name(getSchemaName(shape))
                .namespace(getSchemaNamespace(shape), "::")
                .build();
    }

    private static String getSchemaNamespace(ToShapeId shapeId) {
        if (Prelude.isPreludeShape(shapeId)) {
            return "smithy4rs_core::prelude";
        } else {
            // TODO(errors): this wont handle imported service errors.
            return "local";
        }
    }

    private static String getSchemaName(Shape shapeId) {
        var baseName = CaseUtils.toSnakeCase(shapeId.toShapeId().getName()).toUpperCase(Locale.ENGLISH);
        if (GENERATED_TYPES.contains(shapeId.getType())) {
            return baseName + "_SCHEMA";
        }
        return baseName;
    }
}
