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
            if ma > PI * 3.0 / 4.0 + 0.0001 {
                ret.push('+');
            }
        }
    }
    ret
}

fn write_description(
    chart: &SMChart,
    from_style: Style,
    to_style: Style,
    params: GeneratorParameters,
    extra_description: Option<&String>,
) -> String {
    let mut ret = String::new();
    ret.push_str("AYEAG");
    let params_str = params_str(params);
    if !params_str.is_empty() {
        ret.push('(');
        ret.push_str(&params_str);
        ret.push(')');
    }
    if to_style == from_style {
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
    from_style: Style,
    to_style: Style,
    params: GeneratorParameters,
    edit: bool,
    extra_description: Option<&String>,
    generated_notes: &String,
) -> String {
    let mut ret = String::new();
    ret.push_str("#NOTES:\n");
    ret.push_str("     ");
    ret.push_str(to_style.sm_string());
    ret.push_str(":\n     ");
    ret.push_str(&write_description(
        chart,
        from_style,
        to_style,
        params,
        extra_description,
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
    from_style: Style,
    to_style: Style,
    params: GeneratorParameters,
    edit: bool,
    extra_description: Option<&String>,
    generated_notes: &String,
) -> String {
    let mut ret = String::new();

    ret.push_str("#NOTEDATA:;\n");

    ret.push_str("#STEPSTYPE:");
    ret.push_str(to_style.sm_string());
    ret.push_str(";\n");

    ret.push_str("#DESCRIPTION:");
    ret.push_str(&write_description(
        chart,
        from_style,
        to_style,
        params,
        extra_description,
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
    ret
}

fn generate_notes(
    chart: &SMChart,
    to_style: Style,
    params: GeneratorParameters,
) -> Result<String, String> {
    let mut ret = String::new();
    let mut gen = Generator::new(to_style, params);
    for l in &chart.notes_lines {
        if let Some(cols) = columns(&l, params.remove_jumps) {
            let mut out_cols = Vec::new();
            for col in cols {
                let idx = gen.gen_with_input_col(col);
                out_cols.push(idx);
            }
            let row = (0..(to_style.num_cols()))
                .map(|c| if out_cols.contains(&c) { '1' } else { '0' })
                .collect::<String>();
            ret.push_str(&"0".repeat(to_style.extra_0s()));
            ret.push_str(&row);
            ret.push_str(&"0".repeat(to_style.extra_0s()));
            ret.push('\n');
        } else if l == "," || l == ";" {
            ret.push_str(l);
            ret.push_str("\n");
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
    while let Some(notes_idx) = find_start_at(&contents, search_from, "#NOTES:") {
        let semicolon_idx = find_start_at(&contents, notes_idx, ";")
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
    let mut found_notes = false;
    let mut done = false;
    fn value_for_key(line: &str, key: &str) -> Option<String> {
        let mut line = line.to_owned();
        if line.pop() != Some(';') {
            return None;
        }
        let prefix = format!("#{}:", key);
        if !line.starts_with(&prefix) {
            return None;
        }
        Some(line[prefix.len()..].to_owned())
    }
    for line in lines {
        if done {
            return Err("found more lines after #NOTES:;".to_owned());
        }
        if found_notes {
            if line == ";" {
                done = true;
            }
            notes_lines.push(line);
        } else {
            if let Some(s) = value_for_key(&line, "STEPSTYPE") {
                style = Some(s);
            } else if let Some(d) = value_for_key(&line, "DESCRIPTION") {
                description = Some(d);
            } else if let Some(d) = value_for_key(&line, "DIFFICULTY") {
                difficulty = Some(d);
            } else if let Some(d) = value_for_key(&line, "METER") {
                level = Some(
                    d.parse::<i32>()
                        .map_err(|e| format!("Couldn't parse METER: {}", e))?,
                );
            } else if line == "#NOTES:" {
                found_notes = true;
            }
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
    let mut search_from = 0;
    while let Some(notedata_idx) = find_start_at(&contents, search_from, "#NOTEDATA:") {
        let notes_idx = find_start_at(&contents, notedata_idx, "NOTES:")
            .ok_or("couldn't find #NOTES after #NOTEDATA".to_owned())?;
        let semicolon_idx = find_start_at(&contents, notes_idx, ";")
            .ok_or("couldn't find semicolon after #NOTES".to_owned())?;
        let chart_str = &contents[notedata_idx..=semicolon_idx];
        let chart =
            parse_ssc_chart(chart_str).map_err(|e| format!("Couldn't parse chart: {}", e))?;
        ret.push(chart);
        search_from = semicolon_idx + 1;
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
    let charts = if is_ssc {
        parse_ssc_charts(contents)
    } else {
        parse_sm_charts(contents)
    }?;
    for chart in charts {
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
        let generated_notes = generate_notes(&chart, to_style, params)?;
        ret.push_str(&if is_ssc {
            write_ssc_chart(
                &chart,
                from_style,
                to_style,
                params,
                edit,
                extra_description,
                &generated_notes,
            )
        } else {
            write_sm_chart(
                &chart,
                from_style,
                to_style,
                params,
                edit,
                extra_description,
                &generated_notes,
            )
        });
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
}

pub fn remove_existing_autogen(contents: &str) -> Result<String, String> {
    let mut ret = String::new();
    let mut search_from = 0;
    while let Some(notes_idx) = find_start_at(&contents, search_from, "#NOTES:") {
        // Add everything up until the latest #NOTES
        ret.push_str(&contents[search_from..notes_idx]);
        let semicolon_idx = match find_start_at(&contents, notes_idx, ";") {
            Some(i) => i,
            None => {
                return Err("couldn't find semicolon after #NOTES".to_owned());
            }
        };
        let notes_str = &contents[notes_idx..=semicolon_idx];
        if !notes_str.contains("AYEAG") {
            ret.push_str(notes_str);
        }
        search_from = semicolon_idx + 1;
    }
    // Add everything after the last semicolon (or beginning if no #NOTES)
    ret.push_str(&contents[search_from..]);
    Ok(ret)
}

#[test]
fn test_remove_existing_autogen() {
    {
        let orig = "".to_owned();
        assert_eq!(remove_existing_autogen(&orig).unwrap(), orig);
    }
    {
        let orig = "HIHI".to_owned();
        assert_eq!(remove_existing_autogen(&orig).unwrap(), orig);
    }
    {
        let orig = "ABC\nDEF\n#NOTES:\nasdf:\nAYEAG - 1:\n;\n".to_owned();
        assert_eq!(
            remove_existing_autogen(&orig).unwrap(),
            "ABC\nDEF\n\n".to_owned()
        );
    }
    {
        let orig = "ABC\nDEF\n#NOTES:\nasdf:\nAYEAGF:\n;\n".to_owned();
        assert_eq!(
            remove_existing_autogen(&orig).unwrap(),
            "ABC\nDEF\n\n".to_owned()
        );
    }
    {
        let orig = "ABC\nDEF\n#NOTES:\nasdf:\nAYEnoAG - 1:\n;\n".to_owned();
        assert_eq!(remove_existing_autogen(&orig).unwrap(), orig);
    }
    {
        let orig =
            "ABC\nDEF\n#NOTES:\nasdf:\nAYEAG - 1:\n;\n#NOTES:\nasdf:\nAYEnoAG - 1:\n;".to_owned();
        assert_eq!(
            remove_existing_autogen(&orig).unwrap(),
            "ABC\nDEF\n\n#NOTES:\nasdf:\nAYEnoAG - 1:\n;".to_owned()
        );
    }
    {
        let orig =
            "ABC\nDEF\n#NOTES:\nasdf:\nAYEnoAG - 1:\n;\n#NOTES:\nasdf:\nAYEAG - 1:\n;\n".to_owned();
        assert_eq!(
            remove_existing_autogen(&orig).unwrap(),
            "ABC\nDEF\n#NOTES:\nasdf:\nAYEnoAG - 1:\n;\n\n".to_owned()
        );
    }
}
