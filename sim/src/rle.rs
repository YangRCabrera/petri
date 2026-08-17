//! Decodes Golly-style run-length encoded (RLE) Lenia patterns — extended
//! with Bert Chan's multi-character state codes for continuous cell values
//! — into `[0, 1]` cell intensities. See [`decode`] for the entry point
//! [`crate::universe::Universe::load_rle`] calls.
//!
//! State chars: `.`/`b` is 0 (dead) and `o` is 1 (alive), the two-state Life
//! shorthand; unprefixed `A`-`X` are states 1-24 out of 255, and a `p`-`y`
//! prefix shifts into states 25-264 in blocks of 24 (`pA` = 25, `pX` = 48,
//! `qA` = 49, ...) for finer-grained continuous values.

const MAX_RUN: usize = 1 << 20;

/// Parses the cell-data portion of an RLE pattern (after [`strip_header`]
/// strips any `#`-comment/dimensions lines ahead of it) into one `Vec<f32>`
/// per row. Rows are ragged: a run of dead cells at the very end of a row is
/// conventionally omitted before the row-terminating `$` (or the
/// pattern-terminating `!`), so callers that need a rectangular buffer
/// should go through [`decode`], which pads rows out to a common width.
pub fn parse_rle(rle: &str) -> Vec<Vec<f32>> {
    let rle = strip_header(rle);
    let mut count: Option<usize> = None;
    let mut prefix: Option<char> = None;
    let mut out: Vec<Vec<f32>> = vec![Vec::new()];

    for ch in rle.chars() {
        if ch.is_whitespace() {
            continue;
        }

        if let Some(digit) = ch.to_digit(10) {
            let digit = digit as usize;
            count = Some(
                count
                    .unwrap_or(0)
                    .saturating_mul(10)
                    .saturating_add(digit)
                    .min(MAX_RUN),
            );
            continue;
        }

        if is_prefix_char(ch) {
            prefix = Some(ch);
            continue;
        }

        if is_state_char(ch) {
            let runs = count.take().unwrap_or(1);
            let value = f32::from(char_to_value(prefix.take(), ch)) / 255.0;
            if let Some(row) = out.last_mut() {
                row.resize(row.len() + runs, value);
            }
            continue;
        }

        if ch == '$' {
            let runs = count.unwrap_or(1);
            for _ in 0..runs {
                out.push(Vec::new());
            }
        } else if ch == '!' {
            break;
        }
    }

    out
}

/// Strips `#`-prefixed comment lines and the `x = .., y = .., rule = ..`
/// dimensions line a full `.rle` file carries ahead of its cell data. Only
/// the first non-comment line is ever checked against [`is_header_line`] —
/// once pattern data starts, nothing later is mistaken for a header. A bare
/// cell-data string with no header (what `species.ts` decodes offline today)
/// passes through unchanged.
fn strip_header(rle: &str) -> String {
    let mut result = String::new();
    let mut past_header_check = false;

    for line in rle.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }
        if !past_header_check {
            past_header_check = true;
            if is_header_line(trimmed) {
                continue;
            }
        }
        result.push_str(line);
        result.push('\n');
    }

    result
}

fn is_header_line(line: &str) -> bool {
    line.starts_with('x') && line.contains('=')
}

const LOWERCASE_P: u16 = 'p' as u16;
const UPPERCASE_A: u16 = 'A' as u16;

/// Maps one RLE state character to its 0-255 intensity. A pending `prefix`
/// (from [`is_prefix_char`]) combines with `ch` into an extended state code
/// only when `ch` is `A`-`X` — `(prefix - 'p') * 24 + (ch - 'A' + 25)` walks
/// `pA`..`yX` up through states 25-264. A prefix followed by anything else
/// (`.`/`b`/`o`, which `is_state_char` also admits) falls through to the
/// unprefixed handling below instead of underflowing the `ch - 'A'` subtraction:
/// `.`/`b` and `o` are the two-state Life shorthand (0 and 255), and any other
/// unprefixed letter is a state 1-24 (`A` = 1, `X` = 24).
fn char_to_value(prefix: Option<char>, ch: char) -> u16 {
    if let Some(prefix) = prefix {
        if ('A'..='X').contains(&ch) {
            let prefix = prefix as u16;
            let ch = ch as u16;
            return (prefix - LOWERCASE_P) * 24 + (ch - UPPERCASE_A + 25);
        }
    }

    if ch == '.' || ch == 'b' {
        0
    } else if ch == 'o' {
        255
    } else {
        let ch = ch as u16;
        ch - UPPERCASE_A + 1
    }
}

/// Whether `ch` is one of RLE's extended-state prefix letters (`p`-`y`),
/// which combine with a following state char to reach states beyond 24.
fn is_prefix_char(ch: char) -> bool {
    ('p'..='y').contains(&ch)
}

/// Whether `ch` encodes a cell value on its own, or as the second half of
/// a prefixed pair: `.`/`b`/`o` (the two-state shorthand) or `A`-`X` (the
/// 24 unprefixed/prefixed state letters). Anything else — lowercase
/// letters other than `b`/`o`, or unprefixed `Y`/`Z` (which would
/// otherwise collide with the prefixed `pA`.. range, since `char_to_value`
/// computes unprefixed letter values as `ch - 'A' + 1`) — isn't a valid
/// RLE-Lenia state char and falls through to being silently ignored, same
/// as any other unrecognized character in [`parse_rle`].
fn is_state_char(ch: char) -> bool {
    ch == '.' || ch == 'b' || ch == 'o' || ('A'..='X').contains(&ch)
}

/// Decodes `rle` and pads [`parse_rle`]'s ragged rows out to a common width
/// (RLE conventionally omits the trailing dead-cell run before a row's `$`
/// or the pattern's `!`), returning the `(width, height)` the padded grid
/// ended up and its cells flattened row-major — the shape
/// [`crate::universe::Universe::load_pattern`]'s placement expects. Decoding
/// nothing (empty input, or input with no recognized cells) yields
/// `(0, 0, [])`.
///
/// Trailing empty rows are dropped before computing height: `parse_rle`
/// can't tell a real row separator from redundant trailing noise (a `$`
/// right before `!`, or nothing at all for empty input, which still seeds
/// one empty row) — since height here is derived purely from row count
/// rather than an RLE header's `y = ..`, an uncounted phantom row would
/// otherwise skew centering in [`crate::universe::Universe::load_rle`].
pub fn decode(rle: &str) -> (usize, usize, Vec<f32>) {
    let mut rows = parse_rle(rle);
    while rows.last().is_some_and(Vec::is_empty) {
        rows.pop();
    }
    let height = rows.len();
    let width = rows.iter().map(Vec::len).max().unwrap_or(0);

    let mut cells = vec![0.0; width * height];
    for (y, row) in rows.iter().enumerate() {
        cells[y * width..y * width + row.len()].copy_from_slice(row);
    }

    (width, height, cells)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pads_dead_cell_runs_instead_of_dropping_them() {
        let out = parse_rle("2.3o$3o!");
        assert_eq!(
            out,
            vec![vec![0.0, 0.0, 1.0, 1.0, 1.0], vec![1.0, 1.0, 1.0]]
        );
    }

    #[test]
    fn accumulates_multi_digit_run_counts_by_place_value() {
        let out = parse_rle("12o!");
        assert_eq!(out[0].len(), 12);
    }

    #[test]
    fn strips_comment_and_dimensions_lines_ahead_of_cell_data() {
        let rle = "#N Orbium\n#C some comment\nx = 3, y = 2, rule = Lenia\n2.3o$3o!";
        let out = parse_rle(rle);
        assert_eq!(
            out,
            vec![vec![0.0, 0.0, 1.0, 1.0, 1.0], vec![1.0, 1.0, 1.0]]
        );
    }

    #[test]
    fn treats_bare_cell_data_with_no_header_unchanged() {
        let out = parse_rle("2.3o$3o!");
        assert_eq!(
            out,
            vec![vec![0.0, 0.0, 1.0, 1.0, 1.0], vec![1.0, 1.0, 1.0]]
        );
    }

    #[test]
    fn decodes_prefixed_extended_state_values() {
        // pA -> state 25 -> 25/255
        let out = parse_rle("pA!");
        assert!((out[0][0] - 25.0 / 255.0).abs() < 1e-6);
    }

    #[test]
    fn prefix_does_not_carry_over_to_a_later_unprefixed_state_char() {
        // pA -> state 25 (prefixed); B on its own -> state 2 (unprefixed),
        // not 26 (which is what a leaked 'p' prefix would compute).
        let out = parse_rle("pAB!");
        assert!((out[0][0] - 25.0 / 255.0).abs() < 1e-6);
        assert!((out[0][1] - 2.0 / 255.0).abs() < 1e-6);
    }

    #[test]
    fn decode_pads_ragged_rows_to_a_common_width() {
        // row0 is 5 wide (2 dead + 3 alive), row1 only encodes 3 alive cells
        let (width, height, cells) = decode("2.3o$3o!");
        assert_eq!((width, height), (5, 2));
        assert_eq!(
            cells,
            vec![0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0]
        );
    }

    #[test]
    fn decode_of_empty_input_yields_no_cells() {
        let (width, height, cells) = decode("");
        assert_eq!((width, height), (0, 0));
        assert!(cells.is_empty());
    }

    #[test]
    fn decode_drops_phantom_trailing_row_from_a_dollar_right_before_bang() {
        let (width, height, _cells) = decode("3o$!");
        assert_eq!((width, height), (3, 1));
    }

    #[test]
    fn count_accumulation_is_capped_instead_of_overflowing() {
        let out = parse_rle("999999999999999999999o!");
        assert_eq!(out[0].len(), MAX_RUN);
    }

    #[test]
    fn ignores_unprefixed_letters_outside_the_valid_a_to_x_range() {
        // 'z' isn't a-b/o/A-X, so it's silently ignored; 'A' -> state 1 -> 1/255
        let out = parse_rle("zA!");
        assert_eq!(out[0], vec![1.0 / 255.0]);
    }

    #[test]
    fn ignores_unprefixed_y_and_z_to_avoid_colliding_with_the_prefixed_range() {
        // unprefixed Y would otherwise compute to 25, colliding with prefixed pA
        let out = parse_rle("Y!");
        assert_eq!(out[0].len(), 0);
    }

    #[test]
    fn prefix_followed_by_dead_or_alive_shorthand_does_not_underflow() {
        // 'p' is a pending prefix, but '.'/'b'/'o' aren't A-X, so char_to_value
        // must fall back to the unprefixed shorthand instead of subtracting
        // 'A' from them and underflowing the u16.
        let out = parse_rle("p.pbpo!");
        assert_eq!(out[0], vec![0.0, 0.0, 255.0 / 255.0]);
    }
}
