package dev.hmellema.smithy4rs.codegen.integrations.core;

import dev.hmellema.smithy4rs.codegen.CodeGenerationContext;
import dev.hmellema.smithy4rs.codegen.TraitInitializer;
import dev.hmellema.smithy4rs.codegen.writer.RustWriter;
import software.amazon.smithy.codegen.core.Symbol;
import software.amazon.smithy.model.traits.RangeTrait;

final class RangeTraitInitializer implements TraitInitializer<RangeTrait> {
    private static final Symbol SYMBOL = TraitInitializerUtils.preludeTrait(RangeTrait.class);

    @Override
    public Class<RangeTrait> traitClass() {
        return RangeTrait.class;
    }

    @Override
    public void write(RustWriter writer, CodeGenerationContext context, RangeTrait trait) {
        writer.putContext("t", SYMBOL);
        writer.putContext("min", trait.getMin());
        writer.putContext("max", trait.getMax());
        writer.writeInline(
                "${t:T}::builder()${?min}.min(${min:S})${/min}"
                        + "${?max}.max(${max:S})${/max}.build()");
    }
}
