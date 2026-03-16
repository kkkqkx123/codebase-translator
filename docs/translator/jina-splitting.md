const chunkRegex = new RegExp(  
    "(" +  
    // 1. Headings (Setext-style, Markdown, and HTML-style, with length constraints)  
    `(?:^(?:[#*=-]{1,${MAX\_HEADING\_LENGTH}}|\\w[^\\r\\n]{0,${MAX\_HEADING\_CONTENT\_LENGTH}}\\r?\\n[-=]{2,${MAX\_HEADING\_UNDERLINE\_LENGTH}}|<h[1-6][^>]{0,${MAX\_HTML\_HEADING\_ATTRIBUTES\_LENGTH}}>)[^\\r\\n]{1,${MAX\_HEADING\_CONTENT\_LENGTH}}(?:</h[1-6]>)?(?:\\r?\\n|$))` +  
    "|" +  
    // New pattern for citations  
    `(?:\\[[0-9]+\\][^\\r\\n]{1,${MAX\_STANDALONE\_LINE\_LENGTH}})` +  
    "|" +  
    // 2. List items (bulleted, numbered, lettered, or task lists, including nested, up to three levels, with length constraints)  
    `(?:(?:^|\\r?\\n)[ \\t]{0,3}(?:[-*+•]|\\d{1,3}\\.\\w\\.|\\[[ xX]\\])[ \\t]+(?:(?:\\b[^\\r\\n]{1,${MAX\_LIST\_ITEM\_LENGTH}}\\b(?:[.!?…]|\\.{3}|[\\u2026\\u2047-\\u2049]|[\\p{Emoji_Presentation}\\p{Extended_Pictographic}])(?=\\s|$))|(?:\\b[^\\r\\n]{1,${MAX\_LIST\_ITEM\_LENGTH}}\\b(?=[\\r\\n]|$))|(?:\\b[^\\r\\n]{1,${MAX\_LIST\_ITEM\_LENGTH}}\\b(?=[.!?…]|\\.{3}|[\\u2026\\u2047-\\u2049]|[\\p{Emoji_Presentation}\\p{Extended_Pictographic}])(?:.{1,${LOOKAHEAD\_RANGE}}(?:[.!?…]|\\.{3}|[\\u2026\\u2047-\\u2049]|[\\p{Emoji_Presentation}\\p{Extended_Pictographic}])(?=\\s|$))?))` +  
    `(?:(?:\\r?\\n[ \\t]{2,5}(?:[-*+•]|\\d{1,3}\\.\\w\\.|\\[[ xX]\\])[ \\t]+(?:(?:\\b[^\\r\\n]{1,${MAX\_LIST\_ITEM\_LENGTH}}\\b(?:[.!?…]|\\.{3}|[\\u2026\\u2047-\\u2049]|[\\p{Emoji_Presentation}\\p{Extended_Pictographic}])(?=\\s|$))|(?:\\b[^\\r\\n]{1,${MAX\_LIST\_ITEM\_LENGTH}}\\b(?=[\\r\\n]|$))|(?:\\b[^\\r\\n]{1,${MAX\_LIST\_ITEM\_LENGTH}}\\b(?=[.!?…]|\\.{3}|[\\u2026\\u2047-\\u2049]|[\\p{Emoji_Presentation}\\p{Extended_Pictographic}])(?:.{1,${LOOKAHEAD\_RANGE}}(?:[.!?…]|\\.{3}|[\\u2026\\u2047-\\u2049]|[\\p{Emoji_Presentation}\\p{Extended_Pictographic}])(?=\\s|$))?)))` +  
    `{0,${MAX\_NESTED\_LIST\_ITEMS}}(?:\\r?\\n[ \\t]{4,${MAX\_LIST\_INDENT\_SPACES}}(?:[-*+•]|\\d{1,3}\\.\\w\\.|\\[[ xX]\\])[ \\t]+(?:(?:\\b[^\\r\\n]{1,${MAX\_LIST\_ITEM\_LENGTH}}\\b(?:[.!?…]|\\.{3}|[\\u2026\\u2047-\\u2049]|[\\p{Emoji_Presentation}\\p{Extended_Pictographic}])(?=\\s|$))|(?:\\b[^\\r\\n]{1,${MAX\_LIST\_ITEM\_LENGTH}}\\b(?=[\\r\\n]|$))|(?:\\b[^\\r\\n]{1,${MAX\_LIST\_ITEM\_LENGTH}}\\b(?=[.!?…]|\\.{3}|[\\u2026\\u2047-\\u2049]|[\\p{Emoji_Presentation}\\p{Extended_Pictographic}])(?:.{1,${LOOKAHEAD\_RANGE}}(?:[.!?…]|\\.{3}|[\\u2026\\u2047-\\u2049]|[\\p{Emoji_Presentation}\\p{Extended_Pictographic}])(?=\\s|$))?)))` +  
    `{0,${MAX\_NESTED\_LIST\_ITEMS}})?)` +  
    "|" +  
    // 3. Block quotes (including nested quotes and citations, up to three levels, with length constraints)  
    `(?:(?:^>(?:>|\\s{2,}){0,2}(?:(?:\\b[^\\r\\n]{0,${MAX\_BLOCKQUOTE\_LINE\_LENGTH}}\\b(?:[.!?…]|\\.{3}|[\\u2026\\u2047-\\u2049]|[\\p{Emoji_Presentation}\\p{Extended_Pictographic}])(?=\\s|$))|(?:\\b[^\\r\\n]{0,${MAX\_BLOCKQUOTE\_LINE\_LENGTH}}\\b(?=[\\r\\n]|$))|(?:\\b[^\\r\\n]{0,${MAX\_BLOCKQUOTE\_LINE\_LENGTH}}\\b(?=[.!?…]|\\.{3}|[\\u2026\\u2047-\\u2049]|[\\p{Emoji_Presentation}\\p{Extended_Pictographic}])(?:.{1,${LOOKAHEAD\_RANGE}}(?:[.!?…]|\\.{3}|[\\u2026\\u2047-\\u2049]|[\\p{Emoji_Presentation}\\p{Extended_Pictographic}])(?=\\s|$))?))\\r?\\n?){1,${MAX\_BLOCKQUOTE\_LINES}})` +  
    "|" +  
    // 4. Code blocks (fenced, indented, or HTML pre/code tags, with length constraints)  
    `(?:(?:^|\\r?\\n)(?:\`\`\`|~~~)(?:\\w{0,${MAX\_CODE\_LANGUAGE\_LENGTH}})?\\r?\\n[\\s\\S]{0,${MAX\_CODE\_BLOCK\_LENGTH}}?(?:\`\`\`|~~~)\\r?\\n?` +  
    `|(?:(?:^|\\r?\\n)(?: {4}|\\t)[^\\r\\n]{0,${MAX\_LIST\_ITEM\_LENGTH}}(?:\\r?\\n(?: {4}|\\t)[^\\r\\n]{0,${MAX\_LIST\_ITEM\_LENGTH}}){0,${MAX\_INDENTED\_CODE\_LINES}}\\r?\\n?)` +  
    `|(?:<pre>(?:<code>)?[\\s\\S]{0,${MAX\_CODE\_BLOCK\_LENGTH}}?(?:</code>)?</pre>))` +  
    "|" +  
    // 5. Tables (Markdown, grid tables, and HTML tables, with length constraints)  
    `(?:(?:^|\\r?\\n)(?:\\|[^\\r\\n]{0,${MAX\_TABLE\_CELL\_LENGTH}}\\|(?:\\r?\\n\\|[-:]{1,${MAX\_TABLE\_CELL\_LENGTH}}\\|){0,1}(?:\\r?\\n\\|[^\\r\\n]{0,${MAX\_TABLE\_CELL\_LENGTH}}\\|){0,${MAX\_TABLE\_ROWS}}` +  
    `|<table>[\\s\\S]{0,${MAX\_HTML\_TABLE\_LENGTH}}?</table>))` +  
    "|" +  
    // 6. Horizontal rules (Markdown and HTML hr tag)  
    `(?:^(?:[-*_]){${MIN\_HORIZONTAL\_RULE\_LENGTH},}\\s*$|<hr\\s*/?>)` +  
    "|" +  
    // 10. Standalone lines or phrases (including single-line blocks and HTML elements, with length constraints)  
    `(?:^(?:<[a-zA-Z][^>]{0,${MAX\_HTML\_TAG\_ATTRIBUTES\_LENGTH}}>)?(?:(?:[^\\r\\n]{1,${MAX\_STANDALONE\_LINE\_LENGTH}}(?:[.!?…]|\\.\\.\\.|[\\u2026\\u2047-\\u2049]|[\\p{Emoji_Presentation}\\p{Extended_Pictographic}])(?=\\s|$))|(?:[^\\r\\n]{1,${MAX\_STANDALONE\_LINE\_LENGTH}}(?=[\\r\\n]|$))|(?:[^\\r\\n]{1,${MAX\_STANDALONE\_LINE\_LENGTH}}(?=[.!?…]|\\.\\.\\.|[\\u2026\\u2047-\\u2049]|[\\p{Emoji_Presentation}\\p{Extended_Pictographic}])(?:.{1,${LOOKAHEAD\_RANGE}}(?:[.!?…]|\\.\\.\\.|[\\u2026\\u2047-\\u2049]|[\\p{Emoji_Presentation}\\p{Extended_Pictographic}])(?=\\s|$))?))(?:</[a-zA-Z]+>)?(?:\\r?\\n|$))` +  
    "|" +  
    // 7. Sentences or phrases ending with punctuation (including ellipsis and Unicode punctuation)  
    `(?:(?:[^\\r\\n]{1,${MAX\_SENTENCE\_LENGTH}}(?:[.!?…]|\\.\\.\\.|[\\u2026\\u2047-\\u2049]|[\\p{Emoji_Presentation}\\p{Extended_Pictographic}])(?=\\s|$))|(?:[^\\r\\n]{1,${MAX\_SENTENCE\_LENGTH}}(?=[\\r\\n]|$))|(?:[^\\r\\n]{1,${MAX\_SENTENCE\_LENGTH}}(?=[.!?…]|\\.\\.\\.|[\\u2026\\u2047-\\u2049]|[\\p{Emoji_Presentation}\\p{Extended_Pictographic}])(?:.{1,${LOOKAHEAD\_RANGE}}(?:[.!?…]|\\.\\.\\.|[\\u2026\\u2047-\\u2049]|[\\p{Emoji_Presentation}\\p{Extended_Pictographic}])(?=\\s|$))?))` +  
    "|" +  
    // 8. Quoted text, parenthetical phrases, or bracketed content (with length constraints)  
    "(?:" +  
    `(?<!\\w)\"\"\"[^\"]{0,${MAX\_QUOTED\_TEXT\_LENGTH}}\"\"\"(?!\\w)` +  
    `|(?<!\\w)(?:['\"\`'"])[^\\r\\n]{0,${MAX\_QUOTED\_TEXT\_LENGTH}}\\1(?!\\w)` +  
    `|\\([^\\r\\n()]{0,${MAX\_PARENTHETICAL\_CONTENT\_LENGTH}}(?:\\([^\\r\\n()]{0,${MAX\_PARENTHETICAL\_CONTENT\_LENGTH}}\\)[^\\r\\n()]{0,${MAX\_PARENTHETICAL\_CONTENT\_LENGTH}}){0,${MAX\_NESTED\_PARENTHESES}}\\)` +  
    `|\\[[^\\r\\n\\[\\]]{0,${MAX\_PARENTHETICAL\_CONTENT\_LENGTH}}(?:\\[[^\\r\\n\\[\\]]{0,${MAX\_PARENTHETICAL\_CONTENT\_LENGTH}}\\][^\\r\\n\\[\\]]{0,${MAX\_PARENTHETICAL\_CONTENT\_LENGTH}}){0,${MAX\_NESTED\_PARENTHESES}}\\]` +  
    `|\\$[^\\r\\n$]{0,${MAX\_MATH\_INLINE\_LENGTH}}\\$` +  
    `|\`[^\`\\r\\n]{0,${MAX\_MATH\_INLINE\_LENGTH}}\`` +  
    ")" +  
    "|" +  
    // 9. Paragraphs (with length constraints)  
    `(?:(?:^|\\r?\\n\\r?\\n)(?:<p>)?(?:(?:[^\\r\\n]{1,${MAX\_PARAGRAPH\_LENGTH}}(?:[.!?…]|\\.{3}|[\\u2026\\u2047-\\u2049]|[\\p{Emoji\_Presentation}\\p{Extended\_Pictographic}])(?=\\s|$))|(?:[^\\r\\n]{1,${MAX\_PARAGRAPH\_LENGTH}}(?=[\\r\\n]|$))|(?:[^\\r\\n]{1,${MAX\_PARAGRAPH\_LENGTH}}(?=[.!?…]|\\.{3}|[\\u2026\\u2047-\\u2049]|[\\p{Emoji\_Presentation}\\p{Extended\_Pictographic}])(?:.{1,${LOOKAHEAD\_RANGE}}(?:[.!?…]|\\.{3}|[\\u2026\\u2047-\\u2049]|[\\p{Emoji\_Presentation}\\p{Extended\_Pictographic}])(?=\\s|$))?))(?:</p>)?(?=\\r?\\n\\r?\\n|$))` +  
    "|" +  
    // 11. HTML-like tags and their content (including self-closing tags and attributes, with length constraints)  
    `(?:<[a-zA-Z][^>]{0,${MAX\_HTML\_TAG\_ATTRIBUTES\_LENGTH}}(?:>[\\s\\S]{0,${MAX\_HTML\_TAG\_CONTENT\_LENGTH}}?</[a-zA-Z]+>|\\s*/>))` +  
    "|" +  
    // 12. LaTeX-style math expressions (inline and block, with length constraints)  
    `(?:(?:\\$\\$[\\s\\S]{0,${MAX\_MATH\_BLOCK\_LENGTH}}?\\$\\$)|(?:\\$[^\\$\\r\\n]{0,${MAX\_MATH\_INLINE\_LENGTH}}\\$))` +  
    "|" +  
    // 14. Fallback for any remaining content (with length constraints)  
    `(?:(?:[^\\r\\n]{1,${MAX\_STANDALONE\_LINE\_LENGTH}}(?:[.!?…]|\\.{3}|[\\u2026\\u2047-\\u2049]|[\\p{Emoji\_Presentation}\\p{Extended\_Pictographic}])(?=\\s|$))|(?:[^\\r\\n]{1,${MAX\_STANDALONE\_LINE\_LENGTH}}(?=[\\r\\n]|$))|(?:[^\\r\\n]{1,${MAX\_STANDALONE\_LINE\_LENGTH}}(?=[.!?…]|\\.{3}|[\\u2026\\u2047-\\u2049]|[\\p{Emoji\_Presentation}\\p{Extended\_Pictographic}])(?:.{1,${LOOKAHEAD\_RANGE}}(?:[.!?…]|\\.{3}|[\\u2026\\u2047-\\u2049]|[\\p{Emoji\_Presentation}\\p{Extended\_Pictographic}])(?=\\s|$))?))` +  
    ")",  
    "gmu"  
);  