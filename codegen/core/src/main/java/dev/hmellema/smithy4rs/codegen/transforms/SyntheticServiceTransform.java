package dev.hmellema.smithy4rs.codegen.transforms;

import dev.hmellema.smithy4rs.codegen.RustCodegenSettings;
import java.util.HashSet;
import java.util.Set;
import java.util.logging.Logger;
import java.util.stream.Collectors;
import software.amazon.smithy.model.Model;
import software.amazon.smithy.model.loader.Prelude;
import software.amazon.smithy.model.neighbor.Walker;
import software.amazon.smithy.model.selector.Selector;
import software.amazon.smithy.model.shapes.OperationShape;
import software.amazon.smithy.model.shapes.ServiceShape;
import software.amazon.smithy.model.shapes.Shape;
import software.amazon.smithy.model.shapes.ShapeId;
import software.amazon.smithy.model.shapes.StructureShape;
import software.amazon.smithy.model.traits.ErrorTrait;
import software.amazon.smithy.model.transform.ModelTransformer;

/**
 * Generates a synthetic service for a set of shapes. 
 *
 * <p>Adds a set of shapes to the closure of a synthetic service shape. Operations shapes are added directly
 * to the service shape while all other shapes are added to the service via synthetic operations with synthetic inputs.
 *
 * <p>The logic here is mostly duplicated from: https://github
 * .com/smithy-lang/smithy-java/blob/main/codegen/plugins/types-codegen/src/main/java/software/amazon/smithy/java/codegen/types/TypeCodegenSettings.java
 *
 * <p>This is public so it can be used in the core tests AND in the type-codegen plugin.
 */
public final class SyntheticServiceTransform {
    private static final Logger LOGGER = Logger.getLogger(SyntheticServiceTransform.class.getName());
    static final String SYNTHETIC_NAMESPACE = "smithy.synthetic";
    public static final ShapeId SYNTHETIC_SERVICE_ID = ShapeId.fromParts(SYNTHETIC_NAMESPACE, "SyntheticService");

    public static Model transform(Model model) {
        var serviceBuilder = ServiceShape.builder().id(SYNTHETIC_SERVICE_ID);

        Set<Shape> shapesToAdd = new HashSet<>();
        for (Shape shape : getClosure(model)) {
            switch (shape.getType()) {
                case SERVICE, RESOURCE -> LOGGER.fine(
                        () -> "Skipping service-associated shape {} for type codegen..." + shape
                );
                case OPERATION -> serviceBuilder.addOperation(shape.asOperationShape().orElseThrow());
                case STRUCTURE, ENUM, INT_ENUM, UNION, LIST, MAP -> {
                    var syntheticInput = createSyntheticWrapper(shape, "Input");
                    shapesToAdd.add(syntheticInput);
                    var syntheticOutput = createSyntheticWrapper(shape, "Output");
                    shapesToAdd.add(syntheticOutput);
                    var syntheticOperation = createSyntheticOperation(syntheticInput, syntheticOutput);
                    shapesToAdd.add(syntheticOperation);
                    serviceBuilder.addOperation(syntheticOperation);
                }
                default -> {
                    // All other shapes are skipped with no logging as they should be
                    // implicitly added by aggregate shapes.
                }
            }
        }
        shapesToAdd.add(serviceBuilder.build());

        return ModelTransformer.create().replaceShapes(model, shapesToAdd);
    }

    private static OperationShape createSyntheticOperation(Shape shape, Shape syntheticOutput) {
        var id = ShapeId.fromParts(SYNTHETIC_NAMESPACE, shape.getId().getName() + "Operation");
        var operationBuilder = OperationShape.builder().id(id);
        if (shape.hasTrait(ErrorTrait.class)) {
            operationBuilder.addError(shape.toShapeId());
        } else {
            operationBuilder.input(shape.toShapeId());
        }
        operationBuilder.output(syntheticOutput.toShapeId());
        return operationBuilder.build();
    }

    private static StructureShape createSyntheticWrapper(Shape shape, String suffix) {
        return StructureShape.builder()
                .id(ShapeId.fromParts(SYNTHETIC_NAMESPACE, shape.getId().getName() + suffix))
                .addMember("syntheticMember", shape.getId())
                .build();
    }

    private static Set<Shape> getClosure (Model model){
        Set<Shape> closure = new HashSet<>();
       model.shapes()
               .filter(s -> !s.isMemberShape())
               .filter(s -> !Prelude.isPreludeShape(s))
               .forEach(closure::add);

        // Filter out any shapes from this closure that are contained by any other shapes in the closure
        Walker walker = new Walker(model);
        Set<Shape> nested = new HashSet<>();
        for (Shape shape : closure) {
            nested.addAll(
                    walker.walkShapes(shape)
                            .stream()
                            .filter(s -> !shape.equals(s))
                            .filter(s -> !s.isMemberShape())
                            .filter(s -> !Prelude.isPreludeShape(s))
                            .filter(nested::contains)
                            .collect(Collectors.toSet()));
            }
            closure.removeAll(nested);
            if (closure.isEmpty()) {
                // TODO(errors): Add codegen exception
                throw new RuntimeException("Could not create synthetic service. No shapes found in closure");
            }
            LOGGER.info("Found " + closure.size() + " shapes in synthetic service closure.");

            return closure;
    }
}