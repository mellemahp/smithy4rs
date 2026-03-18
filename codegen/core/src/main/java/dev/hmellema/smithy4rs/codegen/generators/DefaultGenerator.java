package dev.hmellema.smithy4rs.codegen.generators;

import dev.hmellema.smithy4rs.codegen.SymbolProperties;
import dev.hmellema.smithy4rs.codegen.symbols.Smithy4Rs;
import dev.hmellema.smithy4rs.codegen.symbols.StdLib;
import dev.hmellema.smithy4rs.codegen.writer.RustWriter;
import java.util.Objects;
import software.amazon.smithy.codegen.core.SymbolProvider;
import software.amazon.smithy.model.Model;
import software.amazon.smithy.model.node.ArrayNode;
import software.amazon.smithy.model.node.BooleanNode;
import software.amazon.smithy.model.node.Node;
import software.amazon.smithy.model.node.NodeVisitor;
import software.amazon.smithy.model.node.NullNode;
import software.amazon.smithy.model.node.NumberNode;
import software.amazon.smithy.model.node.ObjectNode;
import software.amazon.smithy.model.node.StringNode;
import software.amazon.smithy.model.shapes.*;
import software.amazon.smithy.utils.CaseUtils;
import software.amazon.smithy.utils.StringUtils;

public final class DefaultGenerator extends ShapeVisitor.Default<Void> implements Runnable {
    private final RustWriter writer;
    private final Model model;
    private final Shape shape;
    private final SymbolProvider provider;
    private final Node value;

    public DefaultGenerator(
            RustWriter writer,
            Model model,
            Shape shape,
            SymbolProvider provider,
            Node value
    ) {
        this.writer = writer;
        this.shape = shape;
        this.provider = provider;
        this.value = value;
        this.model = model;
    }

    @Override
    public void run() {
        shape.accept(this);
    }

    @Override
    protected Void getDefault(Shape shape) {
        throw new UnsupportedOperationException("Shape not supported: " + shape);
    }

    @Override
    public Void blobShape(BlobShape blobShape) {
        writer.writeInline(
                "$T::from_bytes($S.as_bytes())",
                Smithy4Rs.BYTE_BUFFER,
                value.expectStringNode().getValue()
        );
        return null;
    }

    @Override
    public Void booleanShape(BooleanShape booleanShape) {
        writer.writeInline("$L", value.expectBooleanNode());
        return null;
    }

    @Override
    public Void listShape(ListShape listShape) {
        if (!value.expectArrayNode().isEmpty()) {
            // TODO: Specific error
            throw new RuntimeException("Only empty lists are permitted for defaults");
        }
        writer.writeInline("$T::<$T>::new()", StdLib.VEC, provider.toSymbol(shape));
        return null;
    }

    @Override
    public Void mapShape(MapShape mapShape) {
        if (!value.expectObjectNode().isEmpty()) {
            // TODO: Specific error
            throw new RuntimeException("Only empty maps are permitted for defaults");
        }
        writer.pushState();
        writer.putContext("map", Smithy4Rs.INDEX_MAP);
        writer.putContext("key", provider.toSymbol(mapShape.getKey()));
        writer.putContext("value", provider.toSymbol(mapShape.getValue()));
        writer.writeInline("${map:T}::<${key:T}, ${value:T}>::default()");
        writer.popState();
        return null;
    }

    @Override
    public Void byteShape(ByteShape byteShape) {
        writer.writeInline("$Li8", value.expectNumberNode().getValue());
        return null;
    }

    @Override
    public Void shortShape(ShortShape shortShape) {
        writer.writeInline("$Li16", value.expectNumberNode().getValue());
        return null;
    }

    @Override
    public Void integerShape(IntegerShape integerShape) {
        writer.writeInline("$Li32", value.expectNumberNode().getValue());
        return null;
    }

    @Override
    public Void intEnumShape(IntEnumShape shape) {
        var number = value.expectNumberNode().getValue().intValue();
        for (var entry: shape.getEnumValues().entrySet()) {
            if (entry.getValue() == number) {
                // TODO: Use a standard method for enum names
                writer.writeInline(
                        "$T::$L",
                        provider.toSymbol(shape),
                        StringUtils.capitalize(CaseUtils.toPascalCase(entry.getKey()))
                );
                return null;
            }
        }
        throw new RuntimeException("Could not find value: `" + number + "` for int enum " + shape.getId());
    }

    @Override
    public Void longShape(LongShape longShape) {
        writer.writeInline("$Li64", value.expectNumberNode().getValue());
        return null;
    }

    @Override
    public Void floatShape(FloatShape floatShape) {
        writer.writeInline("$Lf32", value.expectNumberNode().getValue());
        return null;
    }

    @Override
    public Void documentShape(DocumentShape documentShape) {
        writer.writeInline("$C", new DocumentValueVisitor(writer, value));
        return null;
    }

    @Override
    public Void doubleShape(DoubleShape doubleShape) {
        writer.writeInline("$Lf64", value.expectNumberNode().getValue());
        return null;
    }

    @Override
    public Void bigIntegerShape(BigIntegerShape bigIntegerShape) {
        writer.writeInline("$T::from_str($S).unwrap()", Smithy4Rs.BIG_INT, value);
        return null;
    }

    @Override
    public Void bigDecimalShape(BigDecimalShape bigDecimalShape) {
        writer.writeInline("$T::from_str($S).unwrap()", Smithy4Rs.BIG_DECIMAL, value);
        return null;
    }

    @Override
    public Void stringShape(StringShape stringShape) {
        writer.writeInline("$S.to_string()", value.expectStringNode().getValue());
        return null;
    }

    @Override
    public Void enumShape(EnumShape shape) {
        var stringValue = value.expectStringNode().getValue();
        for (var entry: shape.getEnumValues().entrySet()) {
            System.out.println("FOUND: " + entry.getValue());
            System.out.println("NODE: " + stringValue);

            if (entry.getValue().equals(stringValue)) {
                // TODO: Use a standard method for enum names
                writer.writeInline(
                        "$T::$L",
                        provider.toSymbol(shape),
                        StringUtils.capitalize(CaseUtils.toPascalCase(entry.getKey()))
                );
                return null;
            }
        }
        throw new RuntimeException("Could not find value: `" + value + "` for enum " + shape.getId());
    }

    @Override
    public Void memberShape(MemberShape memberShape) {
        return model.expectShape(memberShape.getTarget()).accept(this);
    }

    @Override
    public Void timestampShape(TimestampShape timestampShape) {

        // TODO
        return null;
    }

    private record DocumentValueVisitor(
            RustWriter writer,
            Node node
    ) implements NodeVisitor<Void>, Runnable {
        @Override
        public void run() {
            node.accept(this);
        }

        @Override
        public Void arrayNode(ArrayNode arrayNode) {
            if (!arrayNode.isEmpty()) {
                throw new RuntimeException("Only empty arrays allowed as defaults");
            }
            // TODO: Use dyn type once merged with upstream changes
            writer.writeInline(
                "$T::<$T<dyn $T>>::new().into()",
                    StdLib.VEC,
                    StdLib.BOX,
                    Smithy4Rs.DOCUMENT
            );
            return null;
        }

        @Override
        public Void booleanNode(BooleanNode booleanNode) {
            writer.writeInline("$L.into()", booleanNode.getValue());
            return null;
        }

        @Override
        public Void nullNode(NullNode nullNode) {
            throw new RuntimeException("Should never explicitly set null doc");
        }

        @Override
        public Void numberNode(NumberNode numberNode) {
            // Note: Uses highest precision of each so we can always convert
            if (numberNode.isFloatingPointNumber()) {
                writer.writeInline("$Lf64", numberNode.getValue());
            } else {
                writer.writeInline("$Li64", numberNode.getValue());
            }
            return null;
        }

        @Override
        public Void objectNode(ObjectNode objectNode) {
            if (!objectNode.isEmpty()) {
                throw new RuntimeException("Only empty maps allowed as defaults");
            }
            // TODO: Use dyn type once merged with upstream changes
            writer.writeInline(
                    "$T::<$T, $T<dyn $T>>::default().into()",
                    StdLib.VEC,
                    StdLib.STRING,
                    StdLib.BOX,
                    Smithy4Rs.DOCUMENT
            );
            return null;
        }

        @Override
        public Void stringNode(StringNode stringNode) {
            writer.writeInline("$S.into()", stringNode.getValue());
            return null;
        }
    }
}
