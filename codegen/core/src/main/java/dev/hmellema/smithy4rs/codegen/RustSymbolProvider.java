package dev.hmellema.smithy4rs.codegen;

import java.util.logging.Logger;
import software.amazon.smithy.codegen.core.Symbol;
import software.amazon.smithy.codegen.core.SymbolProvider;
import software.amazon.smithy.model.shapes.*;

/**
 * Maps Smithy types to Rust symbols
 */
public class RustSymbolProvider implements ShapeVisitor<Symbol>, SymbolProvider {
    private static final Logger LOGGER = Logger.getLogger(RustSymbolProvider.class.getName());

    @Override
    public Symbol toSymbol(Shape shape) {
        return shape.accept(this);
    }

    @Override
    public Symbol blobShape(BlobShape blobShape) {
        return null;
    }

    @Override
    public Symbol booleanShape(BooleanShape booleanShape) {
        return null;
    }

    @Override
    public Symbol listShape(ListShape listShape) {
        return null;
    }

    @Override
    public Symbol mapShape(MapShape mapShape) {
        return null;
    }

    @Override
    public Symbol byteShape(ByteShape byteShape) {
        return null;
    }

    @Override
    public Symbol shortShape(ShortShape shortShape) {
        return null;
    }

    @Override
    public Symbol integerShape(IntegerShape integerShape) {
        return null;
    }

    @Override
    public Symbol longShape(LongShape longShape) {
        return null;
    }

    @Override
    public Symbol floatShape(FloatShape floatShape) {
        return null;
    }

    @Override
    public Symbol documentShape(DocumentShape documentShape) {
        return null;
    }

    @Override
    public Symbol doubleShape(DoubleShape doubleShape) {
        return null;
    }

    @Override
    public Symbol bigIntegerShape(BigIntegerShape bigIntegerShape) {
        return null;
    }

    @Override
    public Symbol bigDecimalShape(BigDecimalShape bigDecimalShape) {
        return null;
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
        return null;
    }

    @Override
    public Symbol structureShape(StructureShape structureShape) {
        return null;
    }

    @Override
    public Symbol unionShape(UnionShape unionShape) {
        return null;
    }

    @Override
    public Symbol memberShape(MemberShape memberShape) {
        return null;
    }

    @Override
    public Symbol timestampShape(TimestampShape timestampShape) {
        return null;
    }
}
