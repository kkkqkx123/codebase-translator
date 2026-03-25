#!/usr/bin/env python3
"""
Detect language script content in files and generate a report.

Usage:
    python detect_cjk.py <file_path> [--language <script_family>] [--output <report_path>] [--verbose]
"""

import sys
import argparse
from pathlib import Path
from typing import List, Tuple, NamedTuple
from datetime import datetime


class Segment(NamedTuple):
    """A segment of consecutive matching lines."""
    start_line: int
    end_line: int
    lines: List[Tuple[int, str]]


def is_cjk_char(char: str) -> bool:
    """
    Check if a character is a CJK character.
    
    CJK Unicode ranges:
    - CJK Unified Ideographs: U+4E00 - U+9FFF
    - CJK Unified Ideographs Extension A: U+3400 - U+4DBF
    - CJK Unified Ideographs Extension B: U+20000 - U+2A6DF
    - Hiragana (Japanese): U+3040 - U+309F
    - Katakana (Japanese): U+30A0 - U+30FF
    - Hangul Syllables (Korean): U+AC00 - U+D7AF
    - Hangul Jamo (Korean): U+1100 - U+11FF
    """
    code_point = ord(char)
    
    cjk_ranges = [
        (0x3400, 0x4DBF),    # CJK Extension A
        (0x4E00, 0x9FFF),    # CJK Unified Ideographs
        (0xF900, 0xFAFF),    # CJK Compatibility Ideographs
        (0x3040, 0x309F),    # Hiragana
        (0x30A0, 0x30FF),    # Katakana
        (0xAC00, 0xD7AF),    # Hangul Syllables
        (0x1100, 0x11FF),    # Hangul Jamo
        (0x2E80, 0x2EFF),    # CJK Radicals Supplement
        (0x2F00, 0x2FDF),    # Kangxi Radicals
        (0x31C0, 0x31EF),    # CJK Strokes
    ]
    
    for start, end in cjk_ranges:
        if start <= code_point <= end:
            return True
    
    # Check extension B-F (surrogate pairs)
    if 0x20000 <= code_point <= 0x2EBEF:
        return True
    
    # Check compatibility ideographs supplement
    if 0x2F800 <= code_point <= 0x2FA1F:
        return True
    
    return False


def is_cyrillic_char(char: str) -> bool:
    """Check if a character is Cyrillic."""
    code_point = ord(char)
    return (0x0400 <= code_point <= 0x04FF) or (0x0500 <= code_point <= 0x052F)


def is_latin_char(char: str) -> bool:
    """Check if a character is Latin."""
    code_point = ord(char)
    return char.isalpha() and (code_point <= 0x024F or char.isascii())


def is_arabic_char(char: str) -> bool:
    """Check if a character is Arabic."""
    code_point = ord(char)
    return (0x0600 <= code_point <= 0x06FF) or (0x0750 <= code_point <= 0x077F)


def is_hebrew_char(char: str) -> bool:
    """Check if a character is Hebrew."""
    code_point = ord(char)
    return 0x0590 <= code_point <= 0x05FF


def is_greek_char(char: str) -> bool:
    """Check if a character is Greek."""
    code_point = ord(char)
    return (0x0370 <= code_point <= 0x03FF) or (0x1F00 <= code_point <= 0x1FFF)


def matches_script(text: str, script: str) -> bool:
    """Check if text contains characters from the specified script."""
    char_checkers = {
        "CJK": is_cjk_char,
        "CYRILLIC": is_cyrillic_char,
        "LATIN": is_latin_char,
        "ARABIC": is_arabic_char,
        "HEBREW": is_hebrew_char,
        "GREEK": is_greek_char,
    }
    
    checker = char_checkers.get(script.upper())
    if not checker:
        return False
    
    return any(checker(char) for char in text)


def find_matching_lines(file_path: Path, script: str) -> List[Tuple[int, str]]:
    """
    Find all lines containing characters from the specified script.
    
    Returns: [(line_number, line_content), ...]
    """
    results = []
    
    try:
        # Try multiple encodings
        encodings = ['utf-8', 'utf-8-sig', 'gbk', 'gb2312', 'big5', 'shift_jis']
        content = None
        
        for encoding in encodings:
            try:
                with open(file_path, 'r', encoding=encoding) as f:
                    content = f.readlines()
                break
            except (UnicodeDecodeError, UnicodeError):
                continue
        
        if content is None:
            print(f"Error: Cannot read file {file_path} with supported encodings")
            return results
        
        for line_num, line in enumerate(content, start=1):
            if matches_script(line.rstrip('\n\r'), script):
                results.append((line_num, line.rstrip('\n\r')))
    
    except FileNotFoundError:
        print(f"Error: File not found: {file_path}")
    except PermissionError:
        print(f"Error: Permission denied: {file_path}")
    except Exception as e:
        print(f"Error: Exception while reading file: {e}")
    
    return results


def merge_consecutive_lines(matching_lines: List[Tuple[int, str]]) -> List[Segment]:
    """Merge consecutive matching lines into segments."""
    if not matching_lines:
        return []
    
    segments = []
    current_start = matching_lines[0][0]
    current_end = matching_lines[0][0]
    current_lines = [matching_lines[0]]
    
    for line_num, line in matching_lines[1:]:
        if line_num == current_end + 1:
            current_end = line_num
            current_lines.append((line_num, line))
        else:
            segments.append(Segment(current_start, current_end, current_lines))
            current_start = line_num
            current_end = line_num
            current_lines = [(line_num, line)]
    
    if current_lines:
        segments.append(Segment(current_start, current_end, current_lines))
    
    return segments


def truncate_string(s: str, max_len: int) -> str:
    """Truncate string to maximum length."""
    if len(s) <= max_len:
        return s
    return s[:max_len] + "..."


def generate_report(file_path: Path, script: str, segments: List[Segment], 
                    total_lines: int, output_path: Path = None, verbose: bool = False) -> str:
    """
    Generate language script detection report.
    """
    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    matching_lines = sum(len(seg.lines) for seg in segments)
    
    report_lines = [
        "=" * 80,
        "Language Detection Report",
        "=" * 80,
        "",
        "Summary:",
        f"  Detection Time:    {timestamp}",
        f"  Target Path:       {file_path.absolute()}",
        f"  Total Files:       1",
        f"  Total Lines:       {total_lines}",
        f"  Matching Lines:    {matching_lines}",
        f"  Matching Segments: {len(segments)}",
        f"  Target Script:     {script}",
        "",
        "-" * 80,
        "",
        "NOTE: Detection is based on Unicode script/language family.",
        "      Specific language identification is not guaranteed.",
        "",
        "-" * 80,
        "",
        "Detection Results:",
        "",
        "-" * 80,
        "",
    ]
    
    if not segments:
        report_lines.append("No matching content found.")
        report_lines.append("")
    else:
        for seg_idx, segment in enumerate(segments, 1):
            report_lines.append(f"  Segment {seg_idx} (Lines {segment.start_line}-{segment.end_line}):")
            
            if verbose:
                # In verbose mode, show all lines
                for line_num, line in segment.lines:
                    report_lines.append(f"    {line_num:>4}: {line}")
            else:
                # In normal mode, show preview (first 3 lines)
                preview_lines = segment.lines[:3]
                for _, line in preview_lines:
                    report_lines.append(f"    {truncate_string(line, 80)}")
                if len(segment.lines) > 3:
                    report_lines.append(f"    ... ({len(segment.lines) - 3} more lines)")
            
            report_lines.append("")
        
        report_lines.append(f"  Total: {matching_lines} matching line(s)")
        report_lines.append("")
        report_lines.append("-" * 80)
        report_lines.append("")
    
    report_lines.append("=" * 80)
    
    report = "\n".join(report_lines)
    
    if output_path:
        output_path.parent.mkdir(parents=True, exist_ok=True)
        with open(output_path, 'w', encoding='utf-8') as f:
            f.write(report)
        print(f"Report saved to: {output_path}")
    
    return report


def main():
    parser = argparse.ArgumentParser(
        description="Detect language script content in files and generate a report",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
    python detect_cjk.py README.md
    python detect_cjk.py README.md --language cjk
    python detect_cjk.py README.md --language cyrillic --output report.md
    python detect_cjk.py README.md --verbose
        """
    )
    
    parser.add_argument(
        "file_path",
        type=Path,
        help="Path to file to detect"
    )
    
    parser.add_argument(
        "--language", "-l",
        type=str,
        default="cjk",
        help="Language family/script to detect (cjk, cyrillic, latin, arabic, hebrew, greek)"
    )
    
    parser.add_argument(
        "--output", "-o",
        type=Path,
        default=None,
        help="Output path for the report (optional, default: stdout)"
    )
    
    parser.add_argument(
        "--verbose", "-v",
        action="store_true",
        help="Verbose mode: show line-by-line details"
    )
    
    args = parser.parse_args()
    
    # Validate script argument
    valid_scripts = ["cjk", "cyrillic", "latin", "arabic", "hebrew", "greek"]
    script = args.language.upper()
    if args.language.lower() not in valid_scripts:
        print(f"Error: Unsupported language family '{args.language}'. Supported families: {', '.join(valid_scripts)}")
        sys.exit(1)
    
    if not args.file_path.exists():
        print(f"Error: File not found: {args.file_path}")
        sys.exit(1)
    
    if args.file_path.is_dir():
        print("Error: Directory detection is not supported in this version.")
        sys.exit(1)
    
    # Read file to get total line count
    try:
        with open(args.file_path, 'r', encoding='utf-8', errors='ignore') as f:
            total_lines = sum(1 for _ in f)
    except Exception:
        total_lines = 0
    
    # Find matching lines
    matching_lines = find_matching_lines(args.file_path, script)
    
    # Merge into segments
    segments = merge_consecutive_lines(matching_lines)
    
    # Generate report
    if args.output:
        generate_report(args.file_path, script, segments, total_lines, args.output, args.verbose)
    else:
        report = generate_report(args.file_path, script, segments, total_lines, verbose=args.verbose)
        print(report)
    
    # Exit with status code
    sys.exit(0 if not segments else 1)


if __name__ == "__main__":
    main()
