use crate::generator::{Generator, GeneratorParameters};
use crate::style::Style;

fn to_lines(s: &str) -> Vec<String> {
    s.lines()
        .map(|s| {
            let mut s = s.to_owned();
            if let Some(p) = s.find("//") {
                s.truncate(p);
            }
            s.trim().to_owned()
        })
        .filter(|s| !s.is_empty())
        .collect()
}

fn find_start_at(slice: &str, at: usize, pat: &str) -> Option<usize> {
    slice[at..].find(pat).map(|i| at + i)
}

fn columns(s: &str, remove_jumps: bool) -> Option<Vec<i8>> {
    let mut ret = Vec::new();
    for (i, c) in s.chars().enumerate() {
        if match c {
            '1' | '2' | '4' | 'L' => true,
            '0' | '3' | 'M' | 'F' => false,
            _ => return None,
        } {
            ret.push(i as i8);
        }
    }
    ret.truncate(if remove_jumps { 1 } else { 2 }); // max 2 columns
    Some(ret)
}

fn params_str(params: GeneratorParameters) -> String {
    use std::f32::consts::PI;
    let mut ret = String::new();
    if params.preserve_input_repetitions.is_some() {
        ret.push('P');
    }
    if !params.disallow_footswitch {
        ret.push('F');
    }
    if let Some(ma) = params.max_angle {
        if ma > PI / 2.0 + 0.0001 {
            ret.push('C');
            if ma > PI - 0.0001 {
                ret.push('+');
            }
        }
    }
    ret
}

fn write_description(
    chart: &SMChart,
    params: GeneratorParameters,
    extra_description: Option<&String>,
    should_write_from_difficulty: bool,
) -> String {
    let mut ret = String::new();
    ret.push_str("AYEAG");
    let params_str = params_str(params);
    if !params_str.is_empty() {
        ret.push('(');
        ret.push_str(&params_str);
        ret.push(')');
    }
    if should_write_from_difficulty {
        if let Some(c) = chart.difficulty.chars().next() {
            ret.push('[');
            ret.push(c);
            ret.push(']');
        }
    }
    if let Some(extra_description) = extra_description {
        ret.push_str(" - ");
        ret.push_str(extra_description);
    }
    if !chart.description.is_empty() {
        ret.push_str(" - ");
        ret.push_str(&chart.description);
    }
    ret
}

fn write_sm_chart(
    chart: &SMChart,
    to_style: Style,
    params: GeneratorParameters,
    edit: bool,
    extra_description: Option<&String>,
    should_write_from_difficulty: bool,
    generated_notes: &str,
) -> String {
    let mut ret = String::new();
    ret.push_str("#NOTES:\n");
    ret.push_str("     ");
    ret.push_str(to_style.sm_string());
    ret.push_str(":\n     ");
    ret.push_str(&write_description(
        chart,
        params,
        extra_description,
        should_write_from_difficulty,
    ));
    ret.push_str(":\n     ");
    ret.push_str(if edit { "Edit" } else { &chart.difficulty });
    ret.push_str(":\n     ");
    ret.push_str(&chart.level.to_string());
    ret.push_str(":\n     :\n");
    ret.push_str(generated_notes);
    ret
}

fn write_ssc_chart(
    chart: &SMChart,
    to_style: Style,
    params: GeneratorParameters,
    edit: bool,
    extra_description: Option<&String>,
    should_write_from_difficulty: bool,
    generated_notes: &str,
) -> String {
    let mut ret = String::new();

    ret.push_str("#NOTEDATA:;\n");

    ret.push_str("#STEPSTYPE:");
    ret.push_str(to_style.sm_string());
    ret.push_str(";\n");

    ret.push_str("#DESCRIPTION:");
    ret.push_str(&write_description(
        chart,
        params,
        extra_description,
        should_write_from_difficulty,
    ));
    ret.push_str(";\n");

    ret.push_str("#DIFFICULTY:");
    ret.push_str(if edit { "Edit" } else { &chart.difficulty });
    ret.push_str(";\n");

    ret.push_str("#METER:");
    ret.push_str(&chart.level.to_string());
    ret.push_str(";\n");

    ret.push_str("#NOTES:\n");
    ret.push_str(generated_notes);
    ret.push_str(";\n");
    ret
}

fn row_notes(cols: &[i8], style: Style) -> String {
    let mut ret = String::new();
    let mut row = "0".repeat(style.num_cols() as usize);
    for col in cols {
        for sm_col in style.sm_cols_for_col(*col) {
            let sm_col_usize = sm_col as usize;
            row.replace_range(sm_col_usize..=sm_col_usize, "1");
        }
    }

    ret.push_str(&"0".repeat(style.extra_0s()));
    ret.push_str(&row);
    ret.push_str(&"0".repeat(style.extra_0s()));
    ret
}

fn chart_hash(chart: &SMChart) -> u64 {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let mut s = DefaultHasher::new();
    chart.notes_lines.hash(&mut s);
    s.finish()
}

fn generate_notes(
    chart: &SMChart,
    to_style: Style,
    mut params: GeneratorParameters,
) -> Result<String, String> {
    let mut ret = String::new();
    if params.seed.is_none() {
        params.seed = Some(chart_hash(chart));
    }
    let mut g = Generator::new(to_style, params);
    for l in &chart.notes_lines {
        if let Some(cols) = columns(l, params.remove_jumps) {
            let is_jump = cols.len() > 1;
            let mut out_cols = Vec::new();
            for col in cols {
                let idx = g.generate_with_input_col(col, is_jump);
                out_cols.push(idx);
            }
            ret.push_str(&row_notes(&out_cols, to_style));
            ret.push('\n');
        } else if l == "," || l == ";" {
            ret.push_str(l);
            ret.push('\n');
        } else {
            return Err(format!("unknown notes line: {}", l));
        }
    }
    Ok(ret)
}

struct SMChart {
    style: String,
    description: String,
    difficulty: String,
    level: i32,
    notes_lines: Vec<String>,
}

impl SMChart {
    fn is_autogen(&self) -> bool {
        self.description.starts_with("AYEAG") || self.description.starts_with("AUTO")
    }
}

fn parse_sm_chart(contents: &str) -> Result<SMChart, String> {
    let lines = to_lines(contents);
    if lines.len() < 6 {
        return Err("invalid metadata".to_owned());
    }

    let (metadata, notes) = lines.split_at(6);
    if metadata[0] != "#NOTES:" {
        return Err(format!("expected '#NOTES:', got '{}'", metadata[0]));
    }
    let level = metadata[4]
        .replace(":", "")
        .parse::<i32>()
        .map_err(|e| format!("Couldn't parse difficulty: {}", e))?;
    let mut style = metadata[1].to_owned();
    if style.pop() != Some(':') {
        return Err("Invalid style".to_owned());
    }
    let mut description = metadata[2].to_owned();
    if description.pop() != Some(':') {
        return Err("Invalid description".to_owned());
    }
    let mut difficulty = metadata[3].to_owned();
    if difficulty.pop() != Some(':') {
        return Err("Invalid difficulty".to_owned());
    }
    Ok(SMChart {
        style,
        description,
        difficulty,
        level,
        notes_lines: notes.iter().map(|s| s.to_owned()).collect::<Vec<String>>(),
    })
}

fn parse_sm_charts(contents: &str) -> Result<Vec<SMChart>, String> {
    let mut ret = Vec::new();
    let mut search_from = 0;
    while let Some(notes_idx) = find_start_at(contents, search_from, "#NOTES:") {
        let semicolon_idx = find_start_at(contents, notes_idx, ";")
            .ok_or("couldn't find semicolon after #NOTES".to_owned())?;
        let notes_str = &contents[notes_idx..=semicolon_idx];
        let chart =
            parse_sm_chart(notes_str).map_err(|e| format!("Couldn't parse chart: {}", e))?;
        ret.push(chart);
        search_from = semicolon_idx + 1;
    }
    Ok(ret)
}

fn parse_ssc_chart(contents: &str) -> Result<SMChart, String> {
    let lines = to_lines(contents);
    let mut style = None;
    let mut description = None;
    let mut difficulty = None;
    let mut level = None;
    let mut notes_lines = Vec::new();
    let mut new_kv = true;
    let mut cur_key = String::new();
    let mut cur_val_lines = Vec::new();
    for mut line in lines {
        if new_kv {
            if line.remove(0) != '#' {
                return Err("New key value line didn't start with #".to_owned());
            }
            let semicolon_idx = line.find(':').ok_or("couldn't find semicolon".to_owned())?;
            let (key, val) = line.split_at(semicolon_idx);
            cur_key = key.to_owned();
            line = val[1..].to_owned();
        }
        new_kv = line.ends_with(';');
        if new_kv {
            line.pop();
        }
        if !line.is_empty() {
            cur_val_lines.push(line);
        }
        if new_kv {
            match cur_key.as_str() {
                "STEPSTYPE" => {
                    if cur_val_lines.len() != 1 {
                        return Err("STEPSTYPE should be one line".to_owned());
                    }
                    style = Some(cur_val_lines.pop().unwrap());
                }
                "DESCRIPTION" => {
                    if cur_val_lines.is_empty() {
                        description = Some("".to_owned());
                    } else if cur_val_lines.len() == 1 {
                        description = Some(cur_val_lines.pop().unwrap());
                    } else {
                        return Err("DESCRIPTION should be one line".to_owned());
                    }
                }
                "DIFFICULTY" => {
                    if cur_val_lines.len() != 1 {
                        return Err("DIFFICULTY should be one line".to_owned());
                    }
                    difficulty = Some(cur_val_lines.pop().unwrap());
                }
                "METER" => {
                    if cur_val_lines.len() != 1 {
                        return Err("METER should be one line".to_owned());
                    }
                    level = Some(
                        cur_val_lines
                            .pop()
                            .unwrap()
                            .parse::<i32>()
                            .map_err(|e| format!("Couldn't parse METER: {}", e))?,
                    );
                }
                "NOTES" => {
                    notes_lines = cur_val_lines;
                }
                _ => {}
            }
            cur_val_lines = Default::default();
        }
    }
    let style = style.ok_or("No STEPSTYPE")?;
    let description = description.ok_or("No DESCRIPTION")?;
    let difficulty = difficulty.ok_or("No DIFFICULTY")?;
    let level = level.ok_or("No METER")?;
    Ok(SMChart {
        style,
        description,
        difficulty,
        level,
        notes_lines,
    })
}

fn parse_ssc_charts(contents: &str) -> Result<Vec<SMChart>, String> {
    let mut ret = Vec::new();
    let mut notedata_idx = match find_start_at(contents, 0, "#NOTEDATA:") {
        Some(i) => i,
        None => {
            return Ok(Vec::new());
        }
    };
    loop {
        match find_start_at(contents, notedata_idx + 1, "#NOTEDATA:") {
            Some(next_notedata) => {
                let chart_str = &contents[notedata_idx..next_notedata];
                let chart = parse_ssc_chart(chart_str)
                    .map_err(|e| format!("Couldn't parse chart: {}", e))?;
                ret.push(chart);
                notedata_idx = next_notedata;
            }
            None => {
                let chart_str = &contents[notedata_idx..];
                let chart = parse_ssc_chart(chart_str)
                    .map_err(|e| format!("Couldn't parse chart: {}", e))?;
                ret.push(chart);
                break;
            }
        }
    }
    Ok(ret)
}

pub fn generate(
    contents: &str,
    from_style: Style,
    to_style: Style,
    params: GeneratorParameters,
    edit: bool,
    extra_description: Option<&String>,
    is_ssc: bool,
) -> Result<String, String> {
    let mut ret = String::new();
    let mut charts = Vec::new();
    for chart in if is_ssc {
        parse_ssc_charts(contents)
    } else {
        parse_sm_charts(contents)
    }? {
        if !edit && chart.style == to_style.sm_string() && chart.difficulty != "Edit" {
            return Err(format!("already contains {} charts", to_style.sm_string()));
        }
        if chart.style != from_style.sm_string() {
            println!("  skipping {} chart", chart.style);
            continue;
        }
        if chart.is_autogen() {
            println!("  skipping existing autogen chart");
            continue;
        }
        if let Some(ignore) = params.min_difficulty {
            if chart.level < ignore {
                continue;
            }
        }
        if let Some(ignore) = params.max_difficulty {
            if chart.level > ignore {
                continue;
            }
        }
        charts.push(chart);
    }
    for chart in &charts {
        let generated_notes = generate_notes(chart, to_style, params)?;
        let write_fn = if is_ssc {
            write_ssc_chart
        } else {
            write_sm_chart
        };
        ret.push_str(&write_fn(
            chart,
            to_style,
            params,
            edit,
            extra_description,
            charts.len() > 1 && edit,
            &generated_notes,
        ));
        println!("  generated for {}", chart.difficulty);
    }

    Ok(ret)
}

#[test]
fn test_generate() {
    let params = GeneratorParameters {
        disallow_footswitch: true,
        ..GeneratorParameters::default()
    };
    {
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     17:\n     useless:\n0000\n;\n".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            params,
            false,
            None,
            false,
        );
        assert_eq!(g, Ok("#NOTES:\n     dance-double:\n     AYEAG - Zaia:\n     Challenge:\n     17:\n     :\n00000000\n;\n".to_owned()))
    }
    {
        let orig = "A\n#NOTES:\n     dance-single:\n     :\n     Challenge:\n     17:\n     useless:\n0000\n;\n".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            params,
            false,
            None,
            false,
        );
        assert_eq!(g, Ok("#NOTES:\n     dance-double:\n     AYEAG:\n     Challenge:\n     17:\n     :\n00000000\n;\n".to_owned()))
    }
    {
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     17:\n     useless:\n0000\n;\n".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            params,
            false,
            Some(&"foo".to_owned()),
            false,
        );
        assert_eq!(g, Ok("#NOTES:\n     dance-double:\n     AYEAG - foo - Zaia:\n     Challenge:\n     17:\n     :\n00000000\n;\n".to_owned()))
    }
    {
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Hard:\n     17:\n     useless:\n0000\n;\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     17:\n     useless:\n0000\n;\n".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            params,
            false,
            None,
            false,
        );
        assert_eq!(g, Ok("#NOTES:\n     dance-double:\n     AYEAG - Zaia:\n     Hard:\n     17:\n     :\n00000000\n;\n#NOTES:\n     dance-double:\n     AYEAG - Zaia:\n     Challenge:\n     17:\n     :\n00000000\n;\n".to_owned()))
    }
    {
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Hard:\n     17:\n     useless:\n0000\n;\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     17:\n     useless:\n0000\n;\n".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgSingles,
            params,
            false,
            None,
            false,
        );
        assert!(g.is_err());
    }
    {
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Hard:\n     17:\n     useless:\n0000\n;\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     17:\n     useless:\n0000\n;\n".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            params,
            true,
            None,
            false,
        );
        assert_eq!(g, Ok("#NOTES:\n     dance-double:\n     AYEAG[H] - Zaia:\n     Edit:\n     17:\n     :\n00000000\n;\n#NOTES:\n     dance-double:\n     AYEAG[C] - Zaia:\n     Edit:\n     17:\n     :\n00000000\n;\n".to_owned()))
    }
    {
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Hard:\n     17:\n     useless:\n0000\n;\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     17:\n     useless:\n0000\n;\n".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgSingles,
            params,
            true,
            None,
            false,
        );
        assert_eq!(g, Ok("#NOTES:\n     dance-single:\n     AYEAG[H] - Zaia:\n     Edit:\n     17:\n     :\n0000\n;\n#NOTES:\n     dance-single:\n     AYEAG[C] - Zaia:\n     Edit:\n     17:\n     :\n0000\n;\n".to_owned()))
    }
    {
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge;\n     17:\n     useless:\n0000\n;".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            params,
            false,
            None,
            false,
        );
        assert!(g.is_err());
    }
    {
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     17:\n     useless:\n0000\n;\n#NOTES:\n     dance-double:\n     Zaia:\n     Challenge:\n     17:\n     useless:\n00000000\n;\n".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            params,
            false,
            None,
            false,
        );
        assert!(g.is_err());
    }
    {
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     17:\n     useless:\n0000\n;\n#NOTES:\n     dance-double:\n     AYEAG - Zaia:\n     Challenge:\n     17:\n     useless:\n00000000\n;\n".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            params,
            false,
            None,
            false,
        );
        assert!(g.is_err());
    }
    {
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     17:\n     useless:\n0000\n".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            params,
            false,
            None,
            false,
        );
        assert!(g.is_err());
    }
    {
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     17:\n     useless:\n0000,0070;\n\n".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            params,
            false,
            None,
            false,
        );
        assert!(g.is_err());
    }
    {
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     17:\n     useless:\n0000\n;\n".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            GeneratorParameters::default(),
            false,
            None,
            false,
        );
        assert_eq!(g, Ok("#NOTES:\n     dance-double:\n     AYEAG(F) - Zaia:\n     Challenge:\n     17:\n     :\n00000000\n;\n".to_owned()))
    }
    {
        let params = GeneratorParameters {
            remove_jumps: true,
            ..GeneratorParameters::default()
        };
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     33:\n     useless:\n0110\n;\n".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            params,
            false,
            None,
            false,
        );
        assert_eq!(g.unwrap().matches('1').count(), 1);
    }
    {
        let params = GeneratorParameters::default();
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     33:\n     useless:\n0110\n;\n".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::PumpHalfDoubles,
            params,
            false,
            None,
            false,
        );
        let res = g.unwrap();
        assert!(res.contains("0000110000"));
        assert!(res.contains("pump-double"));
    }
    {
        let params = GeneratorParameters::default();
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     33:\n     useless:\n0110\n;\n".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::PumpMiddleFour,
            params,
            false,
            None,
            false,
        );
        let res = g.unwrap();
        assert!(res.contains("0000110000"));
        assert!(res.contains("pump-double"));
    }
    {
        let params = GeneratorParameters::default();
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     33:\n     useless:\n0110\n;\n".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::PumpDoublesBrackets,
            params,
            false,
            None,
            false,
        );
        let res = g.unwrap();
        assert!(res.contains("0010110100"));
        assert!(res.contains("pump-double"));
    }
    {
        let params = GeneratorParameters {
            min_difficulty: Some(10),
            ..params
        };
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     9:\n     useless:\n0000\n;\nB\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     10:\n     useless:\n0000\n;\n".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            params,
            false,
            None,
            false,
        );
        assert_eq!(g, Ok("#NOTES:\n     dance-double:\n     AYEAG - Zaia:\n     Challenge:\n     10:\n     :\n00000000\n;\n".to_owned()))
    }
    {
        let params = GeneratorParameters {
            max_difficulty: Some(9),
            ..params
        };
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     9:\n     useless:\n0000\n;\nB\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     10:\n     useless:\n0000\n;\n".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            params,
            false,
            None,
            false,
        );
        assert_eq!(g, Ok("#NOTES:\n     dance-double:\n     AYEAG - Zaia:\n     Challenge:\n     9:\n     :\n00000000\n;\n".to_owned()))
    }
    {
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     9:\n     useless:\n0000\n;\nB\n#NOTES:\n     dance-single:\n     AYEAG...:\n     Challenge:\n     10:\n     useless:\n0000\n;\n".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            params,
            false,
            None,
            false,
        );
        assert_eq!(g, Ok("#NOTES:\n     dance-double:\n     AYEAG - Zaia:\n     Challenge:\n     9:\n     :\n00000000\n;\n".to_owned()))
    }
    {
        let orig = "A\n#NOTEDATA:;\n#STEPSTYPE:dance-single;\n#DIFFICULTY:Challenge;\n#DESCRIPTION:wow;\n#METER:13;\n#NOTES:\n0000\n;".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            params,
            true,
            None,
            true,
        );
        assert_eq!(g, Ok("#NOTEDATA:;\n#STEPSTYPE:dance-double;\n#DESCRIPTION:AYEAG - wow;\n#DIFFICULTY:Edit;\n#METER:13;\n#NOTES:\n00000000\n;\n".to_owned()))
    }
    {
        let orig = "A\n#NOTEDATA:;\n#STEPSTYPE:dance-single;\n#DIFFICULTY:Challenge;\n#DESCRIPTION:wow;\n#METER:13;\n#NOTES:\n0000\n,\n0000\n;".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            params,
            true,
            None,
            true,
        );
        assert_eq!(g, Ok("#NOTEDATA:;\n#STEPSTYPE:dance-double;\n#DESCRIPTION:AYEAG - wow;\n#DIFFICULTY:Edit;\n#METER:13;\n#NOTES:\n00000000\n,\n00000000\n;\n".to_owned()))
    }
    {
        let orig = "A\n#NOTEDATA:;\n#STEPSTYPE:dance-single;\n#DIFFICULTY:Challenge;\n#DESCRIPTION:wow;\n#METER:13;\n#NOTES:\n0000\n;".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            params,
            false,
            None,
            true,
        );
        assert_eq!(g, Ok("#NOTEDATA:;\n#STEPSTYPE:dance-double;\n#DESCRIPTION:AYEAG - wow;\n#DIFFICULTY:Challenge;\n#METER:13;\n#NOTES:\n00000000\n;\n".to_owned()))
    }
    {
        let orig = "A\n#NOTEDATA:;\n#STEPSTYPE:dance-single;\n#DIFFICULTY:Challenge;\n#DESCRIPTION:wow;\n#METER:13;\n#NOTES:\n0000\n;#NOTEDATA:;\n#STEPSTYPE:dance-single;\n#DIFFICULTY:Challenge;\n#DESCRIPTION:wow;\n#METER:13;\n#NOTES:\n0000\n;".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            params,
            false,
            None,
            true,
        );
        assert_eq!(g, Ok("#NOTEDATA:;\n#STEPSTYPE:dance-double;\n#DESCRIPTION:AYEAG - wow;\n#DIFFICULTY:Challenge;\n#METER:13;\n#NOTES:\n00000000\n;\n#NOTEDATA:;\n#STEPSTYPE:dance-double;\n#DESCRIPTION:AYEAG - wow;\n#DIFFICULTY:Challenge;\n#METER:13;\n#NOTES:\n00000000\n;\n".to_owned()))
    }
    {
        let orig = "A\n#NOTEDATA:;\n#STEPSTYPE:dance-single;\n#DIFFICULTY:Challenge;\n#DESCRIPTION:wow;\n#NOTES:\n0000\n;\n#METER:13;\n".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            params,
            false,
            None,
            true,
        );
        assert_eq!(g, Ok("#NOTEDATA:;\n#STEPSTYPE:dance-double;\n#DESCRIPTION:AYEAG - wow;\n#DIFFICULTY:Challenge;\n#METER:13;\n#NOTES:\n00000000\n;\n".to_owned()))
    }
    {
        let orig = "A\n#NOTEDATA:;\n#STEPSTYPE:dance-single;\n#DIFFICULTY:Challenge;\n#DESCRIPTION:;\n#NOTES:\n0000\n;\n#METER:13;\n".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            params,
            false,
            None,
            true,
        );
        assert_eq!(g, Ok("#NOTEDATA:;\n#STEPSTYPE:dance-double;\n#DESCRIPTION:AYEAG;\n#DIFFICULTY:Challenge;\n#METER:13;\n#NOTES:\n00000000\n;\n".to_owned()))
    }
    {
        let orig = "A\n#NOTEDATA:;\n#DIFFICULTY:Challenge;\n#DESCRIPTION:wow;\n#METER:13;\n#NOTES:\n0000\n;".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            params,
            false,
            None,
            true,
        );
        assert_eq!(g, Err("Couldn't parse chart: No STEPSTYPE".to_owned()));
    }
    {
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     17:\n     useless:\n0110\n0110\n;\n".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            params,
            false,
            None,
            false,
        );
        assert_eq!(g, Ok("#NOTES:\n     dance-double:\n     AYEAG - Zaia:\n     Challenge:\n     17:\n     :\n00011000\n00011000\n;\n".to_owned()))
    }
    {
        // check that
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     17:\n     useless:\n0010\n0110\n0000\n1000\n;\n".to_owned();
        let g1 = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            params,
            false,
            None,
            false,
        );
        let g2 = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            params,
            false,
            None,
            false,
        );
        assert_eq!(g1, g2);
    }
}

pub fn remove_existing_autogen(contents: &str, is_ssc: bool) -> String {
    let mut res = String::new();
    let separator = if is_ssc { "#NOTEDATA:" } else { "#NOTES:" };
    let mut first = true;
    for split in contents.split(separator) {
        if first {
            res.push_str(split);
            first = false;
            continue;
        }
        if !split.contains("AYEAG") {
            res.push_str(separator);
            res.push_str(split);
        }
    }
    res
}

#[test]
fn test_remove_existing_autogen() {
    {
        let orig = "".to_owned();
        assert_eq!(remove_existing_autogen(&orig, false), orig);
    }
    {
        let orig = "HIHI".to_owned();
        assert_eq!(remove_existing_autogen(&orig, false), orig);
    }
    {
        let orig = "ABC\nDEF\n#NOTES:\nasdf:\nAYEAG - 1:\n;\n".to_owned();
        assert_eq!(
            remove_existing_autogen(&orig, false),
            "ABC\nDEF\n".to_owned()
        );
    }
    {
        let orig = "ABC\nDEF\n#NOTES:\nasdf:\nAYEAGF:\n;\n".to_owned();
        assert_eq!(
            remove_existing_autogen(&orig, false),
            "ABC\nDEF\n".to_owned()
        );
    }
    {
        let orig = "ABC\nDEF\n#NOTES:\nasdf:\nAYEAGF:\n;\n".to_owned();
        assert_eq!(remove_existing_autogen(&orig, true), orig);
    }
    {
        let orig = "ABC\nDEF\n#NOTES:\nasdf:\nAYEnoAG - 1:\n;\n".to_owned();
        assert_eq!(remove_existing_autogen(&orig, false), orig);
    }
    {
        let orig =
            "ABC\nDEF\n#NOTES:\nasdf:\nAYEAG - 1:\n;\n#NOTES:\nasdf:\nAYEnoAG - 1:\n;".to_owned();
        assert_eq!(
            remove_existing_autogen(&orig, false),
            "ABC\nDEF\n#NOTES:\nasdf:\nAYEnoAG - 1:\n;".to_owned()
        );
    }
    {
        let orig =
            "ABC\nDEF\n#NOTES:\nasdf:\nAYEnoAG - 1:\n;\n#NOTES:\nasdf:\nAYEAG - 1:\n;\n".to_owned();
        assert_eq!(
            remove_existing_autogen(&orig, false),
            "ABC\nDEF\n#NOTES:\nasdf:\nAYEnoAG - 1:\n;\n".to_owned()
        );
    }
    {
        let orig =
            "ABC\nDEF\n#NOTEDATA:\n#NOTES:;\n#DESCRIPTION:AYEnoAG - 1;\n#NOTEDATA:\n#NOTES:;\n#DESCRIPTION:AYEAG - 1:\n;\n".to_owned();
        assert_eq!(
            remove_existing_autogen(&orig, true),
            "ABC\nDEF\n#NOTEDATA:\n#NOTES:;\n#DESCRIPTION:AYEnoAG - 1;\n".to_owned()
        );
    }
}
