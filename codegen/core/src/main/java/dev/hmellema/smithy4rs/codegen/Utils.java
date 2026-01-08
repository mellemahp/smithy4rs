package dev.hmellema.smithy4rs.codegen;

import java.net.URL;
import java.util.Objects;
import software.amazon.smithy.codegen.core.ReservedWords;
import software.amazon.smithy.codegen.core.ReservedWordsBuilder;

public final class Utils {
    private static final URL RUST_RESERVED_WORDS = Objects.requireNonNull(
            Utils.class.getResource("rust-reserved-words.txt"));
    public static final ReservedWords SHAPE_ESCAPER = new ReservedWordsBuilder()
            .loadCaseInsensitiveWords(RUST_RESERVED_WORDS, word -> word + "Shape")
            .build();
    public static final ReservedWords MEMBER_ESCAPER = new ReservedWordsBuilder()
            .loadCaseInsensitiveWords(RUST_RESERVED_WORDS, word -> word + "Member")
            .build();

    private Utils() {
        // Utility class should not be instantiated
    }
}
