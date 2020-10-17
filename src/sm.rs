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
        }
    }
    ret
}

fn generate_chart(
    notes_str: &str,
    from_style: Style,
    to_style: Style,
    params: GeneratorParameters,
) -> Result<String, String> {
    let mut ret = String::new();

    let lines = to_lines(notes_str);
    if lines.len() < 6 {
        return Err("invalid metadata".to_owned());
    }

    let (metadata, notes) = lines.split_at(6);
    if metadata[0] != "#NOTES:" {
        return Err(format!("expected '#NOTES:', got '{}'", metadata[0]));
    }
    ret.push_str("#NOTES:\n");
    if metadata[1] != format!("{}:", from_style.sm_string()) {
        println!(
            "  skipping {} chart",
            &metadata[1][0..(metadata[1].len() - 1)]
        );
        return Ok("".to_owned());
    }
    ret.push_str("     ");
    ret.push_str(to_style.sm_string());
    ret.push_str(":\n     ");
    ret.push_str("AYEAG");
    let params_str = params_str(params);
    ret.push_str(&params_str);
    ret.push_str(" - ");
    if to_style == from_style {
        ret.push_str(&metadata[3].replace(":", ""));
        ret.push_str(" - ");
    }
    ret.push_str(&metadata[2]);
    ret.push_str("\n     ");
    ret.push_str(if to_style == from_style {
        "Edit:"
    } else {
        &metadata[3]
    });
    ret.push_str("\n     ");
    ret.push_str(&metadata[4]);
    ret.push_str("\n     :\n");

    let mut gen = Generator::new(to_style, params);
    for l in notes {
        if let Some(cols) = columns(&l, params.remove_jumps) {
            let mut out_cols = Vec::new();
            for col in cols {
                let idx = gen.gen_with_input_col(col);
                out_cols.push(idx);
            }
            let row = (0..(to_style.num_cols()))
                .map(|c| if out_cols.contains(&c) { '1' } else { '0' })
                .collect::<String>();
            ret.push_str(&row);
            ret.push('\n');
        } else if l == "," || l == ";" {
            ret.push_str(l);
            ret.push_str("\n");
        } else {
            return Err(format!("unknown notes line: {}", l));
        }
    }
    println!(
        "  generated for {}",
        &metadata[3][0..(metadata[3].len() - 1)]
    );
    Ok(ret)
}

pub fn generate(
    contents: &str,
    from_style: Style,
    to_style: Style,
    params: GeneratorParameters,
) -> Result<String, String> {
    if from_style != to_style && contents.contains(to_style.sm_string()) {
        return Err(format!("already contains {} charts", to_style.sm_string()));
    }
    let mut ret = String::new();
    let mut search_from = 0;
    while let Some(notes_idx) = find_start_at(&contents, search_from, "#NOTES:") {
        let semicolon_idx = match find_start_at(&contents, notes_idx, ";") {
            Some(i) => i,
            None => {
                return Err("couldn't find semicolon after #NOTES".to_owned());
            }
        };
        let notes_str = &contents[notes_idx..=semicolon_idx];
        ret.push_str(&generate_chart(notes_str, from_style, to_style, params)?);
        search_from = semicolon_idx + 1;
    }

    Ok(ret)
}

#[test]
fn test_generate() {
    let mut params = GeneratorParameters::default();
    params.disallow_footswitch = true;
    {
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     17:\n     useless:\n0000\n;\n".to_owned();
        let g = generate(&orig, Style::ItgSingles, Style::ItgDoubles, params);
        assert_eq!(g, Ok("#NOTES:\n     dance-double:\n     AYEAG - Zaia:\n     Challenge:\n     17:\n     :\n00000000\n;\n".to_owned()))
    }
    {
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Hard:\n     17:\n     useless:\n0000\n;\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     17:\n     useless:\n0000\n;\n".to_owned();
        let g = generate(&orig, Style::ItgSingles, Style::ItgDoubles, params);
        assert_eq!(g, Ok("#NOTES:\n     dance-double:\n     AYEAG - Zaia:\n     Hard:\n     17:\n     :\n00000000\n;\n#NOTES:\n     dance-double:\n     AYEAG - Zaia:\n     Challenge:\n     17:\n     :\n00000000\n;\n".to_owned()))
    }
    {
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Hard:\n     17:\n     useless:\n0000\n;\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     17:\n     useless:\n0000\n;\n".to_owned();
        let g = generate(&orig, Style::ItgSingles, Style::ItgSingles, params);
        assert_eq!(g, Ok("#NOTES:\n     dance-single:\n     AYEAG - Hard - Zaia:\n     Edit:\n     17:\n     :\n0000\n;\n#NOTES:\n     dance-single:\n     AYEAG - Challenge - Zaia:\n     Edit:\n     17:\n     :\n0000\n;\n".to_owned()))
    }
    {
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge;\n     17:\n     useless:\n0000\n;".to_owned();
        let g = generate(&orig, Style::ItgSingles, Style::ItgDoubles, params);
        assert!(g.is_err());
    }
    {
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     17:\n     useless:\n0000\n;\n#NOTES:\n     dance-double:\n     Zaia:\n     Challenge:\n     17:\n     useless:\n00000000\n;\n".to_owned();
        let g = generate(&orig, Style::ItgSingles, Style::ItgDoubles, params);
        assert!(g.is_err());
    }
    {
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     17:\n     useless:\n0000\n".to_owned();
        let g = generate(&orig, Style::ItgSingles, Style::ItgDoubles, params);
        assert!(g.is_err());
    }
    {
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     17:\n     useless:\n0000,0070;\n\n".to_owned();
        let g = generate(&orig, Style::ItgSingles, Style::ItgDoubles, params);
        assert!(g.is_err());
    }
    {
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     17:\n     useless:\n0000\n;\n".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            GeneratorParameters::default(),
        );
        assert_eq!(g, Ok("#NOTES:\n     dance-double:\n     AYEAGF - Zaia:\n     Challenge:\n     17:\n     :\n00000000\n;\n".to_owned()))
    }
    {
        let mut params = GeneratorParameters::default();
        params.remove_jumps = true;
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     33:\n     useless:\n0110\n;\n".to_owned();
        let g = generate(&orig, Style::ItgSingles, Style::ItgDoubles, params);
        assert_eq!(g.unwrap().matches('1').count(), 1);
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
