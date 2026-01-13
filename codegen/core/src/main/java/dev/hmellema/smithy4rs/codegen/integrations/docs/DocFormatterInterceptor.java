package dev.hmellema.smithy4rs.codegen.integrations.docs;

import dev.hmellema.smithy4rs.codegen.sections.DocstringSection;
import dev.hmellema.smithy4rs.codegen.writer.RustWriter;
import java.util.Scanner;
import software.amazon.smithy.utils.CodeInterceptor;
import software.amazon.smithy.utils.StringUtils;

final class DocFormatterInterceptor implements CodeInterceptor<DocstringSection, RustWriter> {
    private static final int MAX_LINE_LENGTH = 88;

    @Override
    public Class<DocstringSection> sectionType() {
        return DocstringSection.class;
    }

    @Override
    public void write(RustWriter writer, String previousText, DocstringSection section) {
        if (!previousText.isEmpty()) {
            writer.writeInline("/// ");
            writeDocstringLine(writer, previousText);
            writer.write("");
        }
    }

    private void writeDocstringLine(RustWriter writer, String string) {
        for (Scanner it = new Scanner(string); it.hasNextLine();) {
            var s = it.nextLine();

            // Wrap string at max length
            var str = StringUtils.wrap(s, MAX_LINE_LENGTH, writer.getNewline() + "/// ", false);

            writer.writeInlineWithNoFormatting(str);

            if (it.hasNextLine()) {
                writer.writeInlineWithNoFormatting(writer.getNewline() + "/// ");
            }
        }
    }
}
